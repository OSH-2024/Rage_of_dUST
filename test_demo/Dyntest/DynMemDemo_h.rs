use std::cell::Cell;
use std::arch::asm;
use std::ffi::c_void;
use std::any::Any;
use std::mem;
use std::ptr;


pub const LOS_OK: u32 = 1;
pub const LOS_NOK: u32 = 0;
pub const los_record_lr_cnt: usize = 3;
pub static mut g_mem_spin: SpinLockS = SpinLockS{raw_lock: 0};
pub const OS_NULL_INT: u32 = 0xFFFFFFFF;
pub const los_invalid_bit_index: u32 = 32;
pub const os_bitmap_mask: u32 = 0x1f;
static mut g_int_count: [u32; 1] = [0;1];
static mut G_POOL_HEAD: *mut std::ffi::c_void = ptr::null_mut();

struct LosMemPoolInfo {
    pool: *mut std::ffi::c_void,//begining address of this pool
    next_pool: *mut std::ffi::c_void, 
    pool_size: u32 
    // Add other fields if necessary
    //memstat add after changing memstat
    //slab related
}

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

struct LosDlList {
    pst_prev: *mut LosDlList,
    pst_next: *mut LosDlList
}//Structure of a node in a doubly linked list

struct LosMemDynNode {
    self_node: LosMemCtlNode,

    //#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    backup_node: LosMemCtlNode
}

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


macro_rules! Os_Max_Multi_Dlnk_Log2{
    () => {
        29
    };
}
macro_rules! Os_Mem_Node_Aligned_Flag {
    () => {
        0x40000000
    };
}

