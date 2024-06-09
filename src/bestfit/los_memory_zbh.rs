const LOS_OK: u32 = 0;

//处在条件编译内
fn Os_Mem_Integrity_Multi_Check() {
    if Los_Mem_Integrity_Check(M_AUC_SYS_MEM1).is_ok() {
        println!("system memcheck over, all passed!");
        #[cfg(feature = "LOSCFG_SHELL_EXCINFO_DUMP")]
        Write_Exc_Info_To_Buf("system memcheck over, all passed!\n");
    }

    #[cfg(feature = "LOSCFG_EXC_INTERACTION")]
    {
        if Los_Mem_Integrity_Check(M_AUC_SYS_MEM0).is_ok() {
            println!("exc interaction memcheck over, all passed!");
            #[cfg(feature = "LOSCFG_SHELL_EXCINFO_DUMP")]
            Write_Exc_Info_To_Buf("exc interaction memcheck over, all passed!\n");
        }
    }
}

//TODO:这里有个条件编译条件else， 以及下面有#endif， 但#ifdef不在范围中， 最后拼起来的时候需要注意。

fn Los_Mem_Integrity_Check(pool: &[u8]) -> u32 {
    LOS_OK
}
//TODO:与上面方法重名，还不清楚使用逻辑
fn Os_Mem_Integrity_Multi_Check(){}

fn Os_Mem_Node_Debug_Operate(pool: &mut [u8], alloc_node: &mut Los_Mem_Dyn_Node, size: u32) {
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(alloc_node);
    }

    #[cfg(feature = "LOSCFG_MEM_LEAKCHECK")]
    {
        Os_Mem_Link_Register_Record(alloc_node);
    }
}

fn Os_Mem_Info_Get(pool: *const c_void, pool_status: &mut Los_Mem_Pool_Status) -> u32 {
    unsafe {
        let pool_info = &*(pool as *const Los_Mem_Pool_Info);
        let mut tmp_node: *mut Los_Mem_Dyn_Node = Os_Mem_End_Node(pool, pool_info.pool_size);
        tmp_node = Os_Mem_Align(tmp_node, OS_MEM_ALIGN_SIZE);

        if !Os_Mem_Magic_Valid((*tmp_node).self_node.magic) {
            Print_Err("wrong mem pool", pool_info, line!() as i32);
            return LOS_NOK;
        }

        let mut total_used_size = 0;
        let mut total_free_size = 0;
        let mut max_free_node_size = 0;
        let mut used_node_num = 0;
        let mut free_node_num = 0;

        tmp_node = Os_Mem_First_Node(pool_info);

        while tmp_node <= Os_Mem_End_Node(pool_info as *const _, pool_info.pool_size) {
            if !Os_Mem_Node_Get_Used_Flag((*tmp_node).self_node.size_and_flag) {
                free_node_num += 1;
                let node_size = Os_Mem_Node_Get_Size((*tmp_node).self_node.size_and_flag);
                total_free_size += node_size;
                if (max_free_node_size < node_size) {
                    max_free_node_size = node_size;
                }
            } else {
                used_node_num += 1;
                total_used_size += Os_Mem_Node_Get_Size((*tmp_node).self_node.size_and_flag);
            }

            tmp_node = Os_Mem_Next_Node(tmp_node);
        }

        pool_status.uw_total_used_size = total_used_size;
        pool_status.uw_total_free_size = total_free_size;
        pool_status.uw_max_free_node_size = max_free_node_size;
        pool_status.uw_used_node_num = used_node_num;
        pool_status.uw_free_node_num = free_node_num;
        #[cfg(feature = "mem_task_stat")]
        {
            pool_status.uw_usage_water_line = pool_info.stat.mem_total_peak;
        }

        LOS_OK
    }
}

fn Os_Mem_Info_Print(pool: *const c_void){
    unsafe{
        let pool_info = &*(pool as *const Los_Mem_Pool_Info);
        let mut status: LosMemPoolStatus = Default::default();

        if(Os_Mem_Info_Get(pool, status) == LOS_NOK){
            return;
        }
        #[cfg(feature = "loscfg_mem_task_stat")]{
            println!(
                "pool addr          pool size    used size     free size    max free node size   used node num     free node num      UsageWaterLine"
            );
            println!(
                "---------------    --------     -------       --------     --------------       -------------      ------------      ------------"
            );
            println!(
                "{:16p}   0x{:08x}   0x{:08x}    0x{:08x}   0x{:016x}   0x{:013x}    0x{:013x}    0x{:013x}",
                pool_info.pool,
                pool_info.pool_size,
                status.uw_total_used_size,
                status.uw_total_free_size,
                status.uw_max_free_node_size,
                status.uw_used_node_num,
                status.uw_free_node_num,
                status.uw_usage_water_line
            );
        }
        #[cfg(not(feature = "loscfg_mem_task_stat"))]{
            println!(
                "pool addr          pool size    used size     free size    max free node size   used node num     free node num"
            );
            println!(
                "---------------    --------     -------       --------     --------------       -------------      ------------"
            );
            println!(
                "{:16p}   0x{:08x}   0x{:08x}    0x{:08x}   0x{:016x}   0x{:013x}    0x{:013x}",
                pool_info.pool,
                pool_info.pool_size,
                status.uw_total_used_size,
                status.uw_total_free_size,
                status.uw_max_free_node_size,
                status.uw_used_node_num,
                status.uw_free_node_num
            );
        }
    }
}

