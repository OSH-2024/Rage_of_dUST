include!("los_multipledlinkhead.rs");

struct LosMemPoolStatus {
    uw_total_used_size: Cell<u32>,
    uw_total_free_size: Cell<u32>,
    uw_max_free_node_size: Cell<u32>,
    uw_used_node_num: Cell<u32>,
    uw_free_node_num: Cell<u32>,
    uw_usage_waterline: Cell<u32>,
} //LOS_MEM_POOL_STATUS

pub const OS_INVALID: u32 = u32::MAX;

enum LosMoudleId {
    LosModSys = 0x0,
    /**< System ID. Its value is 0x0. */
    LosModMem = 0x1,
    /**< Dynamic memory module ID. Its value is 0x1. */
    LosModTsk = 0x2,
    /**< Task module ID. Its value is 0x2. */
    LosModSwtmr = 0x3,
    /**< Software timer module ID. Its value is 0x3. */
    LosModTick = 0x4,
    /**< Tick module ID. Its value is 0x4. */
    LosModMsg = 0x5,
    /**< Message module ID. Its value is 0x5. */
    LosModQue = 0x6,
    /**< Queue module ID. Its value is 0x6. */
    LosModSem = 0x7,
    /**< Semaphore module ID. Its value is 0x7. */
    LosModMbox = 0x8,
    /**< Static memory module ID. Its value is 0x8. */
    LosModMmu = 0xd,
    /**< MMU module ID. Its value is 0xd. */
    LosModLog = 0xe,
    /**< Log module ID. Its value is 0xe. */
    LosModErr = 0xf,
    /**< Error handling module ID. Its value is 0xf. */
    LosModExc = 0x10,
    /**< Exception handling module ID. Its value is 0x10. */
    LosModButt, /* *< It is end flag of this enumeration. */
}

//type TSK_ENTRY_FUNC = fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void;
/*
struct SortLinkList {
    sort_link_node: LOS_DL_LIST,
    idx_roll_num: u32,
}

struct tagEvent {
    uw_event_id: u32,
    st_event_list: LOS_DL_LIST,
}

type EVENT_CB_S = tagEvent;
type PEVENT_CB_S = *mut tagEvent;

struct HeldLocks {
    lock_ptr: *mut std::ffi::c_void,
    lock_addr: *mut std::ffi::c_void,
    wait_time: u64,
    hold_time: u64,
}

struct LockDep {
    wait_lock: *mut std::ffi::c_void,
    lock_depth: i32,
    held_locks: [HeldLocks; 16], //MAX_LOCK_DEPTH=16U
}

struct SchePercpu {
    run_time: u64,
    contex_switch: u32,
}

struct SchedStat {
    start_runtime: u64,
    all_runtime: u64,
    all_context_switch: u32,
    sched_percpu: SchePercpu, //假定LOSCFG_KERNEL_CORE_NUM=1
}*/

struct LosTaskCB {
    stack_pointer: *mut std::ffi::c_void,
    task_status: u16,
    priority: u16,
    task_flags: u32, //原代码为位域31
    usr_stack: u32,  //原代码为位域1
    stack_size: u32,
    top_of_stack: u32,
    task_id: u32,
    //task_entry: TSK_ENTRY_FUNC,
    task_sem: *mut std::ffi::c_void,
    task_name: *mut char,
    /*// [#(cfg(feature = "LOSCFG_COMPAT_POSIX"))]
    thread_join: *mut std::ffi::c_void,
    // [#(cfg(feature = "LOSCFG_COMPAT_POSIX"))]
    thread_join_retval: *mut std::ffi::c_void,

    task_mux: *mut std::ffi::c_void,

    // [#(cfg(feature = "LOSCFG_OBSOLETE_API"))]
    // args:[u32;4],
    // [#(cfg(not(feature = "LOSCFG_OBSOLETE_API")))]
    args: *mut std::ffi::c_void,
    pend_list: LOS_DL_LIST,
    sort_list: SortLinkList,*/

