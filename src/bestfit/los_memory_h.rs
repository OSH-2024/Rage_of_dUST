include!("los_multipledlinkhead.rs");

struct LosMemPoolStatus{
    uw_total_used_size: Cell<u32>,
    uw_total_free_size: Cell<u32>,
    uw_max_free_node_size: Cell<u32>,
    uw_used_node_num: Cell<u32>,
    uw_free_node_num: Cell<u32>,
    uw_usage_waterline: Cell<u32>

}//LOS_MEM_POOL_STATUS

pub const OS_INVALID: u32 = u32::MAX;

enum LosMoudleId {
    LosModSys = 0x0,          /**< System ID. Its value is 0x0. */
    LosModMem = 0x1,          /**< Dynamic memory module ID. Its value is 0x1. */
    LosModTsk = 0x2,          /**< Task module ID. Its value is 0x2. */
    LosModSwtmr = 0x3,        /**< Software timer module ID. Its value is 0x3. */
    LosModTick = 0x4,         /**< Tick module ID. Its value is 0x4. */
    LosModMsg = 0x5,          /**< Message module ID. Its value is 0x5. */
    LosModQue = 0x6,          /**< Queue module ID. Its value is 0x6. */
    LosModSem = 0x7,          /**< Semaphore module ID. Its value is 0x7. */
    LosModMbox = 0x8,         /**< Static memory module ID. Its value is 0x8. */
    LosModMmu = 0xd,          /**< MMU module ID. Its value is 0xd. */
    LosModLog = 0xe,          /**< Log module ID. Its value is 0xe. */
    LosModErr = 0xf,          /**< Error handling module ID. Its value is 0xf. */
    LosModExc = 0x10,         /**< Exception handling module ID. Its value is 0x10. */
    LosModButt                /* *< It is end flag of this enumeration. */
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
    }
}

macro_rules! Los_Errno_Os_Id {
    () => {
        (0x00 << 16)
    }
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

macro_rules! Os_Mem_Align_Size {
    {} => {
        std::mem::size_of::<u32>()
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


