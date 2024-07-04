include!("los_memory_h.rs");

//regs.h 50 GPT生成，存疑
macro_rules! Arm_Sysreg_Read {
    ($reg:expr) => {{
        let mut val: u32;
        unsafe {
            llvm_asm!("mrc $0, 0, $1, c15, c0, 0"
                : "=r"(val)
                : "r"($reg)
                : "memory");
        }
        val
    }};
}
//regs.h 77
macro_rules! CP15_Reg {
    ($CRn:expr,$Op1:expr,$CRm:expr,$Op2:expr)=>{
        "p15, "#$Op1 ", %0, "#$CRn ","#$CRm ","#$Op2
    };
}
//regs.h 139
macro_rules! Tpidrprw {
    () => {
        CP15_Reg(c13, 0, c0, 4)
    };
}
//task.h 64
fn Arch_Curr_Task_Get() -> *mut std::ffi::c_void {
    (Arm_Sysreg_Read!(Tpidrprw!()) as u32) as *mut std::ffi::c_void
}
//los_task_pri.h 186
fn Os_Curr_Task_Get() -> *mut LosTaskCB {
    Arch_Curr_Task_Get() as *mut LosTaskCB
}
//los_exc.c中定义，在los_memory.c 233行调用
fn Los_Back_Trace() {
    //[#cfg(feature="LOSCFG_BACKTRACE")]
    let run_task: *mut LosTaskCB = Os_Curr_Task_Get();
    printl
}
macro_rules! Os_Back_Trace {
    () => {
        Los_Back_Trace()
    };
}

//los_hwi.c 353
fn Int_Active()->u32{
    let int_count:u32;
    let int_save:u32=Los_Int_Lock();

    //TODO
}

//los_hwi.h 64
macro_rules! Os_Int_Active{
    ()=>{
        Int_Active()
    };
}

//los_hwi.h 74
macro_rules! Os_Int_Inactive{
    ()=>{
        !Os_Int_Active!()
    };
}
// use std::panic::Location; //用于获取行数
// extern crate stdext;
// use stdext::function_name; //用于获取函数名
// macro_rules! os_check_null_return {
//     ($param:expr) => {
//         if $param.is_null() {
//             let location = Location::caller();
//             print_err!("{} {}\n", function_name!(), location.line());
//             return;
//         }
//     };
// }
//69
static mut m_auc_sys_mem0: *mut u8 = 0 as *mut u8;
static mut m_auc_sys_mem1: *mut u8 = 0 as *mut u8;

type MallocHook = fn();
static mut g_malloc_hook: Option<MallocHook> = None;

use std::arch::asm; //?
#[link_section = ".data.init"]
static mut g_sys_mem_addr_end: usize = 0;

//#[cfg(feature = "LOSCFG_EXC_INTERACTION")]
#[link_section = ".data.init"]
static mut g_exc_interact_mem_size: usize = 0;

//#[cfg(feature = "LOSCFD_BASE_MEM_NODE_SIZE_CHECK")]
static mut g_mem_check_level: u8 = 0xff; //LOS_MEM_CHECK_LEVEL_DEFAULT

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")] //MEM_MODULE_MAX=0x20
static mut g_module_mem_used_size: [u32; 0x20 + 1] = [0; 0x20 + 1];

//94
fn Os_Mem_Taskid_Set(node: *mut LosMemDynNode, task_id: u32) {
    (*node).self_node.myunion.extend_field.taskid.set(task_id);

    //#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(node);
    }
}

pub const fn Os_Mem_Taskid_Get(node: *mut LosMemDynNode) -> u32 {
    (*node).self_node.myunion.extend_field.taskid.get()
}
//110
#[inline]
//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Modid_Set(node: *mut LosMemDynNode, module_id: u32) {
    (*node)
        .self_node
        .myunion
        .extend_field
        .moduleid
        .set(module_id);
    //#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(node);
    }
}

//119
#[inline]
//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Modid_Get(node: *mut LosMemDynNode) -> u32 {
    (*node).self_node.myunion.extend_field.moduleid.get()
}

//130
#[inline]
fn Os_Mem_Set_Magic_Num_And_Task_Id(node: *mut LosMemDynNode) {
    //#[cfg(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT"))]
    {
        let run_task: *mut LosTaskCB = Os_Curr_Task_Get();

        let mut value: u32 = (*node).self_node.myunion.extend_field.magic.get();
        Os_Mem_Set_Magic!(value);
        (*node).self_node.myunion.extend_field.magic.set(value);

        if !run_task.is_null() && Os_Int_Inactive!() {
            Os_Mem_Taskid_Set(node, (*run_task).task_id);
        } 
        else{
            Os_Mem_Taskid_Set(node, 12);
        }
        
    }
}

//151行
//rust函数定义顺序不影响其可见性和使用性，原文在此处提前声明2098行的OsMemFindNodeCtrl，rust中不需要
//154行
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
const CHECKSUM_MAGICNUM: u32 = 0xDEADBEEF;

//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
macro_rules! Os_Mem_Node_Checksum_Calculate {
    ($ctl_node:expr) => {
        let ctl_node = $ctl_node; //TODO:
        ((ctl_node.free_node_info.pst_prev)
            ^ (ctl_node.free_node_info.pst_next)
            ^ (ctl_node.pre_node)
            ^ ctlNode.gap_size
            ^ ctlNode.size_and_flag
            ^ CHECKSUM_MAGICNUM)
    };
}

//164行
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Disp_Ctl_Node(ctl_node: *mut LosMemCtlNode) {
    let mut checksum: u32;

    checksum = Os_Mem_Node_Checksum_Calculate!(ctl_node);

    println!("node:{:p} checksum={:p}[{:p}] freeNodeInfo.pstPrev={:p} ",
           "freeNodeInfo.pstNext={:p} preNode={:p} gapSize=0x{:x} sizeAndFlag=0x{:x}",
           ctl_node,
           ctl_node.checksum,
           checksum,//TODO:
           ctl_node->free_node_info.pst_prev,
           ctl_node->free_node_info.pst_next,
           ctl_node->pre_node,
           ctl_node->gap_size,
           ctl_node->size_and_flag);
}

