const LOS_OK: u32 = 0;
const LOS_NOK: u32 = 0;

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
    if alloc_size + Os_Mem_Node_Head_Size!() + Os_Mem_Align_Size!() <= node_size {
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

/*
 * Description : reAlloc a Bigger memory node after merge node and nextNode
 * Input       : pool      --- Pointer to memory pool
 *               allocSize --- the size of new node which will be alloced
 *               node      --- the node which will be realloced
 *               nodeSize  --- the size of old node
 *               nextNode  --- pointer next node which will be merged
 * Output      : node      --- pointer to the new node after realloc
 */

#[inline]
fn Os_Mem_Merge_Node_For_ReAlloc_Bigger(pool: &mut LosMemPoolInfo, alloc_size: u32, node: &mut LosMemDynNode, node_size: u32, next_node: &mut LosMemDynNode){
    let first_node = (Os_Mem_Head_Addr!(pool) as *mut u8).add(Os_Dlnk_Head_Size!()) as *const ();

    node.self_node.size_and_flag.set(node_size);
    Os_Mem_List_Delete(&mut next_node.self_node.free_node_info, first_node);
    Os_Mem_Merge_Node(next_node);
    if alloc_size + Os_Mem_Node_Head_Size!() + Os_Mem_Align_Size!() <= node.self_node.size_and_flag {
        Os_Mem_Split_Node(pool, node, alloc_size);
    }
    //TODO:该宏定义在memstat_pri中
    Os_Mem_Add_Used!(&mut pool.stat, node.self_node.size_and_flag - node_size, Os_Mem_Task_Id_Get(node));

    Os_Mem_Node_Set_Used_Flag!(node.self_node.size_and_flag);

    #[cfg(feature = "loscfg_mem_head_backup")]
    Os_Mem_Node_Save(node);

    #[cfg(feature = "loscfg_mem_leakcheck")]
    Os_Mem_Link_Register_Record(node);
}


/*
 * Description : reAlloc a Bigger memory node after merge node and nextNode
 * Input       : pool      --- Pointer to memory pool
 *               allocSize --- the size of new node which will be alloced
 *               node      --- the node which will be realloced
 *               nodeSize  --- the size of old node
 *               nextNode  --- pointer next node which will be merged
 * Output      : node      --- pointer to the new node after realloc
 */
fn Os_Mem_Init(pool: *mut c_void, size: u32) -> u32 {
    let pool_info = pool as *mut LosMemPoolInfo;
    let mut new_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut end_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut list_node_head: *mut LOS_DL_LIST = std::ptr::null_mut();
    let mut pool_size = size;

    #[cfg(feature = "LOSCFG_KERNEL_LMS")]
    {
        if !g_lms_Mem_Init_Hook.is_null() {
            pool_size = g_lms_Mem_Init_Hook(pool, size);
            if pool_size == 0 {
                pool_size = size;
            }
        }
    }
    unsafe {
        (*pool_info).pool = pool;
        (*pool_info).pool_size = pool_size;

        Os_Dlnk_Init_Multi_Head(Os_Mem_Head_Addr(pool));
        new_node = Os_Mem_First_Node(pool);
        (*new_node).self_node.size_and_flag = (pool_size - ((new_node as u32) - (pool as u32)) - Os_Mem_Node_Head_Size!());
        (*new_node).self_node.pre_node = Os_Mem_End_Node(pool, pool_size) as *mut LosMemDynNode;
        list_node_head = Os_Mem_Head(pool, (*new_node).self_node.size_and_flag);
        if list_node_head.is_null() {
            return LOS_NOK;
        }

        Los_List_Tail_Insert(list_node_head, &mut (*new_node).self_node.free_node_info);
        end_node = os_mem_end_node(pool, pool_size) as *mut LosMemDynNode;
        /*std::ptr::write_bytes(end_node,std::mem::size_of::<end_node>(), 0, std::mem::size_of::<LosMemDynNode>());*/
        Memset_S(nd_node,std::mem::size_of::<end_node>(), 0, std::mem::size_of::<LosMemDynNode>());
        (*end_node).self_node.pre_node = new_node;
        (*end_node).self_node.size_and_flag = Os_Mem_Node_Head_Size!() ;
        Os_Mem_Node_Set_Used_Flag(&mut (*end_node).self_node.size_and_flag);
        Os_Mem_Set_Magic_Num_And_Task_Id(end_node);

        #[cfg(feature = "LOSCFG_MEM_TASK_STAT")]
        {
            let stat_size = std::mem::size_of_val(&(*pool_info).stat);
            /*std::ptr::write_bytes(&mut (*pool_info).stat, 0, stat_size);*/
            Memset_S(&mut (*pool_info).stat,stat_size, 0, stat_size);
            (*pool_info).stat.mem_total_used = std::mem::size_of::<LosMemPoolInfo>() + os_multi_dlnk_head_size!() +
                                               os_mem_node_get_size((*end_node).self_node.size_and_flag);
            (*pool_info).stat.mem_total_peak = (*pool_info).stat.mem_total_used;
        }

        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        {
            Os_Mem_Node_Save(new_node);
            Os_Mem_Node_Save(end_node);
        }
    }

    LOS_OK
}

fn Los_Mem_Init(pool: *mut c_void, mut size: u32) -> u32 {
    let mut int_save: u32;

    if pool.is_null() || size < Os_Mem_Min_Pool_Size!() {
        return LOS_NOK;
    }

    if !Is_Aligned!(size, Os_Mem_Align_Size) || !Is_Aligned!(pool as u32, Os_Mem_Align_Size) {
        println!("pool [{:?}, {:?}) size 0x{:x} should be aligned with OS_MEM_ALIGN_SIZE\n",
                 pool, unsafe { pool.offset(size as isize) }, size);
        size = Os_Mem_Align!(size, Os_Mem_Align_Size) - Os_Mem_Align_Size;
    }

    Mem_Lock!(int_save);
    if Os_Mem_Mul_Pool_Init(pool, size) != 0 {
        Mem_Unlock!(int_save);
        return LOS_NOK;
    }

    if Os_Mem_Init(pool, size) != LOS_OK {
        Os_Mem_Mul_Pool_Deinit(pool);
        Mem_Unlock!(int_save);
        return LOS_NOK;
    }

    Os_Slab_Mem_Init(pool, size);
    Mem_Unlock!(int_save);

    Los_Trace!(MEM_INFO_REQ, pool);
    LOS_OK
}

fn LOS_Mem_Alloc(pool: *mut c_void, size: u32) -> *mut c_void {
    let mut ptr: *mut c_void = std::ptr::null_mut();
    let mut int_save: u32;

    if pool.is_null() || size == 0 {
        return std::ptr::null_mut();
    }

    if !g_Malloc_Hook.is_null() {
        unsafe { g_Malloc_Hook() };
    }

    Mem_Lock!(int_save);
    loop {
        if Os_Mem_Node_Get_Used_Flag!(size) || Os_Mem_Node_Get_Aligned_Flag!(size) {
            break;
        }

        ptr = Os_Slab_Mem_Alloc(pool, size);
        if ptr.is_null() {
            ptr = Os_Mem_Alloc_With_Check(pool, size);
        }
        break;
    }

    Mem_Unlock!(int_save);

    Los_Trace!(Mem_Alloc, pool, ptr as u32, size);
    ptr
}

fn LOS_Mem_Alloc_Align(pool: *mut c_void, size: u32, boundary: u32) -> *mut c_void {
    let mut use_size: u32;
    let mut gap_size: u32;
    let mut ptr: *mut c_void = std::ptr::null_mut();
    let mut aligned_ptr: *mut c_void = std::ptr::null_mut();
    let mut alloc_node: *mut Los_Mem_Dyn_Node = std::ptr::null_mut();
    let mut int_save: u32;

    if pool.is_null() || size == 0 || boundary == 0 || !Is_Pow_Two!(boundary) ||
        !Is_Aligned!(boundary, std::mem::size_of::<*mut c_void>() as u32) {
        return std::ptr::null_mut();
    }

    Mem_Lock!(int_save);
    loop {
        /*
         * sizeof(gapSize) bytes stores offset between alignedPtr and ptr,
         * the ptr has been OS_MEM_ALIGN_SIZE(4 or 8) aligned, so maximum
         * offset between alignedPtr and ptr is boundary - OS_MEM_ALIGN_SIZE
         */
        if (boundary - std::mem::size_of::<u32>() as u32) > (u32::MAX - size) {
            break;
        }

        use_size = (size + boundary) - std::mem::size_of::<u32>() as u32;
        if Os_Mem_Node_Get_Used_Flag!(use_size) || Os_Mem_Node_Get_Aligned_Flag!(use_size) {
            break;
        }

        ptr = Os_Mem_Alloc_With_Check(pool, use_size);

        aligned_ptr = Os_Mem_Align!(ptr, boundary) as *mut c_void;
        if ptr == aligned_ptr {
            break;
        }
        /* store gapSize in address (ptr -4), it will be checked while free */
        gap_size = (aligned_ptr as u32) - (ptr as u32);
        alloc_node = (ptr as *mut Los_Mem_Dyn_Node).offset(-1);
        Os_Mem_Node_Set_Aligned_Flag!((*alloc_node).self_node.size_and_flag);

        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        Os_Mem_Node_Save_With_Gap_Size(alloc_node, gap_size);

        Os_Mem_Node_Set_Aligned_Flag!(gap_size);
        *((aligned_ptr as u32 - std::mem::size_of::<u32>() as u32) as *mut u32) = gap_size;

        ptr = aligned_ptr;
        break;
    }

    Mem_Unlock!(int_save);
    //TODO:枚举类型尚不知是否定义
    Los_Trace!(Mem_Alloc_Align, pool, ptr as u32, size, boundary);
    ptr
}

fn Os_Do_Mem_Free(pool: *mut c_void, ptr: *mut c_void, node: &Los_Mem_Dyn_Node){
    Os_Mem_Check_Used_Node(pool, node);
    Os_Mem_Free_Node(node, pool);

    #[cfg(feature = "LOSCFG_KERNEL_LMS")]{
        if !g_lms_Free_Hook.is_null() {
            g_lms_Free_Hook(ptr);
        }
    }
} 

