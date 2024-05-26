include!("los_multipledlinkhead.rs");

fn Los_Mem_Realloc(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, size: u32) -> *mut std::ffi::c_void{
    let mut int_save: u32;
    let new_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut is_slab_mem: bool = false;
    let mut mem_free_value: u32;
    if Os_Mem_Node_Get_Used_Flag!(size) || Os_Mem_Node_Get_Aligned_Flag!(size) || pool == std::ptr::null_mut(){
        return std::ptr::null_mut();
    }
    if ptr == std::ptr::null_mut() {
        new_ptr = Los_Mem_Alloc(pool, size);
        return new_ptr;
    }
    if size == 0 {
        mem_free_value = Los_Mem_Free(pool, ptr);
        return new_ptr;
    }
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