#[inline]
fn Os_Mem_Info_Alert(pool: *const c_void, alloc_size: u32){
    #[cfg(feature = "loscfg_mem_debug")]{
        Print_Err("---------------------------------------------------\
        --------------------------------------------------------");
        Os_Mem_Info_Print(pool);
        Print_Err(&format!(
        "[{}] No suitable free block, require free node size: 0x{:x}",
        std::module_path!(), alloc_size));
        Print_Err("---------------------------------------------------\
                --------------------------------------------------------");
    }
}

/*
 * Description : Allocate node from Memory pool
 * Input       : pool  --- Pointer to memory pool
 *               size  --- Size of memory in bytes to allocate
 * Return      : Pointer to allocated memory
 */
fn Os_Mem_Alloc_With_Check(pool: &mut LosMemPoolInfo, size: u32) 
    -> *mut c_void{
    let mut alloc_node: *mut LosMemDynNode = ptr::null_mut();
    let alloc_size: usize;

    #[cfg(feature = "loscfg_base_mem_node_integrity_check")]{
        let mut tmp_node = std::ptr::null_mut();
        let mut pre_node = std::ptr::null_mut();
    }
    let first_node = (Os_Mem_Head_Addr!(pool) as *mut u8).wrapping_add(OS_DLNK_HEAD_SIZE) as *const ();

    #[cfg(feature = "loscfg_base_mem_node_integrity_check")]{
        if(Os_Mem_Integrity_Check(pool, &mut tmp_node, &mut pre_node)){
            Os_Mem_Integrity_Check_Error(tmp_node, pre_node);
            return None;
        }
    }
    alloc_size = Os_Mem_Align(size + Os_Mem_Node_Head_Size!(), Os_Mem_Align_Size!());
    alloc_node = Os_Mem_Find_Suitable_Free_Block(pool, alloc_size);
    if alloc_node.is_null() {
        Os_Mem_Info_Alert(pool, alloc_size);
        return None;
    }
    if alloc_size + Os_Mem_Node_Head_Size!() + Os_Mem_Align_Size!() <= (*alloc_node).self_node.size_and_flag {
        Os_Mem_Split_Node(pool, alloc_node, alloc_size);
    }
    Os_Mem_List_Delete(&mut (*alloc_node).self_node.free_node_info, first_node);
    Os_Mem_Set_Magic_Num_And_Task_Id(alloc_node);
    Os_Mem_Node_Set_Used_Flag(&mut (*alloc_node).self_node.size_and_flag);
    Os_Mem_Add_Used(&mut pool.stat, Os_Mem_Node_Get_Size((*alloc_node).self_node.size_and_flag),
                    Os_Mem_Task_Id_Get(alloc_node));
    Os_Mem_Node_Debug_Operate(pool, alloc_node, size);

    #[cfg(feature = "LOSCFG_KERNEL_LMS")]
    {
        if !g_lms_malloc_hook.is_null() {
            g_lms_malloc_hook(unsafe { alloc_node.offset(1) });
        }
    }
    Some(unsafe { alloc_node.offset(1) })
}

/*
 * Description : reAlloc a smaller memory node
 * Input       : pool      --- Pointer to memory pool
 *               allocSize --- the size of new node which will be alloced
 *               node      --- the node which will be realloced
 *               nodeSize  --- the size of old node
 * Output      : node      --- pointer to the new node after realloc
 */
#[inline]
fn Os_Mem_ReAlloc_Smaller(pool: &mut LosMemPoolInfo, alloc_size: u32, node: &mut LosMemDynNode, nodeSize: u32){
    if alloc_size + OS_MEM_NODE_HEAD_SIZE!() + OS_MEM_ALIGN_SIZE!() <= node_size {
        node.self_node.size_and_flag = node_size;
        Os_Mem_Split_Node(pool, node, alloc_size);
        Os_Mem_Node_Set_Used_Flag!(node.self_node.size_and_flag)
        
        #[cfg(feature = "loscfg_mem_head_backup")]
        Os_Mem_Node_Save(node);

        Os_Mem_Reduce_Used(&mut pool.stat, node_size - alloc_size, Os_Mem_Task_Id_Get(node));
    }

    #[cfg(feature = "loscfg_mem_leakcheck")]
    Os_Mem_Link_Register_Record(node);
}