//182行
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Disp_More_Details(node: *mut LosMemDynNode) {
    let task_id: u32;
    //
    println!("************************************************");
    Os_Mem_Disp_Ctl_Node(&mut (*node).self_node as *mut LosMemCtlNode);
    println!("the address of node :{:p}", &node);

    if !Os_Mem_Node_Get_Used_Flag!((*node).self_node.size_and_flag.get()) {
        println!("this is a FREE node");
        println!("************************************************\n");
        return;
    }

    task_id = Os_Mem_Taskid_Get(node);
    if task_id >= 12
    //g_taskMaxNum?
    {
        println!("The task [ID:0x{:x}] is ILLEGAL", task_id);
        if task_id == 12
        //g_taskMaxNum
        {
            println!("PROBABLY alloc by SYSTEM INIT, NOT IN ANY TASK");
        }
        println!("************************************************\n");
        return;
    }

    let task_cb: *mut LosTaskCB = Os_Tcb_From_Tid!(task_id);
    if (task_cb.tsk_status & 0x0001u16)
        || (task_cb.task_entry == None)
        || (task_cb.task_name == None)
    //OS_TASK_STATUS_UNUSED=0x0001U，taskStatus为UINT16类型
    {
        println!("The task [ID:0x{:x}] is NOT CREATED(ILLEGAL)", task_id);
        println!("************************************************\n");
        return;
    }

    println!(
        "allocated by task: {} [ID = 0x{:x}]\n",
        task_cb.task_name, task_id
    );
    //ifdef LOSCFG_MEM_MUL_MODULE
    println!("allocted by moduleId:{}", Os_Mem_Modid_Get(node));

    println!("************************************************\n");
}

//224行
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Disp_Wild_Pointer_Msg(node: *mut LosMemDynNode, ptr: *mut std::ffi::c_void) {
    println!("************************************************");
    println!(
        "find an control block at: {:p}, gap size: 0x{:x}, sizeof(LosMemDynNode): 0x{:x}",
        node,
        (*node).self_node.gapsize.get() as u32,
        std::mem::size_of::<LosMemDynNode>() as u32
    );
    println!(
        "the pointer should be: {:p}",
        node.offset(((*node).self_node.gapsize.get() + std::mem::size_of::<LosMemDynNode>() as u32) as isize)
    );
    println!("the pointer given is: {:p}", ptr);
    println!("PROBABLY A WILD POINTER");
    Os_Back_Trace!(); //TODO
    println!("************************************************");
}

//237行
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Checksum_Set(ctl_node: *mut LosMemCtlNode) {
    (*ctl_node).checksum = Os_Mem_Node_Checksum_Calculate!(ctl_node);
}

//242行
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Checksum_Verify(ctl_node: *mut LosMemCtlNode) -> bool {
    (*ctl_node).checksum == Os_Mem_Node_Checksum_Calculate!(ctl_node)
}

//247行
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Backup_Setup(node: *mut LosMemDynNode) {
    let node_pre: *mut LosMemDynNode = (*node).self_node.prenode;
    if !node_pre.is_null() {
        (*node_pre).backup_node.myunion.free_node_info.pst_next =
            (*node).self_node.myunion.free_node_info.pst_next;
        (*node_pre).backup_node.myunion.free_node_info.pst_prev =
            (*node).self_node.myunion.free_node_info.pst_prev;
        (*node_pre).backup_node.prenode = (*node).self_node.prenode;
        (*node_pre).backup_node.checksum = (*node).self_node.checksum;
        (*node_pre).backup_node.gapsize = (*node).self_node.gapsize;
        (*node_pre).backup_node.size_and_flag = (*node).self_node.size_and_flag;
    }
}

//260
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Next_Get(
    pool: *mut std::ffi::c_void,
    node: *mut LosMemDynNode,
) -> *mut LosMemDynNode {
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;

    if node == Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        return Os_Mem_First_Node!(pool);
    } else {
        return Os_Mem_Next_Node!(node);
    }
}

//271
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Backup_Setup_4_Next(pool: *mut std::ffi::c_void, node: *mut LosMemDynNode) -> u32 {
    let node_next: *mut LosMemDynNode = Os_Mem_Node_Next_Get(pool, node);

    if !Os_Mem_Checksum_Verify(&mut (*node_next).self_node as *mut LosMemCtlNode) {
        println!("[//TODO:function name]the next node is broken!!");
        Os_Mem_Disp_Ctl_Node(&mut (*node_next).self_node as *mut LosMemCtlNode);
        println!("Current node details:");
        Os_Mem_Disp_More_Details(node);

        return 1; //LOS_NOK
    }

    if !Os_Mem_Checksum_Verify(&mut (*node).backup_node as *mut LosMemCtlNode) { 
        (*node).backup_node.myunion.free_node_info.pst_next =
            (*node_next).self_node.free_node_info.pst_next;
        (*node).backup_node.myunion.free_node_info.pst_prev =
            (*node_next).self_node.free_node_info.pst_prev;
        (*node).backup_node.prenode = (*node_next).self_node.prenode;
        (*node).backup_node.checksum = (*node_next).self_node.checksum;
        (*node).backup_node.gapsize = (*node_next).self_node.gapsize;
        (*node).backup_node.size_and_flag = (*node_next).self_node.size_and_flag;
    }
    return 0; //LOS_OK
}

//295
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Backup_Do_Restore(
    pool: *mut std::ffi::c_void,
    node_pre: *mut LosMemDynNode,
    node: *mut LosMemDynNode,
) -> u32 {
    //省略了node!=NULL的判断
    if node.is_null() {
        println!("the node is NULL.");
        return 1; //LOS_NOK
    }

    println!("the backup node information of current node in previous node:");
    Os_Mem_Disp_Ctl_Node((*node_pre).backup_node);
    println!("the detailed information of previous node:");
    Os_Mem_Disp_More_Details(node_pre);

    (*node).self_node.myunion.free_node_info.pst_next =
        (*node_pre).backup_node.myunion.free_node_info.pst_next;
    (*node).self_node.myunion.free_node_info.pst_prev =
        (*node_pre).backup_node.myunion.free_node_info.pst_prev;
    (*node).self_node.prenode = (*node_pre).backup_node.prenode;
    (*node).self_node.checksum = (*node_pre).backup_node.checksum;
    (*node).self_node.gapsize = (*node_pre).backup_node.gapsize;
    (*node).self_node.size_and_flag = (*node_pre).backup_node.size_and_flag;

    /* we should re-setup next node's backup on current node */
    return Os_Mem_Backup_Setup_4_Next(pool, node);
}

