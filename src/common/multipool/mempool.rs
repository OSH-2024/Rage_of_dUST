
include!("mempool_h.rs");

static mut G_POOL_HEAD: *mut std::ffi::c_void = ptr::null_mut();
///los_memory
fn Los_Mem_Pool_Size_Get(pool: *mut std::ffi::c_void) -> u32 {
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }
    (*(pool as *mut LosMemPoolInfo)).pool_size
}

fn Os_Mem_Info_Get(pool: *mut std::ffi::c_void, pool_status: *mut LosMemPoolStatus) -> u32 {
    unsafe {
        let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
        let tmp_node: *mut LosMemDynNode = Os_Mem_End_Node!(pool, (*pool_info).pool_size);
        tmp_node = Os_Mem_Align!(tmp_node, Os_Mem_Align_Size!()) as *mut LosMemDynNode;

        if !Os_Mem_Magic_Valid!((*tmp_node).self_node.myunion.extend_field.magic.get()) {
            println!("wrong mem pool, {:?}, {:?}", pool_info, line!() as i32);
            return LOS_NOK;
        }

        let mut total_used_size = 0;
        let mut total_free_size = 0;
        let mut max_free_node_size = 0;
        let mut used_node_num = 0;
        let mut free_node_num = 0;

        tmp_node = Os_Mem_First_Node!(pool_info);

        while tmp_node <= Os_Mem_End_Node!(pool_info as *const _, (*pool_info).pool_size) {
            if !Os_Mem_Node_Get_Used_Flag!((*tmp_node).self_node.size_and_flag.get()) {
                free_node_num += 1;
                let node_size = Os_Mem_Node_Get_Size!((*tmp_node).self_node.size_and_flag.get());
                total_free_size += node_size;
                if max_free_node_size < node_size {
                    max_free_node_size = node_size;
                }
            } else {
                used_node_num += 1;
                total_used_size += Os_Mem_Node_Get_Size!((*tmp_node).self_node.size_and_flag.get());
            }

            tmp_node = Os_Mem_Next_Node!(tmp_node);
        }

        (*pool_status).uw_total_used_size = total_used_size.into();
        (*pool_status).uw_total_free_size = total_free_size.into();
        (*pool_status).uw_max_free_node_size = max_free_node_size.into();
        (*pool_status).uw_used_node_num = used_node_num.into();
        (*pool_status).uw_free_node_num = free_node_num.into();
        
        #[cfg(feature = "LOSCFG_MEM_TASK_STAT")]
        {
            (*pool_status).uw_usage_water_line = pool_info.stat.mem_total_peak;
        }

        LOS_OK
    }
}

fn Os_Mem_Info_Print(pool: *mut std::ffi::c_void)->(){
    unsafe{
        let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
        let status: *mut LosMemPoolStatus;

        if Os_Mem_Info_Get(pool, status) == LOS_NOK{
            return;
        }
        #[cfg(feature = "LOSCFG_MEM_TASK_STAT")]
        {
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
        #[cfg(not(feature = "LOSCFG_MEM_TASK_STAT"))]
        {
            println!(
                "pool addr          pool size    used size     free size    max free node size   used node num     free node num"
            );
            println!(
                "---------------    --------     -------       --------     --------------       -------------      ------------"
            );
            println!(
                "{:16p}   0x{:08x}   0x{:08x}    0x{:08x}   0x{:016x}   0x{:013x}    0x{:013x}",
                (*pool_info).pool,
                (*pool_info).pool_size,
                (*status).uw_total_used_size.get(),
                (*status).uw_total_free_size.get(),
                (*status).uw_max_free_node_size.get(),
                (*status).uw_used_node_num.get(),
                (*status).uw_free_node_num.get()
            );
        }
    }
}
///
fn Os_Mem_Mul_Pool_Init(pool: *mut std::ffi::c_void, size: u32) -> u32 {
    let mut next_pool = G_POOL_HEAD;
    let mut cur_pool = G_POOL_HEAD;
    while !next_pool.is_null() {
        let pool_end = next_pool.offset(unsafe{Los_Mem_Pool_Size_Get(next_pool)} as isize);
        if (pool <= next_pool && (pool.offset(size as isize) as usize) > next_pool as usize) || 
           ((pool as usize) < pool_end as usize && (pool.offset(size as isize) as usize) >= pool_end as usize) {
            println!("pool [{:?}, {:?}] conflict with pool [{:?}, {:?}]", pool, pool.offset(size as isize), next_pool, next_pool.offset(unsafe{Los_Mem_Pool_Size_Get(next_pool)} as isize));
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

fn Os_Mem_Mul_Pool_Deinit(pool: *const std::ffi::c_void) -> u32 {
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

fn Os_Mem_Mul_Pool_Head_Get() -> *mut std::ffi::c_void {
    unsafe { G_POOL_HEAD }
}

fn Los_Mem_Deinit(pool: *mut std::ffi::c_void) -> u32 {
    let mut ret: u32;
    let mut int_save: u32;
    Mem_Lock!(int_save);
    ret = Os_Mem_Mul_Pool_Deinit(pool);
    Mem_Unlock!(int_save);
    ret
}

fn Los_Mem_Pool_List() -> u32 {
    let mut next_pool = unsafe {G_POOL_HEAD};
    let mut index = 0;
    while !next_pool.is_null() {
        println!("pool{} :size--{}  starting address--{:p}", index, (*(next_pool as *mut LosMemPoolInfo)).pool_size, (*(next_pool as *mut LosMemPoolInfo)).pool);
        index += 1;
        /*********/
        Os_Mem_Info_Print(next_pool);
        /*********/
        unsafe { next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool };
    }
    index
}
