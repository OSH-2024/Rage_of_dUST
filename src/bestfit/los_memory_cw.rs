include!("los_memory_h.rs");
/*********/
//以下带有/*********/的函数为未实现的函数
/*********/

fn Los_Mem_Realloc(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, size: u32) -> *mut std::ffi::c_void{
    let mut int_save: u32;
    let new_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut is_slab_mem: bool = false;
    let mut mem_free_value: u32;
    if Os_Mem_Node_Get_Used_Flag!(size) || Os_Mem_Node_Get_Aligned_Flag!(size) || pool == std::ptr::null_mut(){
        return std::ptr::null_mut();
    }
    if ptr == std::ptr::null_mut() {
        /*********/
        new_ptr = Los_Mem_Alloc(pool, size);
        /*********/
        //Los_Trace!();
        return new_ptr;
    }
    if size == 0 {
        /*********/
        mem_free_value = Los_Mem_Free(pool, ptr);
        /*********/
        //Los_Trace!();
        return new_ptr;
    }
    
    Mem_Lock!(int_save);
    /*********/
    new_ptr = Os_Mem_Realloc_Slab(pool, ptr, &mut is_slab_mem, size);
    /*********/

    if is_slab_mem == true {
        Mem_Unlock!(int_save);
        //Los_Trace!();
        return new_ptr;
    }
    /*********/
    new_ptr = Os_Mem_Realloc(pool, ptr, size);
    /*********/
    Mem_Unlock!(int_save);
    //Los_Trace!();

    new_ptr

}

fn Los_Mem_Total_Used_Get(pool: *mut std::ffi::c_void) -> u32{
    let mut tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut mem_used: u32 = 0;
    let mut int_save: u32;
    if pool == std::ptr::null_mut(){
        return LOS_NOK;
    }

    Mem_Lock!(int_save);
    //
    let mut tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, pool_info.pool_size) {
    // 在这里处理 tmp_node 指向的节点
        if Os_Mem_Node_Get_Used_Flag!(tmp_node.self_node.size_and_flag) {
            mem_used += Os_Mem_Node_Get_Size!(tmp_node.self_node.size_and_flag);
        }
    // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);

    mem_used
}

fn Los_Mem_Used_Blks_Get(pool: *mut std::ffi::c_void) -> u32{
    let mut tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut blknums: u32 = 0;
    let mut int_save: u32;
    if pool == std::ptr::null_mut(){
        return LOS_NOK;
    }

    Mem_Lock!(int_save);
    //
    let mut tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, pool_info.pool_size) {
    // 在这里处理 tmp_node 指向的节点
        if Os_Mem_Node_Get_Used_Flag!(tmp_node.self_node.size_and_flag) {
            blknums++;
        }
    // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);

    blknums
}

fn Los_Mem_Task_Id_Get(ptr: *mut std::ffi::c_void) -> u32{
    let tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    //m_auc_sys_mem1: UINT8 *
    let pool_info: *mut LosMemPoolInfo = (m_auc_sys_mem1 as *mut std::ffi::c_void) as *mut LosMemPoolInfo;
    let mut int_save: u32;
    //LOSCFG_EXC_INTERACTION
    if ptr < m_auc_sys_mem1 as *mut std::ffi::c_void {
        pool_info = (m_auc_sys_mem0 as *mut std::ffi::c_void) as *mut LosMemPoolInfo
    }
    //

    if ((ptr == std::ptr::null_mut()) || 
        (ptr < Os_Mem_First_Node!(pool_info) as *mut std::ffi::c_void) ||
        (ptr > Os_Mem_End_Node!(pool_info, pool_info.pool_size) as *mut std::ffi::c_void)){
        println!("input ptr {:p} is out of system memory range[{:p}, {:p}]\n", ptr, Os_Mem_First_Node!(pool_info), 
                    Os_Mem_End_Node!(pool_info, pool_info.pool_size));
        return OS_INVALID;
        //(UINT32)(-1)
    }

    Mem_Lock!(int_save);

    let mut tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, pool_info.pool_size) {
    // 在这里处理 tmp_node 指向的节点
        if ptr as u32 < tmp_node as u32 {
            if Os_Mem_Node_Get_Used_Flag!(tmp_node.self_node.prenode.self_node.size_and_flag.get()) {
                Mem_Unlock!(int_save);
                return tmp_node.self_node.prenode.self_node.myunion.extend_field.taskid.get();
            }
            else {
                Mem_Unlock!(int_save);
                println!("input ptr {:p} is belong to a free mem node\n", ptr);
                return OS_INVALID;
            }
        }
    // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);
    OS_INVALID

}