//317
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_First_Node_PrevGet(pool_info: *mut LosMemPoolInfo) -> *mut LosMemDynNode {
    let node_pre: *mut LosMemDynNode = Os_Mem_End_Node!(pool_info, pool_info.pool_size);

    if !Os_Mem_Checksum_Verify(&mut (*node_pre).self_node as *mut LosMemCtlNode) {
        println!("the current node is THE FIRST NODE !");
        println!("[//TODO:function name]: the node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&mut (*node_pre).self_node as *mut LosMemCtlNode);
    }
    if !Os_Mem_Checksum_Verify(&mut (*node_pre).backup_node as *mut LosMemCtlNode) {
        println!("the current node is THE FIRST NODE !");
        println!("[//TODO:function name]: the backup node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&mut (*node_pre).backup_node as *mut LosMemCtlNode);
    }

    return node_pre;
}

//337
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Prev_Get(
    pool: *mut std::ffi::c_void,
    node: *mut LosMemDynNode,
) -> *mut LosMemDynNode {
    let node_cur: *mut LosMemDynNode = Os_Mem_First_Node!(pool);
    // let node_pre:*mut LosMemDynNode=Os_Mem_First_Node_PrevGet(pool);
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;

    if node == Os_Mem_First_Node!(pool) {
        return Os_Mem_First_Node_PrevGet(pool_info);
    }

    while node_cur < Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        if !Os_Mem_Checksum_Verify(&mut (*node_cur).self_node as *mut LosMemCtlNode) {
            println!("[//TODO:function name]the node information of current node is bad !!");
            Os_Mem_Disp_Ctl_Node(&mut (*node_cur).self_node as *mut LosMemCtlNode);

            if node_pre.is_null() {
                return ptr::null_mut();
            }

            println!("the detailed information of previous node:");
            Os_Mem_Disp_More_Details(node_pre);
        }

        if !Os_Mem_Checksum_Verify(&mut (*node_cur).backup_node as *mut LosMemCtlNode) {
            println!("[//TODE:function name]the backup node information of current node is bad !!");
            Os_Mem_Disp_Ctl_Node(&mut (*node_cur).backup_node as *mut LosMemCtlNode);

            if !node_pre.is_null() {
                println!("the detailed information of previous node:");
                Os_Mem_Disp_More_Details(node_pre);
            }

            if Os_Mem_Backup_Setup_4_Next(pool, node_cur) != 0
            //LOS_NOK
            {
                return ptr::null_mut();
            }
        }

        if Os_Mem_Next_Node!(node_cur) == node {
            return node_cur;
        }

        if Os_Mem_Next_Node!(node_cur) > node {
            break;
        }

        node_pre = node_cur;

        node_cur = Os_Mem_Next_Node!(node_cur);
    }

    return ptr::null_mut();
}

//395
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Prev_Try_Get(
    pool: *mut std::ffi::c_void,
    node: *mut *mut LosMemDynNode,
    ptr: *mut std::ffi::c_void,
) -> *mut LosMemDynNode {
    let node_should_be: u32;
    let node_cur: *mut LosMemDynNode = Os_Mem_First_Node!(pool);
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let node_pre: *mut LosMemDynNode = Os_Mem_End_Node!(pool, (*pool_info).pool_size);

    while node_cur < Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        if !Os_Mem_Checksum_Verify(&mut (*node_cur).self_node as *mut LosMemCtlNode) {
            println!("[//TODO:function name]the node information of current node is bad !!");
            Os_Mem_Disp_Ctl_Node(&mut (*node_cur).self_node as *mut LosMemCtlNode);

            println!("the detailed information of previous node:");
            Os_Mem_Disp_More_Details(node_pre);

            //due to the every step's checksum verify, nodePre is trustful
            if Os_Mem_Backup_Do_Restore(pool, node_pre, node_cur) != 0
            //LOS_OK
            {
                return ptr::null_mut();
            }
        }

        if !Os_Mem_Checksum_Verify(&mut (*node_cur).backup_node as *mut LosMemCtlNode) {
            println!("[//TODO:function name]the backup node information of current node is bad !!");
            Os_Mem_Disp_Ctl_Node(&mut (*node_cur).backup_node as *mut LosMemCtlNode);

            if !node_pre.is_null() {
                println!("the detailed information of previous node:");
                Os_Mem_Disp_More_Details(node_pre);
            }

            if Os_Mem_Backup_Do_Restore(pool, node_pre, node_cur) != 0
            //LOS_NOK
            {
                return ptr::null_mut();
            }
        }

        node_should_be = (node_cur as u32)
            + (*node_cur).self_node.gapsize.get()
            + std::mem::size_of::<LosMemDynNode>() as u32;
        if node_should_be == ptr as u32 {
            *node = node_cur;
            return node_pre;
        }

        if Os_Mem_Next_Node!(node_cur) > ptr as *mut LosMemDynNode {
            break;
        }

        node_pre = node_cur;

        node_cur = Os_Mem_Next_Node!(node_cur);
    }

    return ptr::null_mut();
}

//449
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Backup_Try_Restore(
    pool: *mut std::ffi::c_void,
    node: *mut *mut LosMemDynNode,
    ptr: *mut std::ffi::c_void,
) -> u32 {
    let node_head: *mut LosMemDynNode = std::ptr::null_mut();
    let node_pre: *mut LosMemDynNode =
        Os_Mem_Node_Prev_Try_Get(pool, &mut node_head as *mut *mut LosMemDynNode, ptr);

    if node_pre.is_null() {
        return 1; //LOS_NOK
    }

    *node = node_head;
    return Os_Mem_Backup_Do_Restore(pool, node_pre as *mut LosMemDynNode, *node);
}

