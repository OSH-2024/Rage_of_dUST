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
        "p15, "#$Op1", %0, "#$CRn","#$CRm","#$Op2
    };
}
//regs.h 139
macro_rules! Tpidrprw
{
    () => {
        CP15_Reg(c13, 0, c0, 4)
    };
}
//task.h 64
fn Arch_Curr_Task_Get() -> *mut std::ffi::c_void{
    (Arm_Sysreg_Read!(Tpidrprw!()) as u32) as *mut std::ffi::c_void
}
//los_task_pri.h 186
fn Os_Curr_Task_Get() -> *mut LosTaskCB {
    Arch_Curr_Task_Get() as *mut LosTaskCB
}
//los_exc.c中定义，在los_memory.c 233行调用
fn Los_Back_Trace(){
    [#cfg(feature="LOSCFG_BACKTRACE")]
    let run_task: *mut LosTaskCB = Os_Curr_Task_Get();
    printl
}
macro_rules! Os_Back_Trace {
    () => {
        Los_Back_Trace()
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

#[cfg(feature = "LOSCFG_EXC_INTERACTION")]
#[link_section = ".data.init"]
static mut g_exc_interact_mem_size: usize = 0;

#[cfg(feature = "LOSCFD_BASE_MEM_NODE_SIZE_CHECK")]
static mut g_mem_check_level: u8 = 0xff; //LOS_MEM_CHECK_LEVEL_DEFAULT

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")] //MEM_MODULE_MAX=0x20
static mut g_module_mem_used_size: [u32; 0x20 + 1] = [0; 0x20 + 1];

#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Node_Save(node: *mut LosMemDynNode) {
    // TODO
}

//94
fn os_mem_taskid_set(node: *mut LosMemDynNode, task_id: u32) {
    (*node).self_node.myunion.extend_field.taskid.set(task_id);

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(node);
    }
}

pub const fn Os_Mem_Taskid_Get(node: *mut LosMemDynNode) -> u32 {
    (*node).self_node.myunion.extend_field.taskid.get()
}
//110
#[inline]
#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Modid_Set(node: *mut LosMemDynNode, module_id: u32) {
    (*node).selfNode.myunion.extend_field.moduleid = module_id;
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(node);
    }
}

//119
#[inline]
#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Modid_Get(node: *mut LosMemDynNode) -> u32 {
    (*node).selfNode.myunion.extend_field.moduleid as u32
}

#[cfg(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT"))]
mod mem_debug_task_stat {
    use crate::{LosMemDynNode, LosTaskCB, OsCurrTaskGet, OS_INT_INACTIVE, TASK_NUM};

    #[inline]
    pub fn os_mem_set_magic_num_and_task_id(node: *mut LosMemDynNode) {
        if let Some(run_task) = OsCurrTaskGet() {
            os_mem_set_magic(node);
            if OS_INT_INACTIVE {
                os_mem_taskid_set(node, run_task.taskid);
            } else {
                os_mem_taskid_set(node, TASK_NUM - 1);
            }
        }
    }

    #[inline]
    fn os_mem_set_magic(node: *mut LosMemDynNode) {
        // TODO
    }

    #[inline]
    fn os_mem_taskid_set(node: *mut LosMemDynNode, task_id: u32) {
        (*node).self_node.taskid = task_id;
    }
}

#[cfg(not(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT")))]
mod mem_debug_task_stat {
    use crate::LosMemDynNode;

    #[inline]
    pub fn os_mem_set_magic_num_and_task_id(_node: *mut LosMemDynNode) {}
}

//151行
//rust函数定义顺序不影响其可见性和使用性，原文在此处提前声明2098行的OsMemFindNodeCtrl，rust中不需要
//154行
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
const CHECKSUM_MAGICNUM: u32 = 0xDEADBEEF;