fn Los_Mem_Free_Blks_Get(pool: *mut std::ffi::c_void) -> u32{
    let tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut blknums: u32 = 0;
    let mut int_save: u32;
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }

    Mem_Lock!(int_save);
    //
    let mut tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, pool_info.pool_size) {
    // 在这里处理 tmp_node 指向的节点
        if Os_Mem_Node_Get_Used_Flag!(tmp_node.self_node.size_and_flag.get()) == 0 {
            blknums++;
        }
    // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);

    blknums
}

fn Los_Mem_Last_Used_Get(pool: *mut std::ffi::c_void) -> u32{
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let node: *mut LosMemPoolInfo = std::ptr::null_mut();
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }
    node = (*(Os_Mem_End_Node!(pool, (*pool_info).pool_size))).self_node.prenode;
    if Os_Mem_Node_Get_Used_Flag!((*node).self_node.size_and_flag.get()) {
        return ((node as *mut char + Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get()) + std::mem::size_of::<LosMemDynNode>()) as u32);
    }
    else {
        return ((node as *mut char + std::mem::size_of::<LosMemDynNode>()) as u32);
    }
}

fn Os_Mem_Reset_End_Node(pool: *mut std::ffi::c_void, pre_addr: u32) ->(){
    let end_node: *mut LosMemDynNode = (Os_Mem_End_Node!(pool, (*(pool as *mut LosMemPoolInfo)).pool_size)) as *mut LosMemDynNode;
    (*end_node).self_node.size_and_flag.set(Os_Mem_Node_Head_Size!());
    if pre_addr != 0 {
        (*end_node).self_node.prenode = (pre_addr - std::mem::size_of::<LosMemDynNode>()) as *mut LosMemDynNode;
    }

    Os_Mem_Node_Set_Used_Flag!((*end_node).self_node.size_and_flag.get());
    /*********/
    Os_Mem_Set_Magic_Num_And_Task_ID(end_node);
    Os_Mem_Node_Save(end_node);
    /*********/
}

fn Los_Mem_Pool_Size_Get(pool: *mut std::ffi::c_void) -> u32{
    if pool == std::ptr::null_mut(){
        return LOS_NOK;
    }
    (*(pool as *mut LosMemPoolInfo)).pool_size
}

fn Los_Mem_Info_Get(pool: *mut std::ffi::c_void, pool_status: *mut LosMemPoolStatus) -> u32{
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut ret: u32;
    let mut int_save: u32;
    if pool_status == std::ptr::null_mut() {
        println!("can't use NULL addr to save info\n");
        return LOS_NOK;
    }
    if (pool_info == std::ptr::null_mut()) || (pool as u32 != ((*pool_info).pool) as u32) {
        println!("wrong mem pool addr: {}, line:{}\n", pool_info, line!());
        return LOS_NOK;
    }
    Mem_Lock!(int_save);
    /*********/
    ret = Os_Mem_Info_Get(pool_info, pool_status);
    /*********/
    Mem_Unlock!(int_save);

    ret
}

fn Os_Show_Free_Node(index: u32, length: u32, count_num: *u32) ->(){
    let mut count: u32 = 0;
    println!("\n    block size:  ");
    for count in 0..= length-1 {
        println!("2^{:<5}", (index + Os_Min_Multi_Dlnk_Log2!() + count));
    }
    println!("\n    node number: ");
    count = 0;
    for count in 0..= length-1 {
        println!("  {:<5}", count_num[count + index]);
    }
}

fn Los_Mem_Free_Node_Show(pool: *mut std::ffi::c_void) -> u32{
    let list_node_head: *mut LosDlList = std::ptr::null_mut();
    let head_addr: *mut LosMultipleDlinkHead = (pool as u32 + std::mem::size_of::<LosMemPoolInfo>()) as *mut LosMultipleDlinkHead;
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut link_head_index: u32;
    let mut count_num: [u32; Os_Multi_Dlnk_Num!()] = [0; Os_Multi_Dlnk_Num!()];
    let mut int_save: u32;

    if (pool == std::ptr::null_mut()) || (pool as u32 != (pool_info.pool) as u32) {
        println!("wrong mem pool addr: {:p}, line:{}\n", pool_info, line!());
        return LOS_NOK;
    }

    println!("\n   ************************ left free node number**********************");
    Mem_Lock!(int_save);

    for link_head_index in 0 ..= Os_Multi_Dlnk_Num!() - 1 {
        list_node_head = head_addr.list_head[link_head_index].pst_next;
        while list_node_head != &mut (head_addr.list_head[link_head_index]) {
            list_node_head = list_node_head.pst_next;
            count_num[link_head_index]++;
        }
    }

    link_head_index = 0;
    while link_head_index < Os_Multi_Dlnk_Num {
        if link_head_index + Column!() < Os_Multi_Dlnk_Num!() {
            //Column!()  未定义 8
            /*********/
            Os_Show_Free_Node(link_head_index, Column!(), count_num);
            /*********/
            link_head_index += Column!();
        }
        else {
            /*********/
            Os_Show_Free_Node(link_head_index, (Os_Multi_Dlnk_Num!() - 1 - link_head_index), count_num);
            /*********/
            break;
        }
    }

    Mem_Unlock!(int_save);
    println!("\n   ********************************************************************\n\n");

    LOS_OK
}

