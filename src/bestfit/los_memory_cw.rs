include!("los_memory_h.rs");


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


    blknums
}

fn Los_Mem_Task_Id_Get(ptr: *mut std::ffi::c_void) -> u32{

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
    for 
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
        return ((node as *mut char + Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get()) + std::mem::size_of()<LosMemDynNode>) as u32);
    }
    else {
        return ((node as *mut char + std::mem::size_of()<LosMemDynNode>) as u32);
    }
}

fn Os_Mem_Reset_End_Node(pool: *mut std::ffi::c_void, pre_addr: u32) ->(){
    let end_node: *mut LosMemDynNode = (Os_Mem_End_Node!(pool, (*(pool as *mut LosMemPoolInfo)).pool_size)) as *mut LosMemDynNode;
    (*end_node).self_node.size_and_flag.set(Os_Mem_Node_Head_Size!());
    if pre_addr != 0 {
        (*end_node).self_node.prenode = (pre_addr - std::mem::size_of()<LosMemDynNode>) as *mut LosMemDynNode;
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