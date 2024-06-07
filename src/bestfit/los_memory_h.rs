include!("los_multipledlinkhead.rs");

struct LosMemPoolStatus{
    uw_total_used_size: Cell<u32>,
    uw_total_free_size: Cell<u32>,
    uw_max_free_node_size: Cell<u32>,
    uw_used_node_num: Cell<u32>,
    uw_free_node_num: Cell<u32>,
    uw_usage_waterline: Cell<u32>

}//LOS_MEM_POOL_STATUS

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

macro_rules! Os_Mem_Align_Size {
    {} => {
        std::mem::size_of::<u32>()
    }
}