//LOSCFG_BASE_MEM_NODE_SIZE_CHECK
fn Los_Mem_Node_Size_Check(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, total_size: *mut u32, avail_size: *mut u32) -> u32 {
    let head: *mut std::ffi::c_void = std::ptr::null_mut();

    
}

fn Os_Mem_Find_Node_Ctrl(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    let head: *mut std::ffi::c_void = ptr;

    if ptr == std::ptr::null_mut() {
        return std::ptr::null_mut();
    }

    head = Os_Mem_Align(head, Os_Mem_Align_Size!());
    while !Os_Mem_Magic_Valid!((head as *mut LosMemDynNode).self_node.myunion.extend_field.magic.get()) {
        head = (head as *mut u8 - std::mem::size_of()<*mut char>) as *mut std::ffi::c_void;
        if head <= pool {
            return std::ptr::null_mut();
        }
    }

    head
}

fn Los_Mem_Check_Level_Set(check_level: u8) -> u32{
    //low 0
    if check_level == Los_Mem_Check_Level_Low!() {
        println!("{:s}: LOS_MEM_CHECK_LEVEL_LOW \n", __FUNCTION__);
    }
    //high 1
    else if check_level == Los_Mem_Check_Level_High!() {
        println!("{:s}: LOS_MEM_CHECK_LEVEL_HIGH \n", __FUNCTION__);
    }
    else if check_level == Los_Mem_Check_Level_Disable!() {
        println!("{:s}: LOS_MEM_CHECK_LEVEL_DISABLE \n", __FUNCTION__);
    }
    else {
        println!("{:s}: wrong param, setting failed !! \n", __FUNCTION__);
        return Los_Errno_Memcheck_Wrong_Level!();
        /////
    }
    g_mem_check_level = check_level;

    LOS_OK

}
fn Los_Mem_Check_Level_Get() -> u8{
    g_mem_check_level
}

fn Os_Mem_Sys_Node_Check(dst_addr: *mut std::ffi::c_void, src_addr: *mut std::ffi::c_void, node_length: u32, pos: u8)->u32{
    let mut ret: u32;
    let mut total_size: u32 = 0;
    let mut avail_size; u32 = 0;
    let pool: *mut u8 = m_auc_sys_mem1;
    //LOSCFG_EXC_INTERACTION
    if dst_addr as u32 < m_auc_sys_mem0 as u32 + g_excinteract_memsize {
        pool = m_auc_sys_mem0;
    }
    //
    ret = Los_Mem_Node_Size_Check(pool, dst_addr, &mut total_size, &mut avail_size);
    if (ret == LOS_OK) && (node_length > avail_size) {
        println!("---------------------------------------------\n"
                "{:s}: dst inode availSize is not enough availSize = 0x{:x}, memcpy length = 0x{:x}\n",
                ((pos == 0) ? "memset" : "memcpy"), avail_size, node_length);
        //Os_Back_Trace();
        println!("---------------------------------------------\n");
        return LOS_NOK;
    }

    if pos == -1 {
        //LOSCFG_EXC_INTERACTION
        if src_addr as u32 < m_auc_sys_mem0 as u32 + g_excinteract_memsize {
            pool = m_auc_sys_mem0;
        }
        else {
            pool = m_auc_sys_mem1;
        }
        //
        ret = Los_Mem_Node_Size_Check(pool, src_addr, &mut total_size, &mut avail_size);
        if ((ret == LOS_OK) && (node_length > avail_size)) {
            println!("---------------------------------------------\n");
            println!("memcpy: src inode availSize is not enough"
                      " availSize = 0x{:x}, memcpy length = 0x{:x}\n",
                      avail_size, node_length);
            //OsBackTrace();
            println!("---------------------------------------------\n");
            return LOS_NOK;
        }
    }

    LOS_OK
}