    /*// [#(cfg(feature = "LOSCFG_BASE_IPC_EVENT"))]
    event: EVENT_CB_S,
    // [#(cfg(feature = "LOSCFG_BASE_IPC_EVENT"))]
    event_mask: u32,
    // [#(cfg(feature = "LOSCFG_BASE_IPC_EVENT"))]
    event_mode: u32,

    msg: *mut std::ffi::c_void,
    pri_bit_map: u32,
    signal: u32,*/

    /*// [#(cfg(feature = "LOSCFG_BASE_CORE_TIMESLICE"))]
    time_slice: u16,

    // [#(cfg(feature = "LOSCFG_KERNEL_SMP"))]
    curr_cpu: u16,
    // [#(cfg(feature = "LOSCFG_KERNEL_SMP"))]
    last_cpu: u16,
    // [#(cfg(feature = "LOSCFG_KERNEL_SMP"))]
    timer_cpu: u32,
    // [#(cfg(feature = "LOSCFG_KERNEL_SMP"))]
    cpu_affi_mask: u32,

    // [#(cfg(feature = "LOSCFG_KERNEL_SMP_TASK_SYNC"))]
    sync_signal: u32,

    // [#(cfg(feature = "LOSCFG_KERNEL_SMP_LOCKDEP"))]
    lock_dep: lockDep,

    // [#(cfg(feature = "LOSCFG_DEBUG_SCHED_STATISTICS"))]
    sched_stat: SchedStat,

    // [#(cfg(feature = "LOSCFG_KERNEL_PERF"))]
    pc: u32,
    // [#(cfg(feature = "LOSCFG_KERNEL_PERF"))]
    fp: u32,*/
}

macro_rules! Los_Mem_Check_Level_High {
    {} => {
        1
    };
}

macro_rules! Los_Mem_Check_Level_Low {
    {} => {
        0
    };
}

macro_rules! Los_Mem_Check_Level_Disable {
    {} => {
        0xff
    };
}

macro_rules! Los_Errtype_Error {
    () => {
        (0x02 << 24)
    };
}

macro_rules! Los_Errno_Os_Id {
    () => {
        (0x00 << 16)
    };
}

macro_rules! Los_Errno_Os_Error {
    ($mid: expr, $errno: expr) => {
        (Los_Errtype_Error!() | Los_Errno_Os_Id!() | (($mid as u32) << 8) | $errno as u32)
    };
}

macro_rules! Los_Errno_Memcheck_Wrong_Level {
    {} => {
        Los_Errno_Os_Error!(LosModMem, 0x4)
    };
}

macro_rules! Los_Errno_Memcheck_Disabled {
    {} => {
        Los_Errno_Os_Error!(LosModMem, 0x5)
    }
}

macro_rules! Los_Errno_Memcheck_Para_Null {
    {} => {
        Los_Errno_Os_Error!(LosModMem, 0x1)
    }
}

macro_rules! Los_Errno_Memcheck_Outside {
    {} => {
        Los_Errno_Os_Error!(LosModMem, 0x2)
    }
}

macro_rules! Los_Errno_Memcheck_No_Head {
    {} => {
        Los_Errno_Os_Error!(LosModMem, 0x3)
    }
}

macro_rules! Mem_Module_Max {
    {} => {
        0x20
    }
}

macro_rules! Os_Null_Int {
    {} => {
        0xFFFFFFFF as u32
    }
}

//los_memory.c 56
macro_rules! Node_Dump_Size {
    () => {
        64
    };
}

//los_memory.c 57
macro_rules! Column_Num {
    () => {
        8
    };
}
/*
macro_rules! Los_Off_Set_Of {
    ($typ: ty, $member: pat) => {
        (&mut ((*(0 as *mut $typ)).$member) ) as u32
    };
}

macro_rules! Los_Dl_List_Entry {
    ($item: expr, $typ: ty, $member: pat) => {
        (($item as *mut char).offset(-Los_Off_Set_Of!($typ, $member) as isize)) as *mut std::ffi::c_void as *mut $typ
    };
}*/

