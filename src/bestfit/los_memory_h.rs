include!("los_multipledlinkhead.rs");

struct LosMemPoolStatus{
    uw_total_used_size: Cell<u32>,
    uw_total_free_size: Cell<u32>,
    uw_max_free_node_size: Cell<u32>,
    uw_used_node_num: Cell<u32>,
    uw_free_node_num: Cell<u32>,
    uw_usage_waterline: Cell<u32>

}//LOS_MEM_POOL_STATUS

pub const OS_INVALID: u32 = -1;

enum LOS_MOUDLE_ID {
    Los_Mod_SYS = 0x0,          /**< System ID. Its value is 0x0. */
    Los_Mod_Mem = 0x1,          /**< Dynamic memory module ID. Its value is 0x1. */
    Los_Mod_Tsk = 0x2,          /**< Task module ID. Its value is 0x2. */
    Los_Mod_Swtmr = 0x3,        /**< Software timer module ID. Its value is 0x3. */
    Los_Mod_Tick = 0x4,         /**< Tick module ID. Its value is 0x4. */
    Los_Mod_Msg = 0x5,          /**< Message module ID. Its value is 0x5. */
    Los_Mod_Que = 0x6,          /**< Queue module ID. Its value is 0x6. */
    Los_Mod_Sem = 0x7,          /**< Semaphore module ID. Its value is 0x7. */
    Los_Mod_Mbox = 0x8,         /**< Static memory module ID. Its value is 0x8. */
    Los_Mod_Mmu = 0xd,          /**< MMU module ID. Its value is 0xd. */
    Los_Mod_Log = 0xe,          /**< Log module ID. Its value is 0xe. */
    Los_Mod_Err = 0xf,          /**< Error handling module ID. Its value is 0xf. */
    Los_Mod_Exc = 0x10,         /**< Exception handling module ID. Its value is 0x10. */
    Los_Mod_Butt                /**< It is end flag of this enumeration. */
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
        Los_Errno_Os_Error!(Los_Mod_Mem, 0x4)
    };
}

macro_rules! Los_Errno_Memcheck_Disabled {
    {} => {
        Los_Errno_Os_Error!(Los_Mod_Mem, 0x5)
    }
}

macro_rules! Los_Errno_Memcheck_Para_Null {
    {} => {
        Los_Errno_Os_Error!(Los_Mod_Mem, 0x1)
    }
}

macro_rules! Los_Errno_Memcheck_Outside {
    {} => {
        Los_Errno_Os_Error!(Los_Mod_Mem, 0x2)
    }
}

macro_rules! Los_Errno_Memcheck_No_Head {
    {} => {
        Los_Errno_Os_Error!(Los_Mod_Mem, 0x3)
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


