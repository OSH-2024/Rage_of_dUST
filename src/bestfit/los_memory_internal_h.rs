include!("mempool_h.rs");

use std::mem;
use std::ptr;
//defined in los_memory.h
pub const los_record_lr_cnt: u32 = 3;

//defined in los_list.h
struct Los_DL_List {
    pst_prev: *mut Los_DL_List,
    pst_next: *mut Los_DL_List
}//Structure of a node in a doubly linked list

#[repr(C)]
struct LosMemCtlNode {
    prenode: *mut LosMemDynNode,
    /* Size and flag of the current node (the high two bits represent a flag,and the rest bits specify the size) */
    size_and_flag: mut u32,
    //
    gapsize: mut u32,
    checksum: mut u32,
    //
    linkreg: mut [u32;los_record_lr_cnt],
    //
    reserve2: mut u32,
    //
    union myunion{
        free_node_info: Los_DL_List,
        struct {
            magic: mut u32,
            taskid: mut u32,
            //
            moduled: mut u32
        }
    }
}

struct LosMemDynNode {
    self_node: LosMemCtlNode,
    //
    backup_node: LosMemCtlNode
}

//defined in los_multipledlinkhead_pri.h
struct LosMultipleDlinkHead{
    list_head: mut [Los_DL_List; Os_Multi_Dlnk_Num!()],
}
//functions in los_multipledlinkhead.c
pub const los_invalid_bit_index: u32 = 32;
pub const os_bitmap_mask: u32 = 0x1f;

fn Los_High_Bit_Get(bit_map: u32) -> u32{
    if bit_map == 0{
        return los_invalid_bit_index;
    }
    (os_bitmap_mask - bit_map.leading_zeros())
}

fn OsLog2(size: u32) -> u32{
    if size > 0 {
        return Los_High_Bit_Get(size);
    }
    0
}

fn Os_Dlnk_Multi_Head(headaddr: *mut std::ffi::c_void, size: u32) -> *mut Los_DL_List{
    let  dlinkhead: *mut LosMultipleDlinkHead = headaddr as *mut LosMultipleDlinkHead;
    let mut index: u32 = OsLog2(size);
    if index > Os_Max_Multi_Dlnk_Log2!() {
        return std::ptr::null_mut();
    }
    else if index <= Os_Min_Multi_Dlnk_Log2!(){
        index = Os_Min_Multi_Dlnk_Log2!();
    }
    (*dlinkhead).list_head + (index - Os_Min_Multi_Dlnk_Log2!())
}


macro_rules! Os_Max_Multi_Dlnk_Log2{
    () => {
        29
    };
}
macro_rules! Os_Min_Multi_Dlnk_Log2{
    () => {
        4
    };
}
macro_rules! Os_Multi_Dlnk_Num{
    () => {
        (Os_Max_Multi_Dlnk_Log2!() - Os_Min_Multi_Dlnk_Log2!() +1)
    };
}

macro_rules! Os_Multi_Dlnk_Head_Size{
    () => {
        mem::size_of::<LosMultipleDlinkHead>()
    };
}
macro_rules! Os_Dlnk_Head_Size{
    () => {
        Os_Multi_Dlnk_Head_Size
    };
}

macro_rules! Os_Mem_Align{
    ($p: expr, $alignsize: expr) => {
        (($p + $alignsize - 1) & ~($alignsize - 1))
    };
}

macro_rules! Os_Mem_Node_Head_Size{
    () => {
        mem::size_of::<LosMemDynNode>()
    };
}

macro_rules! Os_Mem_Min_Pool_Size{
    () => {
        (Os_Dlnk_Head_Size!() + 2*Os_Mem_Node_Head_Size!() + mem::size_of::<LosMemPoolInfo>())
    };
}

macro_rules! Is_Pow_Two{
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

macro_rules! Os_Mem_Node_Get_Aligned_Flag{
    ($sizeandflag: expr) => {
       ($sizeandflag & Os_Mem_Node_Aligned_Flag!())
    };
}

macro_rules! Os_Mem_Node_Set_Aligned_Flag{
    ($sizeandflag: expr) => {
       ($sizeandflag = ($sizeandflag | Os_Mem_Node_Aligned_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Aligned_GapSize{
    ($sizeandflag: expr) => {
       ($sizeandflag & (~Os_Mem_Node_Aligned_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Used_Flag{
    ($sizeandflag: expr) => {
       ($sizeandflag & Os_Mem_Node_Used_Flag!())
    };
}

macro_rules! Os_Mem_Node_Set_Aligned_Flag{
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
macro_rules! Os_Mem_Head{
    ($pool:expr, $size:expr) => {
       Os_Dlnk_Multi_Head(Os_Mem_Head_Addr!($pool), size);
    };
}

macro_rules! Os_Mem_Head_Addr{
    ($pool: expr) => {
       ((($pool as mut u32) + mem::size_of::<LosMemPoolInfo>()) as *mut std::ffi::c_void)
    };
}

macro_rules! Os_Mem_Next_Node{
    ($node: expr) =>{
        (((($node as *mut char) + Os_Mem_Node_Get_Size!((*($node)).self_node.size_and_flag)) as *mut std::ffi::c_void) as *mut LosMemDynNode)
    };
}

macro_rules! Os_Mem_First_Node{
    ($pool: expr) =>{
        ((((Os_Mem_Head_Addr!($pool) as *mut char) + Os_Dlnk_Head_Size!()) as *mut std::ffi::c_void) as *mut LosMemDynNode)
    };
}

macro_rules! Os_Mem_End_Node{
    ($pool:expr, $size:expr) =>{
        (((($pool as *mut char) + size - Os_Mem_Node_Head_Size!()) as *mut std::ffi::c_void) as *mut LosMemDynNode)

    };
}

macro_rules! Os_Mem_Middle_Addr_Open_End{
    ($startaddr:expr, $middleaddr:expr, $endaddr:expr) =>{
        (($startaddr as *mut char) <= ($middleaddr as *mut char)) && (($middleaddr as *mut char) < ($endaddr as *mut char))
    };
}

macro_rules! Os_Mem_Middle_Addr{
    ($startaddr:expr, $middleaddr:expr, $endaddr:expr) =>{
        (($startaddr as *mut char) <= ($middleaddr as *mut char)) && (($middleaddr as *mut char) <= ($endaddr as *mut char))
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














