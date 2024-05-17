use core::str;

//重写C头文件中宏定义
macro_rules! LITE_OS_TEXT_MINOR{() => {}}


macro_rules! MIN_TASK_ID{
    ($x:expr, $y:expr) => {
        {if $x > $y {$y} else {$x}}
    }
}
macro_rules! MAX_MEM_USE{
    ($x:expr, $y:expr) => {
        {if $x > $y {$x} else {$y}}
    }
}

pub const LOSCFG_BASE_CORE_TSK_LIMIT: u32 = 64;
pub const TASK_NUM: u32 = LOSCFG_BASE_CORE_TSK_LIMIT + 1;
//用到的结构体定义


struct TaskMemUsedinfo {
    memUsed: u32,
    memPeak: u32
}
struct Memstat{
    memTotalUsed: u32,
    memTotalPeak: u32,
    taskMemstats: [TaskMemUsedinfo; TASK_NUM()]
}


LITE_OS_TEXT_MINOR!{} fn OSMemstatTaskUsedInc(mut stat: Memstat, usedSize: u32, taskId: u32){
    let record :u32 = MIN_TASK_ID!(taskId, TASK_NUM - 1); 
    let mut taskMemstats = &stat.taskMemstats;

    taskMemstats[record].memUsed += usedSize;
    taskMemstats[record].memPeak = MAX_MEM_USE!(taskMemstats[record].memPeak, taskMemstats[record].memUsed);

    stat.memTotalUsed += usedSize;
    stat.memTotalPeak = MAX_MEM_USE!(stat.memTotalPeak, stat.memTotalUsed); 
}

LITE_OS_TEXT_MINOR!{} fn OsMemstatTaskUsedDec(mut stat: Memstat, usedSize: u32, taskId: u32){
    let record :u32 = MIN_TASK_ID!(taskId, TASK_NUM - 1); 
    let mut taskMemstats = &stat.taskMemstats;

    if taskMemstats[record as usize].memUsed < usedSize{
        print!("mem used of current task '{}': 0x{}, decrease size: 0x{}\n", todo!());
    }

    taskMemstats[record as usize].memUsed = 0;
    taskMemstats[record as usize].memPeak = 0;
}

LITE_OS_TEXT_MINOR!{} fn OSMemstatTaskClear(mut stat: &Memstat, taskId: u32){
    let record :u32 = MIN_TASK_ID!(taskId, TASK_NUM - 1); 
    let mut taskMemstats = &stat.taskMemstats;

    if taskMemstats[record].memUsed != 0{
        println!("mem used of task '{}' is 0x{}, not zero when task being deleted\n", todo!());
    }

    taskMemstats[record].memUsed = 0;
    taskMemstats[record].memPeak = 0;
}

LITE_OS_TEXT_MINOR!{} fn OsMemstatTaskUsage(stat: &Memstat, taskId: u32)->u32{
    let record : u32 = MIN_TASK_ID!(taskId, TASK_NUM-1);
    let taskMemstats = stat.taskMemstats;

    return taskMemstats[record as usize].memUsed;
}

fn OsMemTaskUsage(taskId: u32)->u32{
    let mut pool = todo!();
    let mut stat : &Memstat;

    if cfg!(LOSCFG_MEM_MUL_POOL){
        /* If Multi-pool is not enabled, then trace SYSTEM MEM only */
        let pool = todo!();
        stat = &pool.stat;
        return OsMemstatTaskUsage(stat, taskId);
    }
    else{
        let inUse:u32 = 0;
        pool = todo!();
        while !pool.is_null() {
            stat = &(pool.stat);
            inUse += OsMemstatTaskUsage(stat, taskId);
            pool = pool.nextpool;
        }
        return inUse;
    }
}

fn OsMemTaskClear(taskId: u32){
    let mut pool = todo!();
    let mut stat: &Memstat;

    if cfg!(LOSCFG_MEM_MUL_POOL){
        pool = todo!();
        stat = &(pool.stat);
        OSMemstatTaskClear(stat, taskId);
    }
    else{
        pool = todo!();
        while !pool.is_null() {
            stat = &(pool.stat);
            OSMemstatTaskClear(stat, taskId);
            pool = pool.nextPool;
        }
    }
}