//461
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Backup_Restore(pool: *mut std::ffi::c_void, node: *mut LosMemDynNode) -> u32 {
    let node_pre: *mut LosMemDynNode = Os_Mem_Node_Prev_Get(pool, node);

    if node_pre.is_null() {
        return 1; //LOS_NOK
    }

    return Os_Mem_Backup_Do_Restore(pool, node_pre as *mut LosMemDynNode, node);
}

//471
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Backup_Check_And_Restore(
    pool: *mut std::ffi::c_void,
    node: *mut LosMemDynNode,
    ptr: *mut std::ffi::c_void,
) -> u32 {
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let start_node: *mut LosMemDynNode = Os_Mem_First_Node!(pool);
    let end_node: *mut LosMemDynNode = Os_Mem_End_Node!(pool, (*pool_info).pool_size);

    if Os_Mem_Middle_Addr!(start_node, node, end_node) {
        //GapSize is bad or node is broken, we need to verify & try to restore
        if !Os_Mem_Checksum_Verify((&mut (*node).self_node) as *mut LosMemCtlNode) {
            node = (ptr as u32 - Os_Mem_Node_Head_Size!()) as *mut LosMemDynNode;
            return Os_Mem_Backup_Try_Restore(pool, &mut node as *mut *mut LosMemDynNode, ptr);
        }
    }
    return 0; //LOS_OK
}

//487
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Set_Gap_Size(ctl_node: *mut LosMemCtlNode, gap_size: u32) {
    (*ctl_node).gapsize.set(gap_size);
}

//492
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Save(node: *mut LosMemDynNode) {
    Os_Mem_Set_Gap_Size(&mut (*node).self_node as *mut LosMemCtlNode, 0);
    Os_Mem_Checksum_Set(&mut (*node).self_node as *mut LosMemCtlNode);
    Os_Mem_Backup_Setup(node);
}

//499
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Save_With_Gap_Size(node: *mut LosMemDynNode, gap_size: u32) {
    Os_Mem_Set_Gap_Size(&mut (*node).self_node as *mut LosMemCtlNode, gap_size);
    Os_Mem_Checksum_Set(&mut (*node).self_node as *mut LosMemCtlNode);
    Os_Mem_Backup_Setup(node);
}

//506
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_List_Delete(node: *mut LosDlList, first_node: *mut std::ffi::c_void) {
    let dyn_node: *mut LosMemDynNode = std::ptr::null_mut();

    (*(*node).pst_next).pst_prev = (*node).pst_prev;
    (*(*node).pst_prev).pst_next = (*node).pst_next;

    if (*node).pst_next as *mut std::ffi::c_void >= first_node {
        dyn_node = Los_Dl_List_Entry!((*node).pst_next, LosMemDynNode, self_node.free_node_info);
        Os_Mem_Node_Save(dyn_node);
    }
    if (*node).pst_prev as *mut std::ffi::c_void >= first_node {
        dyn_node = Los_Dl_List_Entry!((*node).pst_prev, LosMemDynNode, self_node.free_node_info);
        Os_Mem_Node_Save(dyn_node);
    }

    (*node).pst_next = std::ptr::null_mut();
    (*node).pst_prev = std::ptr::null_mut();

    dyn_node = Los_Dl_List_Entry!(node, LosMemDynNode, self_node.free_node_info);
    Os_Mem_Node_Save(dyn_node);
}

//530
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_MEM_List_Add(
    list_node: *mut LosDlList,
    node: *mut LosDlList,
    first_node: *mut std::ffi::c_void,
) {
    let dyn_node: *mut LosMemDynNode = std::ptr::null_mut();

    (*node).pst_next = (*list_node).pst_next;
    (*node).pst_prev = list_node;

    dyn_node = Los_Dl_List_Entry!(node, LosMemDynNode, self_node.free_node_info);
    Os_Mem_Node_Save(dyn_node);

    (*(*list_node).pst_next).pst_prev = node;
    if (*list_node).pst_next as *mut std::ffi::c_void >= first_node {
        dyn_node = Los_Dl_List_Entry!(
            (*list_node).pst_next,
            LosMemDynNode,
            self_node.free_node_info
        );
        Os_Mem_Node_Save(dyn_node);
    }

    (*list_node).pst_next = node;
}

//549
//#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Los_Mem_Bad_Node_Show(pool: *mut std::ffi::c_void) {
    let node_pre: *mut LosMemDynNode = std::ptr::null_mut();
    let tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let int_save: u32;

    if pool.is_null() {
        return;
    }

    Mem_Lock!(int_save);

    tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        if Os_Mem_Checksum_Verify(&mut (*tmp_node).self_node as *mut LosMemCtlNode) {
            tmp_node = Os_Mem_Next_Node!(tmp_node);
            continue;
        }

        node_pre = Os_Mem_Node_Prev_Get(pool, tmp_node);
        if node_pre.is_null() {
            println!("the current node is invalid, but cannot find its previous Node");
            tmp_node = Os_Mem_Next_Node!(tmp_node);
            continue;
        }

        println!("the detailed information of previous node:");
        Os_Mem_Disp_More_Details(node_pre);

        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);
    println!("check finish");
}

/*********/
//以下带有/*********/的函数为未实现的函数
/*********/

fn Los_Mem_Realloc(
    pool: *mut std::ffi::c_void,
    ptr: *mut std::ffi::c_void,
    size: u32,
) -> *mut std::ffi::c_void {
    let mut int_save: u32;
    let new_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut is_slab_mem: bool = false;
    let mut mem_free_value: u32;
    if Os_Mem_Node_Get_Used_Flag!(size)
        || Os_Mem_Node_Get_Aligned_Flag!(size)
        || pool == std::ptr::null_mut()
    {
        return std::ptr::null_mut();
    }
    if ptr == std::ptr::null_mut() {
        /*********/
        new_ptr = Los_Mem_Alloc(pool, size);
        /*********/
        //Los_Trace!();
        return new_ptr;
    }
    if size == 0 {
        /*********/
        mem_free_value = Los_Mem_Free(pool, ptr);
        /*********/
        //Los_Trace!();
        return new_ptr;
    }

    Mem_Lock!(int_save);
    /*********/
    new_ptr = Os_Mem_Realloc_Slab(pool, ptr, &mut is_slab_mem, size);
    /*********/

    if is_slab_mem == true {
        Mem_Unlock!(int_save);
        //Los_Trace!();
        return new_ptr;
    }
    /*********/
    new_ptr = Os_Mem_Realloc(pool, ptr, size);
    /*********/
    Mem_Unlock!(int_save);
    //Los_Trace!();

    new_ptr
}