macro_rules! Os_Mem_Node_Checksum_Calculate {
    ($ctl_node:expr) => {
        let ctl_node = $ctl_node;
        ((ctl_node.free_node_info.pst_prev)
            ^ (ctl_node.free_node_info.pst_next)
            ^ (ctl_node.pre_node)
            ^ ctlNode.gap_size
            ^ ctlNode.size_and_flag
            ^ CHECKSUM_MAGICNUM)
    };
}

//164行
fn Os_Mem_Disp_Ctl_Node(ctl_node: *mut LosMemCtlNode) {
    let mut checksum: u32;

    checksum = Os_Mem_Node_Checksum_Calculate!(ctl_node);

    println!("node:{:p} checksum={:p}[{:p}] freeNodeInfo.pstPrev={:p} ",
           "freeNodeInfo.pstNext={:p} preNode={:p} gapSize=0x{:x} sizeAndFlag=0x{:x}",
           ctl_node,
           ctl_node.checksum,
           checksum,
           ctl_node->free_node_info.pst_prev,
           ctl_node->free_node_info.pst_next,
           ctl_node->pre_node,
           ctl_node->gap_size,
           ctl_node->size_and_flag);
}

//182行
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
unsafe fn Os_Mem_Disp_Wild_Pointer_Msg(node: *mut LosMemDynNode, ptr: *mut std::ffi::c_void) {
    println!("************************************************");
    println!(
        "find an control block at: {:p}, gap size: 0x{:x}, sizeof(LosMemDynNode): 0x{:x}",
        node,
        (*node).self_node.gapsize,
        std::mem::size_of::<LosMemDynNode>() as u32
    );
    println!(
        "the pointer should be: {:p}",
        node.offset((*node).self_node.gap_size + std::mem::size_of::<LosMemDynNode>() as u32)
    );
    println!("the pointer given is: {:p}", ptr);
    println!("PROBABLY A WILD POINTER");
    Os_Back_Trace!(); //TODO
    println!("************************************************");
}

//237行
fn Os_Mem_Checksum_Set(ctl_node: *mut LosMemCtlNode) {
    (*ctl_node).checksum = Os_Mem_Node_Checksum_Calculate!(ctl_node);
}

//242行
fn Os_Mem_Checksum_Verify(ctl_node: *mut LosMemCtlNode) -> bool {
    (*ctl_node).checksum == Os_Mem_Node_Checksum_Calculate!(ctl_node)
}