fn Os_Mem_Free(pool: *mut c_void, ptr: *const c_void) -> u32 {
    let mut ret: u32 = LOS_OK;
    let mut gap_size: u32;
    let mut node: *mut Los_Mem_Dyn_Node = std::ptr::null_mut();

    loop {
        gap_size = *((ptr as u32 - std::mem::size_of::<u32>() as u32) as *mut u32);
        if Os_Mem_Node_Get_Aligned_Flag!(gap_size) && Os_Mem_Node_Get_Used_Flag!(gap_size) {
            eprintln!("[{}:{}]: gapSize:0x{:x} error", “Os_Mem_Free()”, line!(), gap_size);
            return ret;
        }

        node = (ptr as u32 - Os_Mem_Node_Head_Size!()) as *mut Los_Mem_Dyn_Node;

        if Os_Mem_Node_Get_Aligned_Flag!(gap_size) {
            gap_size = Os_Mem_Node_Get_Aligned_Gapsize!(gap_size);
            if (gap_size & (Os_Mem_Align_Size!() - 1)) != 0 || gap_size > (ptr as u32 - Os_Mem_Node_Head_Size!()) {
                eprintln!("illegal gapSize: 0x{:x}", gap_size);
                break;
            }
            node = (ptr as u32 - gap_size - Os_Mem_Node_Head_Size!()) as *mut Los_Mem_Dyn_Node;
        }

        #[cfg(not(feature = "LOSCFG_MEM_HEAD_BACKUP"))]
        Os_Do_Mem_Free(pool, ptr, node);
        break;
    }

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        ret = Os_Mem_Backup_Check_And_Restore(pool, ptr, node);
        if ret == 0 {
            Os_Do_Mem_Free(pool, ptr, node);
        }
    }

    ret
}

fn Los_Mem_Free(pool: *mut c_void, ptr: *mut c_void) -> u32 {
    let mut ret: u32;
    let int_save: u32;

    if pool.is_null() || ptr.is_null() || !Is_Aligned!(pool as u32, std::mem::size_of::<*mut c_void>()) || !Is_Aligned!(ptr as u32, std::mem::size_of::<*mut c_void>()) {
        return LOS_NOK;
    }

    Mem_Lock!(int_save);

    if Os_Slab_Mem_Free(pool, ptr) {
        ret = LOS_OK;
    } 
    else {
        ret = Os_Mem_Free(pool, ptr);
    }

    Mem_Unlock!(int_save);

    Los_Trace!(MEM_FREE, pool, ptr as u32);
    ret
}