fn Los_Mem_Total_Used_Get(pool: *mut std::ffi::c_void) -> u32 {
    let mut tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut mem_used: u32 = 0;
    let mut int_save: u32;
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }

    Mem_Lock!(int_save);
    //
    let mut tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        // 在这里处理 tmp_node 指向的节点
        if Os_Mem_Node_Get_Used_Flag!((*tmp_node).self_node.size_and_flag.get()) {
            mem_used += Os_Mem_Node_Get_Size!((*tmp_node).self_node.size_and_flag.get());
        }
        // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);

    mem_used
}

fn Los_Mem_Used_Blks_Get(pool: *mut std::ffi::c_void) -> u32 {
    let mut tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut blknums: u32 = 0;
    let mut int_save: u32;
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }

    Mem_Lock!(int_save);
    //
    let mut tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        // 在这里处理 tmp_node 指向的节点
        if Os_Mem_Node_Get_Used_Flag!((*tmp_node).self_node.size_and_flag.get()) {
            blknums = blknums + 1;
        }
        // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);

    blknums
}

fn Los_Mem_Task_Id_Get(ptr: *mut std::ffi::c_void) -> u32 {
    let tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    //m_auc_sys_mem1: UINT8 *
    let pool_info: *mut LosMemPoolInfo =
        (m_auc_sys_mem1 as *mut std::ffi::c_void) as *mut LosMemPoolInfo;
    let mut int_save: u32;

    //#[cfg(feature = "LOSCFG_EXC_INTERACTION")]
    {
        if ptr < m_auc_sys_mem1 as *mut std::ffi::c_void {
            pool_info = (m_auc_sys_mem0 as *mut std::ffi::c_void) as *mut LosMemPoolInfo
        }
    }

    if (ptr == std::ptr::null_mut())
        || (ptr < Os_Mem_First_Node!(pool_info) as *mut std::ffi::c_void)
        || (ptr > Os_Mem_End_Node!(pool_info, (*pool_info).pool_size) as *mut std::ffi::c_void)
    {
        println!(
            "input ptr {:p} is out of system memory range[{:p}, {:p}]\n",
            ptr,
            Os_Mem_First_Node!(pool_info),
            Os_Mem_End_Node!(pool_info, (*pool_info).pool_size)
        );
        return OS_INVALID;
        //(UINT32)(-1)
    }

    Mem_Lock!(int_save);

    let mut tmp_node = Os_Mem_First_Node!(pool_info);
    while tmp_node <= Os_Mem_End_Node!(pool_info, (*pool_info).pool_size) {
        // 在这里处理 tmp_node 指向的节点
        if (ptr as u32) < (tmp_node as u32) {
            if Os_Mem_Node_Get_Used_Flag!((*((*tmp_node).self_node.prenode))
                .self_node
                .size_and_flag
                .get())
            {
                Mem_Unlock!(int_save);
                return (*((*tmp_node).self_node.prenode))
                    .self_node
                    .myunion
                    .extend_field
                    .taskid
                    .get();
            } else {
                Mem_Unlock!(int_save);
                println!("input ptr {:p} is belong to a free mem node\n", ptr);
                return OS_INVALID;
            }
        }
        // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);
    OS_INVALID
}

fn Los_Mem_Free_Blks_Get(pool: *mut std::ffi::c_void) -> u32 {
    let tmp_node: *mut LosMemDynNode = std::ptr::null_mut();
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut blknums: u32 = 0;
    let mut int_save: u32;
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }

    Mem_Lock!(int_save);
    //
    let mut tmp_node = Os_Mem_First_Node!(pool);
    while tmp_node <= Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        // 在这里处理 tmp_node 指向的节点
        if !Os_Mem_Node_Get_Used_Flag!((*tmp_node).self_node.size_and_flag.get()) {
            blknums = blknums + 1;
        }
        // 获取下一个节点
        tmp_node = Os_Mem_Next_Node!(tmp_node);
    }

    Mem_Unlock!(int_save);

    blknums
}

fn Los_Mem_Last_Used_Get(pool: *mut std::ffi::c_void) -> u32 {
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let node: *mut LosMemDynNode = std::ptr::null_mut();
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }
    node = (*(Os_Mem_End_Node!(pool, (*pool_info).pool_size)))
        .self_node
        .prenode;
    if Os_Mem_Node_Get_Used_Flag!((*node).self_node.size_and_flag.get()) {
        return (((node as *mut char)
            .offset(Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get()) as isize))
            as usize
            + std::mem::size_of::<LosMemDynNode>()) as u32;
    } else {
        return ((node as *mut char).offset(std::mem::size_of::<LosMemDynNode>() as isize)) as u32;
    }
}

fn Os_Mem_Reset_End_Node(pool: *mut std::ffi::c_void, pre_addr: u32) -> () {
    let end_node: *mut LosMemDynNode =
        (Os_Mem_End_Node!(pool, (*(pool as *mut LosMemPoolInfo)).pool_size)) as *mut LosMemDynNode;
    (*end_node)
        .self_node
        .size_and_flag
        .set(Os_Mem_Node_Head_Size!());
    if pre_addr != 0 {
        (*end_node).self_node.prenode =
            (pre_addr - std::mem::size_of::<LosMemDynNode>() as u32) as *mut LosMemDynNode;
    }

    let mut sizeandflag: u32 = (*end_node).self_node.size_and_flag.get();
    Os_Mem_Node_Set_Used_Flag!(sizeandflag);
    (*end_node).self_node.size_and_flag.set(sizeandflag);

    /*********/
    Os_Mem_Set_Magic_Num_And_Task_ID(end_node);

    //#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(end_node);
    }
    /*********/
}