//247行
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
fn Os_Mem_Backup_Setup_4_Next(pool: *mut std::ffi::c_void, node: *mut LosMemDynNode) -> u32 {
    let node_next: *mut LosMenDynNode = Os_Mem_Node_Next_Get(pool, node);

    if !Os_Mem_Checksum_Verify((*node_next).self_node) {
        println!("[//TODO:function name]the next node is broken!!");
        Os_Mem_Disp_Ctl_Node(&(*node_next).self_node);
        println!("Current node details:");
        Os_Mem_Disp_More_Details(node);

        return 1; //LOS_NOK
    }

    if !Os_Mem_Checksum_Verify(&(*node).backup_node) {
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
fn Os_Mem_First_Node_PrevGet(pool_info: *mut LosMemPoolInfo) -> *mut LosMemDynNode {
    let node_pre: *mut LosMemDynNode = Os_Mem_END_Node!(pool_info, pool_info.pool_size);

    if !Os_Mem_Checksum_Verify(&(*node_pre).self_node) {
        println!("the current node is THE FIRST NODE !");
        println!("[//TODO:function name]: the node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&(*node_pre).self_node);
    }
    if !Os_Mem_Checksum_Verify(&(*node_pre).backup_node) {
        println!("the current node is THE FIRST NODE !");
        println!("[//TODO:function name]: the backup node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&(*node_pre).backup_node);
    }

    return node_pre;
}

//337
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

        if !Os_Mem_Checksum_Verify(&(*node_cur).backup_node) {
            println!("[//TODE:function name]the backup node information of current node is bad !!");
            Os_Mem_Disp_Ctl_Node(&(*node_cur).backup_node);

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

        if !Os_Mem_Checksum_Verify(&(*node_cur).backup_node) {
            println!("[//TODO:function name]the backup node information of current node is bad !!");
            Os_Mem_Disp_Ctl_Node(&(*node_cur).backup_node);

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
            + (*node_cur).self_node.gapsize
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
unsafe fn Os_Mem_Backup_Restore(pool: *mut std::ffi::c_void, node: *mut LosMemDynNode) -> u32 {
    let node_pre: *mut LosMemDynNode = Os_Mem_Node_Prev_Get(pool, node);

    if node_pre.is_null() {
        return 1; //LOS_NOK
    }

    return Os_Mem_Backup_Do_Restore(pool, node_pre as *mut LosMemDynNode, node);
}

//471
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
        if !Os_Mem_Checksum_Verify(&mut (*node).self_nodeas *mut LosMemCtlNode) {
            node = (ptr as u32 - Os_Mem_Node_Head_Size!()) as *mut LosMemDynNode;
            return Os_Mem_Backup_Try_Restore(pool, &mut node as *mut *mut LosMemDynNode, ptr);
        }
    }
    return 0; //LOS_OK
}

//487
unsafe fn Os_Mem_Set_Gap_Size(ctl_node: *mut LosMemCtlNode, gap_size: u32) {
    (*ctl_node).gap_size = gap_size;
}

//492
unsafe fn Os_Mem_Node_Save(node: *mut LosMemDynNode) {
    Os_Mem_Set_Gap_Size(&mut (*node).self_node as *mut LosMemCtlNode, 0);
    Os_Mem_Checksum_Set(&mut (*node).self_node as *mut LosMemCtlNode);
    Os_Mem_Backup_Setup(node);
}

//499
unsafe fn Os_Mem_Node_Save_With_Gap_Size(node: *mut LosMemDynNode, gap_size: u32) {
    Os_Mem_Set_Gap_Size(&mut (*node).self_node as *mut LosMemCtlNode, gap_size);
    Os_Mem_Checksum_Set(&mut (*node).self_node as *mut LosMemCtlNode);
    Os_Mem_Backup_Setup(node);
}

//506
unsafe fn Os_Mem_List_Delete(node: *mut LOS_DL_LIST, first_node: *mut std::ffi::c_void) {
    let dyn_node: *mut LosMemDynNode = std::ptr::null_mut();

    (*(*node).pst_next).pst_prev = (*node).pst_prev;
    (*(*node).pst_prev).pst_next = (*node).pst_next;

    if (*node).pst_next as std::ffi::c_void >= first_node {
        dyn_node = Los_Dl_List_Entry!((*node).pst_next, LosMemDynNode, self_node.free_node_info);
        Os_Mem_Node_Save(dyn_node);
    }
    if (*node).pst_prev as std::ffi::c_void >= first_node {
        dyn_node = Los_Dl_List_Entry!((*node).pst_prev, LosMemDynNode, self_node.free_node_info);
        Os_Mem_Node_Save(dyn_node);
    }

    (*node).pst_next = std::ptr::null_mut();
    (*node).pst_prev = std::ptr::null_mut();

    dyn_node = Los_Dl_List_Entry!(node, LosMemDynNode, self_node.free_node_info);
    Os_Mem_Node_Save(dyn_node);
}

//530
unsafe fn Os_MEM_List_Add(
    list_node: *mut LOS_DL_LIST,
    node: *mut LOS_DL_LIST,
    first_node: *mut std::ffi::c_void,
) {
    let dyn_node: *mut LosMemDynNode = std::ptr::null_mut();

    (*node).pst_next = (*list_node).pst_next;
    (*node).pst_prev = list_node;

    dyn_node = Los_Dl_List_Entry!(node, LosMemDynNode, self_node.free_node_info);
    Os_Mem_Node_Save(dyn_node);

    (*(*list_node).pst_next).pst_prev = node;
    if (*list_node).pst_next as std::ffi::c_void >= first_node {
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
