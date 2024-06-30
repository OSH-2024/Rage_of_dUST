include!("mempool.rs");

use std::cell::Cell;
//使用 Cell 可变性容器,通过 .get 方法获取值，.set 方法来修改值
use std::mem;

//defined in los_memory.h
pub const los_record_lr_cnt: usize = 3;

//defined in los_list.h
struct LosDlList {
    pst_prev: *mut LosDlList,
    pst_next: *mut LosDlList,
} //Structure of a node in a doubly linked list

#[repr(C)]
struct LosMemCtlNode {
    prenode: *mut LosMemDynNode,
    /* Size and flag of the current node (the high two bits represent a flag,and the rest bits specify the size) */
    size_and_flag: Cell<u32>,
    //
    gapsize: Cell<u32>,
    checksum: Cell<u32>,
    //
    linkreg: [u32; los_record_lr_cnt],
    //
    reserve2: Cell<u32>,
    //
    myunion: Myunion,
}

union Myunion {
    free_node_info: std::mem::ManuallyDrop<Cell<LosDlList>>,
    extend_field: std::mem::ManuallyDrop<Moreinfo>,
}

struct Moreinfo {
    magic: Cell<u32>,
    taskid: Cell<u32>,
    //
    moduled: Cell<u32>,
}

struct LosMemDynNode {
    self_node: LosMemCtlNode,
    //
    backup_node: LosMemCtlNode,
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

macro_rules! Os_Max_Multi_Dlnk_Log2 {
    () => {
        29
    };
}
macro_rules! Os_Min_Multi_Dlnk_Log2 {
    () => {
        4
    };
}
macro_rules! Os_Multi_Dlnk_Num {
    () => {
        (Os_Max_Multi_Dlnk_Log2!() - Os_Min_Multi_Dlnk_Log2!() + 1)
    };
}

//defined in los_multipledlinkhead_pri.h
struct LosMultipleDlinkHead {
    list_head: [LosDlList; Os_Multi_Dlnk_Num!()],
}
//functions in los_multipledlinkhead.c
pub const los_invalid_bit_index: u32 = 32;
pub const os_bitmap_mask: u32 = 0x1f;

macro_rules! Os_Multi_Dlnk_Head_Size {
    () => {
        mem::size_of::<LosMultipleDlinkHead>()
    };
}
macro_rules! Os_Dlnk_Head_Size {
    () => {
        Os_Multi_Dlnk_Head_Size
    };
}

macro_rules! Os_Mem_Align{
    ($p: expr, $alignsize: expr) => {
        (($p + $alignsize - 1) & ~($alignsize - 1))
    };
}

macro_rules! Os_Mem_Node_Head_Size {
    () => {
        mem::size_of::<LosMemDynNode>()
    };
}

macro_rules! Os_Mem_Min_Pool_Size {
    () => {
        (Os_Dlnk_Head_Size!()
            + 2 * Os_Mem_Node_Head_Size!()
            + std::mem::size_of::<LosMemPoolInfo>())
    };
}

macro_rules! Is_Pow_Two {
    ($value: expr) => {
        (($value & ($value - 1)) == 0)
    };
}

macro_rules! Pool_Addr_Alignsize {
    () => {
        64
    };
}

macro_rules! Os_Mem_Node_Used_Flag {
    () => {
        0x80000000
    };
}

macro_rules! Os_Mem_Node_Aligned_Flag {
    () => {
        0x40000000
    };
}

macro_rules! Os_Mem_Node_Aligned_And_Used_Flag {
    () => {
        (Os_Mem_Node_Aligned_Flag!() | Os_Mem_Node_Used_Flag!())
    };
}

macro_rules! Os_Mem_Node_Get_Aligned_Flag {
    ($sizeandflag: expr) => {
        ($sizeandflag & Os_Mem_Node_Aligned_Flag!())
    };
}

macro_rules! Os_Mem_Node_Set_Aligned_Flag {
    ($sizeandflag: expr) => {
        ($sizeandflag = ($sizeandflag | Os_Mem_Node_Aligned_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Aligned_GapSize{
    ($sizeandflag: expr) => {
       ($sizeandflag & (~Os_Mem_Node_Aligned_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Used_Flag {
    ($sizeandflag: expr) => {
        ($sizeandflag & Os_Mem_Node_Used_Flag!())
    };
}

macro_rules! Os_Mem_Node_Set_Used_Flag {
    ($sizeandflag: expr) => {
        ($sizeandflag = ($sizeandflag | Os_Mem_Node_Used_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Size{
    ($sizeandflag: expr) => {
       ($sizeandflag & (~Os_Mem_Node_Aligned_And_Used_Flag!()))
    };
}
////
macro_rules! Os_Mem_Head {
    ($pool:expr, $size:expr) => {
        Os_Dlnk_Multi_Head(Os_Mem_Head_Addr!($pool), size);
    };
}

macro_rules! Os_Mem_Head_Addr{
    ($pool: expr) => {
       ((($pool as mut u32) + std::mem::size_of::<LosMemPoolInfo>()) as *mut std::ffi::c_void)
    };
}

macro_rules! Os_Mem_Next_Node {
    ($node: expr) => {
        (((($node as *mut char) + Os_Mem_Node_Get_Size!((*($node)).self_node.size_and_flag.get()))
            as *mut std::ffi::c_void) as *mut LosMemDynNode)
    };
}

macro_rules! Os_Mem_First_Node {
    ($pool: expr) => {
        ((((Os_Mem_Head_Addr!($pool) as *mut char) + Os_Dlnk_Head_Size!()) as *mut std::ffi::c_void)
            as *mut LosMemDynNode)
    };
}

macro_rules! Os_Mem_End_Node {
    ($pool:expr, $size:expr) => {
        (((($pool as *mut char) + size - Os_Mem_Node_Head_Size!()) as *mut std::ffi::c_void)
            as *mut LosMemDynNode)
    };
}

macro_rules! Os_Mem_Middle_Addr_Open_End {
    ($startaddr:expr, $middleaddr:expr, $endaddr:expr) => {
        (($startaddr as *mut char) <= ($middleaddr as *mut char))
            && (($middleaddr as *mut char) < ($endaddr as *mut char))
    };
}

macro_rules! Os_Mem_Middle_Addr {
    ($startaddr:expr, $middleaddr:expr, $endaddr:expr) => {
        (($startaddr as *mut char) <= ($middleaddr as *mut char))
            && (($middleaddr as *mut char) <= ($endaddr as *mut char))
    };
}

macro_rules! Os_Mem_Set_Magic{
    ($value: expr) => {
        ($value = ((&mut $value) as mut u32) ^ ((-1) as mut u32) )
    };
}

macro_rules! Os_Mem_Magic_Valid{
    ($value: expr) =>{
        (($value as mut u32) ^ ((&mut $value) as mut u32) == ((-1) as mut u32))
    };
}
