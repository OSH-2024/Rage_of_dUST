use core::str;
use memstat_h;

fn Os_Memstat_Task_Used_Inc(stat: &mut Memstat, used_size: u32, task_id: u32) {
    let record = MIN_TASK_ID!(task_id, TASK_NUM - 1);
    let task_memstats = &mut stat.task_memstats;

    task_memstats[record as usize].mem_used += used_size;
    task_memstats[record as usize].mem_peak = MAX_MEM_USE!(task_memstats[record as usize].mem_peak, task_memstats[record as usize].mem_used);

    stat.mem_total_used += used_size;
    stat.mem_total_peak = MAX_MEM_USE!(stat.mem_total_peak, stat.mem_total_used);
}

fn Os_Memstat_Task_Used_Dec(stat: &mut Memstat, used_size: u32, task_id: u32) {
    let record = MIN_TASK_ID!(task_id, TASK_NUM - 1);
    let task_memstats = &mut stat.task_memstats;

    if task_memstats[record as usize].mem_used < used_size {
        println!("mem used of current task '{}':0x{:x}, decrease size:0x{:x}\n",
                 Os_Curr_Task_Get().task_name, task_memstats[record as usize].mem_used, used_size);
        return;
    }

    task_memstats[record as usize].mem_used -= used_size;
    stat.mem_total_used -= used_size;
}

fn Os_Memstat_Task_Clear(stat: &mut Memstat, task_id: u32) {
    let record = MIN_TASK_ID!(task_id, TASK_NUM - 1);
    let task_memstats = &mut stat.task_memstats;

    if task_memstats[record as usize].mem_used != 0 {
        println!("mem used of task '{}' is:0x{:x}, not zero when task being deleted\n",
                 Os_Curr_Task_Get().task_name, task_memstats[record as usize].mem_used);
    }

    task_memstats[record as usize].mem_used = 0;
    task_memstats[record as usize].mem_peak = 0;
}

fn OsMemstatTaskUsage(stat: &Memstat, taskId: u32)->u32{
    let record : u32 = MIN_TASK_ID!(taskId, TASK_NUM-1);
    let task_Memstats = &mut stat.taskMemstats;

    return task_Memstats[record as usize].memUsed;
}

fn Os_Mem_Task_Usage(task_id: u32) -> u32 {
    let mut pool: *mut LosMemPoolInfo = std::ptr::null_mut();
    let mut stat: *mut Memstat = std::ptr::null_mut();

    #[cfg(not(feature = "LOSCFG_MEM_MUL_POOL"))]
    {
        /* If Multi-pool is not enabled, then trace SYSTEM MEM only */
        pool = OS_SYS_MEM_ADDR as *mut LosMemPoolInfo;
        stat = &mut (*pool).stat;
        return Os_Memstat_Task_Usage(stat, task_id);
    }

    #[cfg(feature = "LOSCFG_MEM_MUL_POOL")]
    {
        let mut in_use = 0;
        pool = Os_Mem_Mul_Pool_Head_Get();
        while !pool.is_null() {
            stat = &mut (*pool).stat;
            in_use += Os_Memstat_Task_Usage(stat, task_id);
            pool = (*pool).next_pool;
        }
        return in_use;
    }
}

fn Os_Mem_Task_Clear(taskId: u32){
    let mut pool: *mut LosMemPoolInfo = std::ptr::null_mut();
    let mut stat: *mut Memstat = std::ptr::null_mut();

    #[cfg(not(feature = "LOSCFG_MEM_MUL_POOL"))]{
        pool = OS_SYS_MEM_ADDR as *mut LosMemPoolInfo;
        stat = &mut (*pool).stat;
        Os_Mem_Stat_Task_Clear(stat, taskId);
    }

    #[cfg(feature = "LOSCFG_MEM_MUL_POOL")]{
        pool = Os_Mem_Mul_Pool_Head_Get();
        while !pool.is_null() {
            stat = &mut (*pool).stat;
            Os_Mem_Stat_Task_Clear(stat, taskId);
            pool = (*pool).next_pool;
        }
    }
}