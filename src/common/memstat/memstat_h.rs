pub const LOSCFG_BASE_CORE_TSK_LIMIT: u32 = 64;
pub const TASK_NUM: u32 = LOSCFG_BASE_CORE_TSK_LIMIT + 1;

extern "C" {
    static __heap_start: u8;
}

let OS_SYS_MEM_ADDR = unsafe { &__heap_start as *const _ as usize };

//重写C头文件中宏定义
macro_rules! LITE_OS_TEXT_MINOR{() => {}}


macro_rules! MIN_TASK_ID{
    ($x:expr, $y:expr) => {
        {if $x > $y {$y} else {$x}}
    }
}memstat
macro_rules! MAX_MEM_USE{
    ($x:expr, $y:expr) => {
        {if $x > $y {$x} else {$y}}
    }
}

struct TaskMemUsedinfo {
    memUsed: u32,
    memPeak: u32
}
struct Memstat{
    memTotalUsed: u32,
    memTotalPeak: u32,
    taskMemstats: [TaskMemUsedinfo; TASK_NUM]
}