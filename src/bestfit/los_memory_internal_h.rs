include!("mempool.rs");

use std::cell::Cell;
//使用 Cell 可变性容器,通过 .get 方法获取值，.set 方法来修改值
use std::mem;
extern crate cortex_m;
use cortex_m::asm;


struct SpinLockS{
    raw_lock: u32   
}

pub static mut g_mem_spin: SpinLockS = SpinLockS{raw_lock: 0};

//defined in los_memory.h
pub const los_record_lr_cnt: usize = 3;

//defined in los_list.h
struct LosDlList {
    pst_prev: *mut LosDlList,
    pst_next: *mut LosDlList
}//Structure of a node in a doubly linked list

#[repr(C)]
struct LosMemCtlNode {
    prenode: *mut LosMemDynNode,
    /* Size and flag of the current node (the high two bits represent a flag,and the rest bits specify the size) */
    size_and_flag: Cell<u32>,

    //#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    gapsize:  Cell<u32>,

    //#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    checksum:  Cell<u32>,

    //#[cfg(feature = "LOSCFG_MEM_LEAKCHECK")]
    linkreg:  [u32;los_record_lr_cnt],

    //#[cfg(feature = "LOSCFG_AARCH64")]
    reserve2:  Cell<u32>,

    myunion: Myunion
}

union Myunion{
    free_node_info: std::mem::ManuallyDrop<LosDlList>,
    extend_field:  std::mem::ManuallyDrop<Moreinfo>
}

struct Moreinfo{
    magic: Cell<u32>,
    taskid: Cell<u32>,

    //#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
    moduleid: Cell<u32>
}

struct LosMemDynNode {
    self_node: LosMemCtlNode,

    //#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    backup_node: LosMemCtlNode
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

//defined in los_multipledlinkhead_pri.h
struct LosMultipleDlinkHead{
    list_head:  [LosDlList; Os_Multi_Dlnk_Num!()]
}
//functions in los_multipledlinkhead.c
pub const los_invalid_bit_index: u32 = 32;
pub const os_bitmap_mask: u32 = 0x1f;


macro_rules! Os_Multi_Dlnk_Head_Size{
    () => {
        std::mem::size_of::<LosMultipleDlinkHead>() as u32
    };
}
macro_rules! Os_Dlnk_Head_Size{
    () => {
        Os_Multi_Dlnk_Head_Size!()
    };
}

macro_rules! Os_Mem_Align{
    ($p: expr, $alignsize: expr) => {
        (($p as usize + $alignsize - 1) & !($alignsize - 1))
    };
}

macro_rules! Os_Mem_Node_Head_Size{
    () => {
       std::mem::size_of::<LosMemDynNode>() as u32
    };
}

macro_rules! Os_Mem_Min_Pool_Size{
    () => {
        (Os_Dlnk_Head_Size!() + 2*Os_Mem_Node_Head_Size!() + std::mem::size_of::<LosMemPoolInfo>() as u32)
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
       ($sizeandflag & Os_Mem_Node_Aligned_Flag!()) != 0
    };
}

macro_rules! Os_Mem_Node_Set_Aligned_Flag{
    ($sizeandflag: expr) => {
       ($sizeandflag = ($sizeandflag | Os_Mem_Node_Aligned_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Aligned_GapSize{
    ($sizeandflag: expr) => {
       ($sizeandflag & (!Os_Mem_Node_Aligned_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Used_Flag{
    ($sizeandflag: expr) => {
       ($sizeandflag & Os_Mem_Node_Used_Flag!()) != 0
    };
}

macro_rules! Os_Mem_Node_Set_Used_Flag{
    ($sizeandflag: expr) => {
        ($sizeandflag = ($sizeandflag | Os_Mem_Node_Used_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Size{
    ($sizeandflag: expr) => {
       ($sizeandflag & (!Os_Mem_Node_Aligned_And_Used_Flag!()))
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
       ((($pool as u32) + std::mem::size_of::<LosMemPoolInfo>() as u32) as *mut std::ffi::c_void)
    };
}

macro_rules! Os_Mem_Next_Node{
    ($node: expr) =>{
        (((($node as *mut u8).offset(Os_Mem_Node_Get_Size!((*($node)).self_node.size_and_flag.get()) as isize)) as *mut std::ffi::c_void) as *mut LosMemDynNode)
    };
}

macro_rules! Os_Mem_First_Node{
    ($pool: expr) =>{
        ((((Os_Mem_Head_Addr!($pool) as *mut u8).offset(Os_Dlnk_Head_Size!() as isize)) as *mut std::ffi::c_void) as *mut LosMemDynNode)
    };
}

macro_rules! Os_Mem_End_Node{
    ($pool:expr, $size:expr) =>{
        (((($pool as *mut u8).offset(($size - Os_Mem_Node_Head_Size!()) as isize)) as *mut std::ffi::c_void) as *mut LosMemDynNode)

    };
}

macro_rules! Os_Mem_Middle_Addr_Open_End{
    ($startaddr:expr, $middleaddr:expr, $endaddr:expr) =>{
        (($startaddr as *mut u8) <= ($middleaddr as *mut u8)) && (($middleaddr as *mut u8) < ($endaddr as *mut u8))
    };
}

macro_rules! Os_Mem_Middle_Addr{
    ($startaddr:expr, $middleaddr:expr, $endaddr:expr) =>{
        (($startaddr as *mut u8) <= ($middleaddr as *mut u8)) && (($middleaddr as *mut u8) <= ($endaddr as *mut u8))
    };
}

macro_rules! Os_Mem_Set_Magic{
    ($value: expr) => {
        ($value = (&$value as *const _ as usize) ^ ((u32::MAX) as usize) )//////
    };
}

macro_rules! Os_Mem_Magic_Valid{
    ($value: expr) =>{
        (($value as usize) ^ (&$value as *const _ as usize) == ((u32::MAX) as usize))//////
    };
}

///mem_lock_unlock
fn Arch_Int_Lock()->u32{
    let int_save: u32;
    let temp: u32;
    unsafe {
        asm!(
            "mrs    $0, cpsr",
            "orr    $1, $0, #0xc0",
            "msr    cpsr_c, $1",
            lateout("r") int_save,
            lateout("r") temp,
        );
    }
    int_save
}

fn Arch_Int_Restore(int_save: u32) ->(){
    unsafe {
        asm!(
            "msr cpsr_c, $0", // 将 intSave 的值写入 CPSR
            in(reg) int_save, // 使用 in(reg) 来指定输入寄存器
        );
    }
}

fn Los_Int_Lock()-> u32{
    Arch_Int_Lock()
}

fn Los_Int_Restore(int_save: u32)->(){
    Arch_Int_Restore(int_save);
}

fn Los_Spin_Lock_Save(lock: *mut SpinLockS, int_save: *mut u32) ->() {
    ////lock as std::ffi::c_void;
    *int_save = Los_Int_Lock();
}

fn Los_Spin_Unlock_Restore(lock: *mut SpinLockS, int_save: u32) ->() {
    ////lock as std::ffi::c_void;
    Los_Int_Restore(int_save);
}

//#[macro_export]
macro_rules! Mem_Lock {
    ($int_save: expr) =>{
        Los_Spin_Lock_Save(std::ptr::addr_of_mut!(g_mem_spin), &mut ($int_save));
    };
}

//#[macro_export]
macro_rules! Mem_Unlock {
    ($int_save: expr) =>{
        Los_Spin_Unlock_Restore(std::ptr::addr_of_mut!(g_mem_spin), $int_save);
    };
}