fn Los_Mem_Pool_Size_Get(pool: *mut std::ffi::c_void) -> u32 {
    if pool == std::ptr::null_mut() {
        return LOS_NOK;
    }
    (*(pool as *mut LosMemPoolInfo)).pool_size
}

fn Los_Mem_Info_Get(pool: *mut std::ffi::c_void, pool_status: *mut LosMemPoolStatus) -> u32 {
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut ret: u32;
    let mut int_save: u32;
    if pool_status == std::ptr::null_mut() {
        println!("can't use NULL addr to save info\n");
        return LOS_NOK;
    }
    if (pool_info == std::ptr::null_mut()) || (pool as u32 != ((*pool_info).pool) as u32) {
        println!("wrong mem pool addr: {:?}, line:{}\n", pool_info, line!());
        return LOS_NOK;
    }
    Mem_Lock!(int_save);
    /*********/
    ret = Os_Mem_Info_Get(pool_info, pool_status);
    /*********/
    Mem_Unlock!(int_save);

    ret
}

fn Os_Show_Free_Node(index: u32, length: u32, count_num: *mut u32) -> () {
    let mut count: u32 = 0;
    println!("\n    block size:  ");
    for count in 0..=length - 1 {
        println!("2^{:<5}", (index + Os_Min_Multi_Dlnk_Log2!() + count));
    }
    println!("\n    node number: ");
    count = 0;
    for count in 0..=length - 1 {
        println!(
            "  {:?}",
            count_num.wrapping_add((count + index).try_into().unwrap())
        );
    }
}

fn Los_Mem_Free_Node_Show(pool: *mut std::ffi::c_void) -> u32 {
    let list_node_head: *mut LosDlList = std::ptr::null_mut();
    let head_addr: *mut LosMultipleDlinkHead =
        (pool as u32 + std::mem::size_of::<LosMemPoolInfo>() as u32) as *mut LosMultipleDlinkHead;
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut link_head_index: u32;
    let mut count_num: [u32; Os_Multi_Dlnk_Num!()] = [0; Os_Multi_Dlnk_Num!()];
    let mut int_save: u32;

    if (pool == std::ptr::null_mut()) || (pool as u32 != ((*pool_info).pool) as u32) {
        println!("wrong mem pool addr: {:p}, line:{}\n", pool_info, line!());
        return LOS_NOK;
    }

    println!("\n   ************************ left free node number**********************");
    Mem_Lock!(int_save);

    for link_head_index in 0..=Os_Multi_Dlnk_Num!() - 1 {
        list_node_head = (*head_addr).list_head[link_head_index].pst_next;
        while list_node_head != &mut ((*head_addr).list_head[link_head_index]) {
            list_node_head = (*list_node_head).pst_next;
            count_num[link_head_index] = count_num[link_head_index] + 1;
        }
    }

    link_head_index = 0;
    while link_head_index < Os_Multi_Dlnk_Num!() {
        if link_head_index + Column_Num!() < Os_Multi_Dlnk_Num!() {
            Os_Show_Free_Node(link_head_index, Column_Num!(), count_num.as_mut_ptr());
            link_head_index += Column_Num!();
        } else {
            Os_Show_Free_Node(
                link_head_index,
                Os_Multi_Dlnk_Num!() - 1 - link_head_index,
                count_num.as_mut_ptr(),
            );
            break;
        }
    }

    Mem_Unlock!(int_save);
    println!("\n   ********************************************************************\n\n");

    LOS_OK
}

//#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
fn Los_Mem_Node_Size_Check(
    pool: *mut std::ffi::c_void,
    ptr: *mut std::ffi::c_void,
    total_size: *mut u32,
    avail_size: *mut u32,
) -> u32 {
    let head: *mut std::ffi::c_void = std::ptr::null_mut();
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let end_pool: *mut u8 = std::ptr::null_mut();

    if g_mem_check_level == Los_Mem_Check_Level_Disable!() {
        return Los_Errno_Memcheck_Disabled!();
    }

    if (pool == std::ptr::null_mut())
        || (ptr == std::ptr::null_mut())
        || (total_size == std::ptr::null_mut())
        || (avail_size == std::ptr::null_mut())
    {
        return Los_Errno_Memcheck_Para_Null!();
    }

    end_pool = (pool as *mut u8).wrapping_add((*pool_info).pool_size.try_into().unwrap());
    if !Os_Mem_Middle_Addr_Open_End!(pool, ptr, end_pool) {
        return Los_Errno_Memcheck_Outside!();
    }

    if g_mem_check_level == Los_Mem_Check_Level_High!() {
        head = Os_Mem_Find_Node_Ctrl(pool, ptr);
        if (head == std::ptr::null_mut())
            || (Os_Mem_Node_Get_Size!((*(head as *mut LosMemDynNode))
                .self_node
                .size_and_flag
                .get())
                < (ptr as u32 - head as u32))
        {
            return Los_Errno_Memcheck_No_Head!();
        }
        *total_size = Os_Mem_Node_Get_Size!(
            (*(head as *mut LosMemDynNode))
                .self_node
                .size_and_flag
                .get()
                - std::mem::size_of::<LosMemDynNode>() as u32
        );
        *avail_size = Os_Mem_Node_Get_Size!(
            (*(head as *mut LosMemDynNode))
                .self_node
                .size_and_flag
                .get()
                - (ptr as u32 - head as u32)
        );

        return LOS_NOK;
    }
    if g_mem_check_level == Los_Mem_Check_Level_Low!() {
        if ptr != Os_Mem_Align!(ptr, Os_Mem_Align_Size!()) as *mut std::ffi::c_void {
            return Los_Errno_Memcheck_No_Head!();
        }
        head = (ptr as u32 - std::mem::size_of::<LosMemDynNode>() as u32) as *mut std::ffi::c_void;
        if Os_Mem_Magic_Valid!((*(head as *mut LosMemDynNode))
            .self_node
            .myunion
            .extend_field
            .magic
            .get())
        {
            *total_size = Os_Mem_Node_Get_Size!(
                (*(head as *mut LosMemDynNode))
                    .self_node
                    .size_and_flag
                    .get()
                    - std::mem::size_of::<LosMemDynNode>() as u32
            );
            *avail_size = Os_Mem_Node_Get_Size!(
                (*(head as *mut LosMemDynNode))
                    .self_node
                    .size_and_flag
                    .get()
                    - std::mem::size_of::<LosMemDynNode>() as u32
            );
            return LOS_OK;
        } else {
            return Los_Errno_Memcheck_No_Head!();
        }
    }

    Los_Errno_Memcheck_Wrong_Level!()
}