macro_rules! Os_Mem_Node_Get_Aligned_Gapsize{
    ($sizeandflag: expr) => {
       ($sizeandflag & (!Os_Mem_Node_Aligned_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Aligned_And_Used_Flag {
    () => {
        (Os_Mem_Node_Aligned_Flag!() | Os_Mem_Node_Used_Flag!())
    };
}

macro_rules! Os_Mem_Node_Get_Size{
    ($sizeandflag: expr) => {
       ($sizeandflag & (!Os_Mem_Node_Aligned_And_Used_Flag!()))
    };
}

macro_rules! Os_Mem_Node_Get_Used_Flag{
    ($sizeandflag: expr) => {
       ($sizeandflag & Os_Mem_Node_Used_Flag!()) != 0
    };
}

macro_rules! Os_Mem_Next_Node{
    ($node: expr) =>{
        (((($node as *mut u8).offset(Os_Mem_Node_Get_Size!((*($node)).self_node.size_and_flag.get()) as isize)) as *mut std::ffi::c_void) as *mut LosMemDynNode)
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

struct LosMultipleDlinkHead{
    list_head:  [LosDlList; Os_Multi_Dlnk_Num!()]
}

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


macro_rules! Is_Aligned {
    ($value:expr, $align_size:expr) => {
        ($value & ($align_size as u32 - 1)) == 0
    };
}

macro_rules! Os_Mem_Node_Head_Size{
    () => {
        std::mem::size_of::<LosMemDynNode>() as u32
    };
}
macro_rules! Os_Mem_Node_Get_Aligned_Flag{
    ($sizeandflag: expr) => {
       ($sizeandflag & Os_Mem_Node_Aligned_Flag!()) != 0
    };
}

macro_rules! Os_Mem_Min_Pool_Size{
    () => {
        (Os_Dlnk_Head_Size!() + 2 * Os_Mem_Node_Head_Size!() + std::mem::size_of::<LosMemPoolInfo>() as u32)
    };
}

macro_rules! Os_Mem_Align{
    ($p: expr, $alignsize: expr) => {
        (($p as usize + $alignsize - 1) & !($alignsize - 1))
    };
}

macro_rules! Os_Mem_Align_Size {
    {} => {
        std::mem::size_of::<u32>()
    }
}

///mem_lock_unlock

struct SpinLockS{
    raw_lock: u32   
}

fn Arch_Int_Lock()->u32{
    unsafe {
        let int_save: u32 = 1;
        /* TODO: 汇编报错
        let temp: u32;
        asm!(
            "mrs $0, cpsr",        // 读取当前程序状态寄存器到 $0
            "orr $1, $0, #0xc0",  // 将 $0 和 0xc0 进行或操作，并存储到 $1
            "msr cpsr_c, $1",      // 将 $1 写回到 cpsr_c 寄存器
            "=&r"(int_save) "=&r"(temp), // 输出约束，指定使用通用寄存器
            "memory"             // 告诉编译器内联汇编代码可能会修改内存
        );*/
        int_save
    }
}

fn Arch_Int_Restore(int_save: u32) ->(){
    /*unsafe {
        asm!(
            //"msr cpsr_c, $0", // 将 intSave 的值写入 CPSR
            //in(reg) int_save, // 使用 in(reg) 来指定输入寄存器
        );
    }*/
}

fn Los_Int_Lock()-> u32{
    Arch_Int_Lock()
}

fn Los_Int_Restore(int_save: u32)->(){
    Arch_Int_Restore(int_save);
}

fn Los_Spin_Lock_Save(lock: *mut SpinLockS, int_save: *mut u32) ->() {
    ////lock as std::ffi::c_void;
    unsafe {*int_save = Los_Int_Lock();}
}

fn Los_Spin_Unlock_Restore(lock: *mut SpinLockS, int_save: u32) ->() {
    ////lock as std::ffi::c_void;
    Los_Int_Restore(int_save);
}

/*macro_rules! Mem_Lock {
    ($int_save: expr) =>{
        Los_Spin_Lock_Save(std::ptr::addr_of_mut!(g_mem_spin), &mut $int_save);
    };
}*/

pub fn Mem_Lock(mut int_save: u32){
    unsafe {Los_Spin_Lock_Save(std::ptr::addr_of_mut!(g_mem_spin), &mut int_save);}

}

pub fn Mem_Unlock(mut int_save: u32){
    unsafe {Los_Spin_Unlock_Restore(std::ptr::addr_of_mut!(g_mem_spin), int_save);}
}


/*macro_rules! Mem_Unlock {
    ($int_save: expr) =>{
        Los_Spin_Unlock_Restore(std::ptr::addr_of_mut!(g_mem_spin), $int_save);
    };
}*/

//mempool部分
unsafe fn los_mempoolsizeget(pool: *mut std::ffi::c_void ) -> u32 {
    let mut heapmanager: *mut LosMemPoolInfo = std::ptr::null_mut();
    if pool.is_null() {
        return OS_NULL_INT;
    }
    heapmanager = pool as *mut LosMemPoolInfo;
    (*heapmanager).pool_size
}

unsafe fn Os_Mem_Mul_Pool_Init(pool: *mut std::ffi::c_void, size: u32) -> u32 {
    let mut next_pool = G_POOL_HEAD;
    let mut cur_pool = G_POOL_HEAD;
    while !next_pool.is_null() {
        let pool_end = next_pool.offset(unsafe{los_mempoolsizeget(next_pool)} as isize);
        if (pool <= next_pool && (pool.offset(size as isize) as usize) > next_pool as usize) || 
           ((pool as usize) < pool_end as usize && (pool.offset(size as isize) as usize) >= pool_end as usize) {
            println!("pool [{:?}, {:?}] conflict with pool [{:?}, {:?}]", pool, pool.offset(size as isize), next_pool, next_pool.offset(unsafe{los_mempoolsizeget(next_pool)} as isize));
            return LOS_NOK;
        }
        cur_pool = next_pool;
        next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool;
    }

    if G_POOL_HEAD.is_null() {
        G_POOL_HEAD = pool;
    } else {
        (*(cur_pool as *mut LosMemPoolInfo)).next_pool = pool;
    }

    (*(pool as *mut LosMemPoolInfo)).next_pool = std::ptr::null_mut();
    LOS_OK
}

unsafe fn Os_Mem_Mul_Pool_Deinit(pool: *const std::ffi::c_void) -> u32 {
    let mut ret = LOS_NOK;
    let mut next_pool: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut cur_pool: *mut std::ffi::c_void = std::ptr::null_mut();

    if pool.is_null() {
        return ret;
    }

    if pool == G_POOL_HEAD {
        G_POOL_HEAD = (*(G_POOL_HEAD as *mut LosMemPoolInfo)).next_pool;
        return LOS_OK;
    }

    cur_pool = G_POOL_HEAD;
    next_pool = G_POOL_HEAD;
    while !next_pool.is_null() {
        if pool == next_pool {
            (*(cur_pool as *mut LosMemPoolInfo)).next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool;
            ret = LOS_OK;
            break;
        }
        cur_pool = next_pool;
        next_pool = (*(next_pool as *mut LosMemPoolInfo)).next_pool;
    }

    ret
}

//init部分

macro_rules! Os_Mem_Head_Addr{
    ($pool: expr) => {
       ((($pool as u32) + std::mem::size_of::<LosMemPoolInfo>() as u32) as *mut std::ffi::c_void)
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

macro_rules! Os_Mem_Head{
    ($pool:expr, $size:expr) => {
       Os_Dlnk_Multi_Head(Os_Mem_Head_Addr!($pool), $size);
    };
}

macro_rules! Os_Mem_Node_Used_Flag {
    () => {
        0x80000000 
    };
}

macro_rules! Os_Mem_Node_Set_Used_Flag{
    ($sizeandflag: expr) => {
        ($sizeandflag = $sizeandflag | Os_Mem_Node_Used_Flag!())
    };
}

macro_rules! Arm_Sysreg_Read {
    ($reg:expr) => {
        {
        let mut val: u32 = 1;
        //TODO: 汇编报错 
        /*
        unsafe {
            asm!("mrc $0, 0, $1, c15, c0, 0"
                : "=&r"(val), // Add a comma after val
                : "r"($reg)
                : "memory");
        }*/
        val
    }
};
}

macro_rules! Os_Mem_Set_Magic{
    ($value: expr) => {
        ($value = ((&$value as *const _ as usize) ^ ((u32::MAX) as usize)) as u32)//////
    };
}

macro_rules! Os_Int_Active {
    () => {
        Int_Active()
    };
}

macro_rules! Os_Int_Inactive {
    () => {
        !Os_Int_Active!() != 0
    };
}

fn OsLog2(size: u32) -> u32{
    if size > 0 {
        return Los_High_Bit_Get(size);
    }
    0
}

fn Los_High_Bit_Get(bit_map: u32) -> u32{
    if bit_map == 0{
        return los_invalid_bit_index;
    }
    os_bitmap_mask - bit_map.leading_zeros()
}

fn Int_Active() -> u32 {
    let mut int_count: u32;
    let mut int_save: u32 = Los_Int_Lock();
    unsafe {int_count = g_int_count[0];}
    Los_Int_Restore(int_save);

    int_count
    //TODO
}

unsafe fn Os_Dlnk_Multi_Head(headaddr: *mut std::ffi::c_void, size: u32) -> *mut LosDlList{
    let  dlinkhead: *mut LosMultipleDlinkHead = headaddr as *mut LosMultipleDlinkHead;
    let mut index: u32 = OsLog2(size);
    if index > Os_Max_Multi_Dlnk_Log2!() {
        return std::ptr::null_mut();
    }
    else if index <= Os_Min_Multi_Dlnk_Log2!(){
        index = Os_Min_Multi_Dlnk_Log2!();
    }
    &mut (*dlinkhead).list_head[(index - Os_Min_Multi_Dlnk_Log2!())as usize]
}

fn Os_Mem_Init(pool: *mut std::ffi::c_void, size: u32) -> u32 {
    let mut pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut new_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut end_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut list_node_head: *mut LosDlList = std::ptr::null_mut();
    let mut pool_size:u32 = size;

    #[cfg(feature = "LOSCFG_KERNEL_LMS")]
    {
        if !g_lms_Mem_Init_Hook.is_null() {
            pool_size = g_lms_Mem_Init_Hook(pool, size);
            if pool_size == 0 {
                pool_size = size;
            }
        }
    }
    unsafe {
        (*pool_info).pool = pool;
        (*pool_info).pool_size = pool_size;
        
        Os_Dlnk_Init_Multi_Head(Os_Mem_Head_Addr!(pool));
        new_node = Os_Mem_First_Node!(pool);
       //println!("new_node: {:?}, head:{}, pool:{:p}", new_node, pool_size - ((new_node as u32) - (pool as u32)) - Os_Mem_Node_Head_Size!(), pool);
        /*(*new_node).self_node.size_and_flag.set(pool_size - ((new_node as u32) - (pool as u32)) - Os_Mem_Node_Head_Size!());
        (*new_node).self_node.prenode = Os_Mem_End_Node!(pool, pool_size) as *mut LosMemDynNode;
        list_node_head = Os_Mem_Head!(pool, pool_size - ((new_node as u32) - (pool as u32)) - Os_Mem_Node_Head_Size!());
        if !list_node_head.is_null() {
            return LOS_NOK;
        }*/
        //Los_List_Tail_Insert(list_node_head, &mut *(*new_node).self_node.myunion.free_node_info);
        ///end_node = Os_Mem_End_Node!(pool, pool_size) as *mut LosMemDynNode;
        //std::ptr::write_bytes(end_node, 0, std::mem::size_of::<LosMemDynNode>());
        //(*end_node).self_node.prenode = new_node;
        //(*end_node).self_node.size_and_flag.set(Os_Mem_Node_Head_Size!()) ;

        //let mut size: u32 = (*end_node).self_node.size_and_flag.get();
        //Os_Mem_Node_Set_Used_Flag!(size);
        //(*end_node).self_node.size_and_flag.set(size);

        //Os_Mem_Set_Magic_Num_And_Task_Id(end_node);

        #[cfg(feature = "LOSCFG_MEM_TASK_STAT")]
        {
            let stat_size = std::mem::size_of_val(&(*pool_info).stat);
            /*std::ptr::write_bytes(&mut (*pool_info).stat, 0, stat_size);*/
            Memset_S(&mut (*pool_info).stat,stat_size, 0, stat_size);
            (*pool_info).stat.mem_total_used = std::mem::size_of::<LosMemPoolInfo>() + Os_Multi_Dlnk_Head_Sizeize!() +
                                               Os_Mem_Node_Get_Size!((*end_node).self_node.size_and_flag.get());
            (*pool_info).stat.mem_total_peak = (*pool_info).stat.mem_total_used;
        }

        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        {
            Os_Mem_Node_Save(new_node);
            Os_Mem_Node_Save(end_node);
        }
    }

    LOS_OK
}

fn Los_List_Add(list: *mut LosDlList, node: *mut LosDlList)->(){
    unsafe {
        (*node).pst_next = (*list).pst_next;
        (*node).pst_prev = list;
        (*(*list).pst_next).pst_prev = node;
        (*list).pst_next = node;
    }
}
fn Los_List_Tail_Insert(list: *mut LosDlList, node: *mut LosDlList)->(){
    unsafe {Los_List_Add((*list).pst_prev, node);}
} 

unsafe fn Os_Dlnk_Init_Multi_Head(headaddr: *mut std::ffi::c_void)->(){
    let dlinkhead: *mut LosMultipleDlinkHead = headaddr as *mut LosMultipleDlinkHead;
    let mut list_node_head: *mut LosDlList = &mut (*dlinkhead).list_head[0];
    let mut index: u32;
    for index in  1..=Os_Multi_Dlnk_Num!() {
        list_node_head = unsafe { list_node_head.offset(1) };
        //Los_List_Init(list_node_head);
    }
}

fn Los_List_Init(list: *mut LosDlList)->(){
    unsafe {
        (*list).pst_next = list;
        (*list).pst_prev = list;
    }
}

#[inline]
unsafe fn Os_Mem_Set_Magic_Num_And_Task_Id(node: *mut LosMemDynNode) {
    //#[cfg(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT"))]
    {
        let run_task: *mut LosTaskCB = Os_Curr_Task_Get();

        let mut value: u32 = (*node).self_node.myunion.extend_field.magic.get();
        Os_Mem_Set_Magic!(value);
        (*node).self_node.myunion.extend_field.magic.set(value);

        if !run_task.is_null() && Os_Int_Inactive!() {
            Os_Mem_Taskid_Set(node, (*run_task).task_id);
        } else {
            Os_Mem_Taskid_Set(node, 12);
        }
    }
}

fn Arch_Curr_Task_Get() -> *mut std::ffi::c_void {
    (Arm_Sysreg_Read!(Tpidrprw!()) as u32) as *mut std::ffi::c_void
}
//los_task_pri.h 186
fn Os_Curr_Task_Get() -> *mut LosTaskCB {
    Arch_Curr_Task_Get() as *mut LosTaskCB
}

fn Os_Mem_Taskid_Set(node: *mut LosMemDynNode, task_id: u32) {
    unsafe {(*node).self_node.myunion.extend_field.taskid.set(task_id);}

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(node);
    }
}

//alloc部分
fn Os_Mem_Alloc_With_Check(pool: *mut LosMemPoolInfo, size: u32) 
    -> *mut std::ffi::c_void{
    let alloc_node: *mut LosMemDynNode = ptr::null_mut();
    let alloc_size: usize;

    #[cfg(feature = "loscfg_base_mem_node_integrity_check")]{
        let mut tmp_node = std::ptr::null_mut();
        let mut pre_node = std::ptr::null_mut();
    }
    let first_node: *mut std::ffi::c_void = (Os_Mem_Head_Addr!(pool) as *mut u8).wrapping_add(Os_Dlnk_Head_Size!() as usize) as *mut std::ffi::c_void;

    #[cfg(feature = "loscfg_base_mem_node_integrity_check")]{
        if(Os_Mem_Integrity_Check(pool, &mut tmp_node, &mut pre_node)){
            Os_Mem_Integrity_Check_Error(tmp_node, pre_node);
            return None;
        }
    }
    alloc_size = Os_Mem_Align!(size + Os_Mem_Node_Head_Size!(), Os_Mem_Align_Size!());
    //alloc_node = Os_Mem_Find_Suitable_Free_Block(pool as *mut std::ffi::c_void, alloc_size as u32).expect("REASON");
    if alloc_node.is_null() {
        Os_Mem_Info_Alert(pool as *mut std::ffi::c_void, alloc_size as u32);
        return std::ptr::null_mut();
    }
    unsafe {
        if alloc_size + Os_Mem_Node_Head_Size!() as usize + Os_Mem_Align_Size!() <= (*alloc_node).self_node.size_and_flag.get() as usize {
            Os_Mem_Split_Node(pool as *mut std::ffi::c_void, alloc_node, alloc_size as u32);
        }
        Os_Mem_List_Delete(&mut *(*alloc_node).self_node.myunion.free_node_info, first_node);
        Os_Mem_Set_Magic_Num_And_Task_Id(alloc_node);

        let mut size_and_flag:u32 = (*alloc_node).self_node.size_and_flag.get();
        Os_Mem_Node_Set_Used_Flag!(size_and_flag);
        (*alloc_node).self_node.size_and_flag.set(size_and_flag);

        Os_Mem_Node_Debug_Operate(pool as *mut u8, alloc_node, size);
    }

    #[cfg(feature = "LOSCFG_KERNEL_LMS")]
    {
        if !g_lms_malloc_hook.is_null() {
            g_lms_malloc_hook(unsafe { alloc_node.offset(1) });
        }
    }
    unsafe {alloc_node.offset(1) as *mut std::ffi::c_void}
}

#[cfg(not(feature = "LOSCFG_MEM_HEAD_BACKUP"))]
unsafe fn Os_Mem_List_Delete(node: *mut LosDlList, first_node: *const std::ffi::c_void)  {
    // unsafe {
        //let _ = first_node;
        (*(*node).pst_next).pst_prev = (*node).pst_prev;
        (*(*node).pst_prev).pst_next = (*node).pst_next;
        (*node).pst_next = std::ptr::null_mut();
        (*node).pst_prev = std::ptr::null_mut();//空指针，mark
    // }
}

fn Os_Mem_Node_Debug_Operate(pool: *mut u8, alloc_node: *mut LosMemDynNode, size: u32) {
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(alloc_node);
    }

    #[cfg(feature = "LOSCFG_MEM_LEAKCHECK")]
    {
        Os_Mem_Link_Register_Record(alloc_node);
    }
}

fn Os_Mem_Info_Alert(pool: *mut std::ffi::c_void, alloc_size: u32)->(){
    #[cfg(feature = "LOSCFG_MEM_DEBUG")]
    {
        Print_Err("---------------------------------------------------\
        --------------------------------------------------------");
        Os_Mem_Info_Print(pool);
        Print_Err(&format!(
        "[{}] No suitable free block, require free node size: 0x{:x}",
        std::module_path!(), alloc_size));
        Print_Err("---------------------------------------------------\
                --------------------------------------------------------");
    }
}

fn Os_Mem_Split_Node(pool: *mut std::ffi::c_void, alloc_node: *mut LosMemDynNode, alloc_size: u32) {
    unsafe {
        let mut new_free_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut next_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut list_node_head: *mut LosDlList = std::ptr::null_mut();
    let first_node = Os_Mem_Head_Addr!(pool).offset(Os_Dlnk_Head_Size!() as isize) as *const std::ffi::c_void;

    new_free_node = (alloc_node as *mut LosMemDynNode).offset(alloc_size as isize) as *mut LosMemDynNode;
    unsafe {
        /*self_node啥用，mark*/
        (*new_free_node).self_node.prenode = alloc_node;
        (*new_free_node).self_node.size_and_flag = ((*alloc_node).self_node.size_and_flag.get() - alloc_size).into();
        (*alloc_node).self_node.size_and_flag = alloc_size.into();
        next_node = Os_Mem_Next_Node!(new_free_node);
        (*next_node).self_node.prenode = new_free_node;
    }
    if !Os_Mem_Node_Get_Used_Flag!((*next_node).self_node.size_and_flag.get()) {
        Os_Mem_List_Delete(&mut *(*next_node).self_node.myunion.free_node_info, first_node);
        Os_Mem_Merge_Node(next_node);
    }
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(next_node);
    }
    list_node_head = Os_Mem_Head!(pool, (*new_free_node).self_node.size_and_flag.get());
    if list_node_head.is_null() {
        return;
    }
    Os_Mem_List_Add(list_node_head, &mut *(*new_free_node).self_node.myunion.free_node_info, first_node);
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(new_free_node);
    }
    }
}

fn Os_Mem_Merge_Node(node: *mut LosMemDynNode) {
    let mut next_node: *mut LosMemDynNode = std::ptr::null_mut();

    unsafe {
        (*(*node).self_node.prenode).self_node.size_and_flag = ((*node).self_node.size_and_flag.get() + (*(*node).self_node.prenode).self_node.size_and_flag.get()).into();
        next_node = (node as *mut LosMemDynNode).offset((*node).self_node.size_and_flag.get() as isize) as *mut LosMemDynNode;
        (*next_node).self_node.prenode = (*node).self_node.prenode;
    }
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save((*node).self_node.prenode);
        Os_Mem_Node_Save(next_node);
    }
    Os_Mem_Clear_Node(node);
}

fn Os_Mem_Clear_Node(node: *mut LosMemDynNode) {
    unsafe {
        std::ptr::write_bytes(node, 0, std::mem::size_of::<LosMemDynNode>());
    }
}

#[cfg(not(feature = "LOSCFG_MEM_HEAD_BACKUP"))]
unsafe fn Os_Mem_List_Add(list_node: *mut LosDlList, node: *mut LosDlList, first_node: *const std::ffi::c_void) {
    // unsafe {
        //let _ = first_node; (VOID)firstNode mark
        (*node).pst_next = (*list_node).pst_next;
        (*node).pst_prev = list_node;
        (*(*list_node).pst_next).pst_prev = node;
        (*list_node).pst_next = node;
    // }
}

fn Os_Mem_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void) -> u32 {
    let mut ret: u32 = LOS_OK;
    let mut gap_size: u32 = 0;
    let mut node: *mut LosMemDynNode;

    loop {
        unsafe {
           
        //gap_size = *((ptr.offset(std::mem::size_of::<u32>() as isize * -1) )as *mut u32) as u32;
        
        if Os_Mem_Node_Get_Aligned_Flag!(gap_size) && Os_Mem_Node_Get_Used_Flag!(gap_size) {
            eprintln!("[{}:{}]: gapSize:0x{:x} error", "Os_Mem_Free", line!(), gap_size);
            return ret;
        }
        
        node = (ptr.offset((Os_Mem_Node_Head_Size!() as isize)*(-1))) as *mut LosMemDynNode;
        
        if Os_Mem_Node_Get_Aligned_Flag!(gap_size) {
            gap_size = Os_Mem_Node_Get_Aligned_Gapsize!(gap_size);
            if (gap_size & (Os_Mem_Align_Size!() as u32- 1)) != 0 || gap_size > (ptr as u32 - Os_Mem_Node_Head_Size!()) {
                eprintln!("illegal gapSize: 0x{:x}", gap_size);
                break;
            }
            node = (ptr.offset(((Os_Mem_Node_Head_Size!() + gap_size) as isize)*(-1))) as *mut LosMemDynNode;
        }
        
        #[cfg(not(feature = "LOSCFG_MEM_HEAD_BACKUP"))]
        Os_Do_Mem_Free(pool, ptr, node);
        break;
        }
    }

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        ret = Os_Mem_Backup_Check_And_Restore(pool, node, ptr as *mut std::ffi::c_void);
        if ret == 0 {
            Os_Do_Mem_Free(pool, ptr, node);
        }
    }

    ret
}

