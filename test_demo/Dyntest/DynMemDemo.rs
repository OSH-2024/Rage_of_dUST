include!("DynMemDemo_h.rs");

pub fn Los_Mem_Init(pool: *mut c_void, mut size: u32) -> u32 {
    unsafe{
        //需要初始化
        println!("pool:{:p}, size:{:x}", pool, size);
        let mut int_save: u32 = 0;
        if pool.is_null() || size < Os_Mem_Min_Pool_Size!() {
            println!("Os_Mem_Mul_Pool_Init failed due to invalid input\n");
            return LOS_NOK;
        }
        
        if !Is_Aligned!(size, Os_Mem_Align_Size!()) || !Is_Aligned!(pool as u32, Os_Mem_Align_Size!()) {
            println!("pool [{:?}, {:?}) size 0x{:x} should be aligned with OS_MEM_ALIGN_SIZE\n",
                     pool, unsafe { pool.offset(size as isize) }, size);
            size = Os_Mem_Align!(size as usize, Os_Mem_Align_Size!()) as u32 - Os_Mem_Align_Size!() as u32;
        }
    
        Mem_Lock(int_save);
        if Os_Mem_Mul_Pool_Init(pool, size) != LOS_OK {
            println!("Os_Mem_Mul_Pool_Init failed\n");
            Mem_Unlock(int_save);
            return LOS_NOK;
        }
    
        if Os_Mem_Init(pool, size) != LOS_OK {
            println!("Os_Mem_Init failed\n");
            Os_Mem_Mul_Pool_Deinit(pool);
            Mem_Unlock(int_save);
            return LOS_NOK;
        }
        Mem_Unlock(int_save);
        LOS_OK
    }

}

pub fn Los_Mem_Alloc(pool: *mut std::ffi::c_void, size: u32) -> *mut std::ffi::c_void {
    let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut int_save: u32 = 0;

    if pool.is_null() || size == 0 {
        return std::ptr::null_mut();
    }

    Mem_Lock(int_save);
    loop {
        if Os_Mem_Node_Get_Used_Flag!(size) || Os_Mem_Node_Get_Aligned_Flag!(size) {
            break;
        }
        ptr = std::ptr::null_mut();
        if ptr.is_null() {
            ptr = Os_Mem_Alloc_With_Check(pool as *mut LosMemPoolInfo, size);
        }
        break;
    }

    Mem_Unlock(int_save);

    //Los_Trace!(Mem_Alloc, pool, ptr as u32, size);
    ptr
}

pub fn Los_Mem_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void) -> u32 {
    let mut ret: u32 = LOS_OK;
    let mut int_save: u32 = 0;
    if pool.is_null() || ptr.is_null() {
        println!("p;");
        return LOS_NOK;
    }

    Mem_Lock(int_save);

    ret = Os_Mem_Free(pool, ptr);
    

    Mem_Unlock(int_save);

    //Los_Trace!(MEM_FREE, pool, ptr as u32);
    ret
}