//#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
fn Os_Mem_Find_Node_Ctrl(
    pool: *mut std::ffi::c_void,
    ptr: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    let head: *mut std::ffi::c_void = ptr;

    if ptr == std::ptr::null_mut() {
        return std::ptr::null_mut();
    }

    head = Os_Mem_Align!(head, Os_Mem_Align_Size!()) as *mut std::ffi::c_void;
    while !Os_Mem_Magic_Valid!((*(head as *mut LosMemDynNode))
        .self_node
        .myunion
        .extend_field
        .magic
        .get())
    {
        head = ((head as *mut u8).wrapping_sub(std::mem::size_of::<*mut char>()))
            as *mut std::ffi::c_void;
        if head <= pool {
            return std::ptr::null_mut();
        }
    }

    head
}

//#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
fn Los_Mem_Check_Level_Set(check_level: u8) -> u32 {
    //low 0
    if check_level == Los_Mem_Check_Level_Low!() {
        println!(
            "{}: LOS_MEM_CHECK_LEVEL_LOW \n",
            std::any::type_name::<fn()>()
        );
    }
    //high 1
    else if check_level == Los_Mem_Check_Level_High!() {
        println!(
            "{}: LOS_MEM_CHECK_LEVEL_HIGH \n",
            std::any::type_name::<fn()>()
        );
    } else if check_level == Los_Mem_Check_Level_Disable!() {
        println!(
            "{}: LOS_MEM_CHECK_LEVEL_DISABLE \n",
            std::any::type_name::<fn()>()
        );
    } else {
        println!(
            "{}: wrong param, setting failed !! \n",
            std::any::type_name::<fn()>()
        );
        return Los_Errno_Memcheck_Wrong_Level!();
        /////
    }
    g_mem_check_level = check_level;

    LOS_OK
}

//#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
fn Los_Mem_Check_Level_Get() -> u8 {
    g_mem_check_level
}