fn Os_Do_Mem_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, node: *mut LosMemDynNode)->(){
    Os_Mem_Check_Used_Node(pool, node);
    //Os_Mem_Free_Node(node, pool as *mut LosMemPoolInfo);
    #[cfg(feature = "LOSCFG_KERNEL_LMS")]
    {
        if !g_lms_Free_Hook.is_null() {
            g_lms_Free_Hook(ptr);
        }
    }
} 

fn Os_Mem_Free_Node(node: *mut LosMemDynNode, pool: *mut LosMemPoolInfo) {
    unsafe{
        let mut next_node: *mut LosMemDynNode = std::ptr::null_mut();
        let mut list_node_head: *mut LosDlList = std::ptr::null_mut();
        let first_node = (Os_Mem_Head_Addr!(pool) as *mut u8).offset(Os_Dlnk_Head_Size!() as isize) as *const std::ffi::c_void;
        
        //(*node).self_node.size_and_flag = Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get()).into();
        
        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        {
            Os_Mem_Node_Save(node);
        }
    
        #[cfg(feature = "LOSCFG_MEM_LEAKCHECK")]
        {
            Os_Mem_Link_Register_Record(node);
        }
        if !Os_Mem_Node_Get_Used_Flag!((*((*node).self_node.prenode)).self_node.size_and_flag.get()) {
            let pre_node = (*node).self_node.prenode;
            Os_Mem_Merge_Node(node);
            next_node = Os_Mem_Next_Node!(pre_node);
            if !Os_Mem_Node_Get_Used_Flag!((*next_node).self_node.size_and_flag.get()) {
                Os_Mem_List_Delete(&mut *(*next_node).self_node.myunion.free_node_info, first_node);
                Os_Mem_Merge_Node(next_node);
            }
            Os_Mem_List_Delete(&mut *(*pre_node).self_node.myunion.free_node_info, first_node);
            list_node_head = Os_Mem_Head!(pool, (*pre_node).self_node.size_and_flag.get());
            if list_node_head.is_null() {
                return;
            }
            Os_Mem_List_Add(list_node_head, &mut *(*pre_node).self_node.myunion.free_node_info, first_node);
        } else {
            next_node = Os_Mem_Next_Node!(node);
            if !Os_Mem_Node_Get_Used_Flag!((*next_node).self_node.size_and_flag.get()) {
                Os_Mem_List_Delete(&mut *(*next_node).self_node.myunion.free_node_info, first_node);
                Os_Mem_Merge_Node(next_node);
            }
            list_node_head = Os_Mem_Head!(pool, (*node).self_node.size_and_flag.get());
            if list_node_head.is_null() {
                return;
            }
            Os_Mem_List_Add(list_node_head,&mut *(*node).self_node.myunion.free_node_info, first_node);
        }
    }

}

fn Os_Mem_Check_Used_Node(pool: *const std::ffi::c_void, node: *mut LosMemDynNode) {
}

