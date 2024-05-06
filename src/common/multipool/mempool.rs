use std::ptr;
include!("mempool_h.rs");

static mut G_POOL_HEAD: *mut std::ffi::c_void = ptr::null_mut();
//get pool size function by ourselves
unsafe fn los_mempoolsizeget(pool: *mut std::ffi::c_void ) -> u32 {
    let mut heapmanager: *mut LosMemPoolInfo = std::ptr::null_mut();
    if pool.is_null() {
        return OS_NULL_INT;
    }
    heapmanager = pool as *mut LosMemPoolInfo;
    (*heapmanager).pool_size
}

unsafe fn os_mem_mul_pool_init(pool: *mut std::ffi::c_void, size: u32) -> u32 {
    let mut next_pool = G_POOL_HEAD;
    let mut cur_pool = G_POOL_HEAD;
    while !next_pool.is_null() {
        let pool_end = next_pool.offset(unsafe{los_mempoolsizeget(next_pool)} as isize);
        if (pool <= next_pool && (pool.offset(size as isize) as usize) > next_pool as usize) || 
           ((pool as usize) < pool_end as usize && (pool.offset(size as isize) as usize) >= pool_end as usize) {
            println!("pool [{:?}, {:?}] conflict with pool [{:?}, {:?}]", pool, pool.offset(size as isize), next_pool, next_pool.offset(unsafe{los_mempoolsizeget(next_pool)} as isize));
            return LOS_NOK;
        }
        cur_pool = next_pool;
        next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool;
    }

    if G_POOL_HEAD.is_null() {
        G_POOL_HEAD = pool;
    } else {
        (*(cur_pool as *mut LosMemPoolInfo)).next_pool = pool;
    }

    (*(pool as *mut LosMemPoolInfo)).next_pool = std::ptr::null_mut();
    LOS_OK
}

unsafe fn os_mem_mul_pool_deinit(pool: *const std::ffi::c_void) -> u32 {
    let mut ret = LOS_NOK;
    let mut next_pool: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut cur_pool: *mut std::ffi::c_void = std::ptr::null_mut();

    if pool.is_null() {
        return ret;
    }

    if pool == G_POOL_HEAD {
        G_POOL_HEAD = (*(G_POOL_HEAD as *mut LosMemPoolInfo)).next_pool;
        return LOS_OK;
    }

    cur_pool = G_POOL_HEAD;
    next_pool = G_POOL_HEAD;
    while !next_pool.is_null() {
        if pool == next_pool {
            (*(cur_pool as *mut LosMemPoolInfo)).next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool;
            ret = LOS_OK;
            break;
        }
        cur_pool = next_pool;
        next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool;
    }

    ret
}

 fn os_mem_mul_pool_head_get() -> *mut std::ffi::c_void {
    unsafe { G_POOL_HEAD }
}

/*unsafe fn los_mem_de_init(pool: *mut std::ffi::c_void) -> u32 {
    let ret: u32;
    let int_save: u32;

    MEM_LOCK(int_save);
    ret = os_mem_mul_pool_deinit(pool);
    MEM_UNLOCK(int_save);

    ret
}*/

unsafe fn los_mem_pool_list() -> u32 {
    let mut next_pool = unsafe {G_POOL_HEAD};
    let mut index = 0;
    while !next_pool.is_null() {
        println!("pool{} :size--{}  starting address--{:p}", index, (*(next_pool as *mut LosMemPoolInfo)).pool_size, (*(next_pool as *mut LosMemPoolInfo)).pool);
        index += 1;
        //os_mem_info_print(next_pool);
        unsafe { next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool };
    }
    index
}