//[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
fn Os_Mem_Sys_Node_Check(
    dst_addr: *mut std::ffi::c_void,
    src_addr: *mut std::ffi::c_void,
    node_length: u32,
    pos: u8,
) -> u32 {
    let mut ret: u32;
    let mut total_size: u32 = 0;
    let mut avail_size: u32 = 0;
    let pool: *mut u8 = m_auc_sys_mem1;

    //#[cfg(feature = "LOSCFG_EXC_INTERACTION")]
    {
        if (dst_addr as u32) < (m_auc_sys_mem0 as u32 + g_exc_interact_mem_size as u32) {
            pool = m_auc_sys_mem0;
        }
    }

    ret = Los_Mem_Node_Size_Check(
        pool as *mut std::ffi::c_void,
        dst_addr,
        &mut total_size,
        &mut avail_size,
    );
    if (ret == LOS_OK) && (node_length > avail_size) {
        println!("---------------------------------------------\n{}: dst inode availSize is not enough availSize = 0x{:x}, memcpy length = 0x{:x}\n",if pos == 0 { "memset" } else { "memcpy" }, avail_size, node_length);
        //Os_Back_Trace();
        println!("---------------------------------------------\n");
        return LOS_NOK;
    }

    if pos == u8::MAX {
        //#[cfg(feature = "LOSCFG_EXC_INTERACTION")]
        {
            if (src_addr as u32) < (m_auc_sys_mem0 as u32 + g_exc_interact_mem_size as u32) {
                pool = m_auc_sys_mem0;
            } else {
                pool = m_auc_sys_mem1;
            }
        }
        ret = Los_Mem_Node_Size_Check(
            pool as *mut std::ffi::c_void,
            src_addr,
            &mut total_size,
            &mut avail_size,
        );
        if (ret == LOS_OK) && (node_length > avail_size) {
            println!("---------------------------------------------\n");
            println!("memcpy: src inode availSize is not enough availSize = 0x{:x}, memcpy length = 0x{:x}\n", avail_size, node_length);
            //OsBackTrace();
            println!("---------------------------------------------\n");
            return LOS_NOK;
        }
    }

    LOS_OK
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Mod_Check(moduleid: u32) -> u32 {
    if moduleid > Mem_Module_Max!() {
        println!("error module ID input!\n");
        return LOS_NOK;
    }
    return LOS_OK;
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Ptr_To_Node(ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    let mut gapsize: u32;
    if ((ptr as u32) & ((Os_Mem_Align_Size!() - 1) as u32)) != 0 {
        println!(
            "[{}:{}]ptr:{:p} not align by 4byte\n",
            std::any::type_name::<fn()>(),
            line!(),
            ptr
        );
        return std::ptr::null_mut();
    }

    gapsize = *((ptr as u32 - std::mem::size_of::<u32>() as u32) as *mut u32);
    if Os_Mem_Node_Get_Aligned_Flag!(gapsize) && Os_Mem_Node_Get_Used_Flag!(gapsize) {
        println!(
            "[{}:{}]gapSize:0x{:x} error\n",
            std::any::type_name::<fn()>(),
            line!(),
            gapsize
        );
        return std::ptr::null_mut();
    }

    if Os_Mem_Node_Get_Aligned_Flag!(gapsize) {
        gapsize = Os_Mem_Node_Get_Aligned_GapSize!(gapsize);
        if ((gapsize & (Os_Mem_Align_Size!() - 1) as u32) != 0)
            || (gapsize > ((ptr as u32) - Os_Mem_Node_Head_Size!()))
        {
            println!(
                "[{}:{}]gapSize:0x{:x} error\n",
                std::any::type_name::<fn()>(),
                line!(),
                gapsize
            );
            return std::ptr::null_mut();
        }

        ptr = ((ptr as u32) - gapsize) as *mut std::ffi::c_void;
    }

    (ptr as u32 - Os_Mem_Node_Head_Size!() as u32) as *mut std::ffi::c_void
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Node_Size_Get(ptr: *mut std::ffi::c_void) -> u32 {
    let node: *mut LosMemDynNode = Os_Mem_Ptr_To_Node(ptr) as *mut LosMemDynNode;
    if node == std::ptr::null_mut() {
        return 0;
    }

    return Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get());
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Los_Mem_M_Alloc(pool: *mut std::ffi::c_void, size: u32, moduleid: u32) -> *mut std::ffi::c_void {
    let mut int_save: u32;
    let ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let node: *mut std::ffi::c_void = std::ptr::null_mut();
    if Os_Mem_Mod_Check(moduleid) == LOS_NOK {
        return std::ptr::null_mut();
    }
    ptr = Los_Mem_Alloc(pool, size); //1500
    if ptr != std::ptr::null_mut() {
        Mem_Lock!(int_save);
        g_module_mem_used_size[moduleid as usize] =
            g_module_mem_used_size[moduleid as usize] + Os_Mem_Node_Size_Get(ptr);
        node = Os_Mem_Ptr_To_Node(ptr);
        if node != std::ptr::null_mut() {
            Os_Mem_Modid_Set(node as *mut LosMemDynNode, moduleid); //100
        }
        Mem_Unlock!(int_save);
    }

    ptr
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Los_Mem_M_Alloc_Align(
    pool: *mut std::ffi::c_void,
    size: u32,
    boundary: u32,
    moduleid: u32,
) -> *mut std::ffi::c_void {
    let mut int_save: u32;
    let ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let node: *mut std::ffi::c_void = std::ptr::null_mut();
    if Os_Mem_Mod_Check(moduleid) == LOS_NOK {
        return std::ptr::null_mut();
    }
    ptr = Los_Mem_Alloc_Align(pool, size, boundary); //1500
    if ptr != std::ptr::null_mut() {
        Mem_Lock!(int_save);
        g_module_mem_used_size[moduleid as usize] =
            g_module_mem_used_size[moduleid as usize] + Os_Mem_Node_Size_Get(ptr);
        node = Os_Mem_Ptr_To_Node(ptr);
        if node != std::ptr::null_mut() {
            Os_Mem_Modid_Set(node as *mut LosMemDynNode, moduleid); //100
        }
        Mem_Unlock!(int_save);
    }

    ptr
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Los_Mem_M_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, moduleid: u32) -> u32 {
    let mut int_save: u32;
    let mut ret: u32;
    let mut size: u32;
    let node: *mut LosMemDynNode = std::ptr::null_mut();

    if (Os_Mem_Mod_Check(moduleid) == LOS_NOK)
        || (ptr == std::ptr::null_mut())
        || (pool == std::ptr::null_mut())
    {
        return LOS_NOK;
    }

    node = Os_Mem_Ptr_To_Node(ptr) as *mut LosMemDynNode;
    if node == std::ptr::null_mut() {
        return LOS_NOK;
    }

    size = Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get());

    if moduleid != Os_Mem_Modid_Get(node) {
        println!(
            "node[{:p}] alloced in module {}, but free in module {}\n node's taskId: 0x{:x}\n",
            ptr,
            Os_Mem_Modid_Get(node),
            moduleid,
            Os_Mem_Taskid_Get(node)
        );
        moduleid = Os_Mem_Modid_Get(node);
    }

    ret = LOS_MemFree(pool, ptr);
    if ret == LOS_OK {
        Mem_Lock!(int_save);
        g_module_mem_used_size[moduleid as usize] =
            g_module_mem_used_size[moduleid as usize] - size;
        Mem_Unlock!(int_save);
    }
    return ret;
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Los_Mem_M_Realloc(
    pool: *mut std::ffi::c_void,
    ptr: *mut std::ffi::c_void,
    size: u32,
    moduleid: u32,
) -> *mut std::ffi::c_void {
    let new_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut old_node_size: u32;
    let mut int_save: u32;
    let node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut old_module_id = moduleid;
    let mut temp: u32;
    if (Os_Mem_Mod_Check(moduleid) == LOS_NOK) || (pool == std::ptr::null_mut()) {
        return std::ptr::null_mut();
    }

    if ptr == std::ptr::null_mut() {
        return LOS_Mem_M_Alloc(pool, size, moduleid);
    }

    node = Os_Mem_Ptr_To_Node(ptr) as *mut LosMemDynNode;
    if node == std::ptr::null_mut() {
        return std::ptr::null_mut();
    }

    if moduleid != Os_Mem_Modid_Get(node) {
        println!(
            "a node[{:p}] alloced in module {}, but realloc in module {}\n node's taskId: {}\n",
            ptr,
            Os_Mem_Modid_Get(node),
            moduleid,
            Os_Mem_Taskid_Get(node)
        );
        old_module_id = Os_Mem_Modid_Get(node);
    }

    if size == 0 {
        temp = Los_Mem_M_Free(pool, ptr, old_module_id);
        return std::ptr::null_mut();
    }

    old_node_size = Os_Mem_Node_Size_Get(ptr);
    new_ptr = Los_Mem_Realloc(pool, ptr, size);
    if new_ptr != std::ptr::null_mut() {
        Mem_Lock!(int_save);
        g_module_mem_used_size[moduleid as usize] =
            g_module_mem_used_size[moduleid as usize] + Os_Mem_Node_Size_Get(new_ptr);
        g_module_mem_used_size[old_module_id as usize] =
            g_module_mem_used_size[old_module_id as usize] - old_node_size;
        node = Os_Mem_Ptr_To_Node(new_ptr) as *mut LosMemDynNode;
        Os_Mem_Modid_Set(node, moduleid);
        Mem_Unlock!(int_save);
    }
    return new_ptr;
}

//#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Los_Mem_M_Used_Get(moduleid: u32) -> u32 {
    if Os_Mem_Mod_Check(moduleid) == LOS_NOK {
        return Os_Null_Int!();
    }
    g_module_mem_used_size[moduleid as usize]
}
