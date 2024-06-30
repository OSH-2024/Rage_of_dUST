#[allow(unused_macros)]
macro_rules! print_err {
    ($fmt:expr $(, $($arg:tt)+)?) => {
        {
            eprint!("[ERR] ");
            eprint!($fmt $(, $($arg)+)?);
        }
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
static mut m_auc_sys_mem0: *mut u8=0 as *mut u8;
static mut m_auc_sys_mem1: *mut u8=0 as *mut u8;

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

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]//MEM_MODULE_MAX=0x20
static mut g_module_mem_used_size: [u32; 0x20 + 1] = [0; 0x20 + 1];

#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Node_Save(node: &mut LosMemDynNode) {
    // TODO
}

#[inline]
fn os_mem_taskid_set(node: &mut LosMemDynNode, task_id: u32) {
    node.self_node.taskId = task_id;

    #[cfg(LOSCFG_MEM_HEAD_BACKUP)]
    {
        Os_Mem_Node_Save(node);
    }
}

pub const fn Os_Mem_Taskid_Get(node: &LosMemDynNode) -> u32 {
    node.self_node.Myunion.extend_field.taskid
}

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
mod mem_mul_module {
    use crate::LosMemDynNode;

    #[inline]
    pub fn Os_Mem_Modid_Set(node: &mut LosMemDynNode, module_id: u32) {
        node.selfNode.moduleId = module_id;

        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        {
            Os_Mem_Node_Save(node);
        }
    }

    #[inline]
    pub fn Os_Mem_Modid_Get(node: &LosMemDynNode) -> u32 {
        node.selfNode.moduleId
    }

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    fn Os_Mem_Node_Save(node: &mut LosMemDynNode) {
        // TODO
    }
}

#[cfg(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT"))]
mod mem_debug_task_stat {
    use crate::{LosMemDynNode, LosTaskCB, OsCurrTaskGet, OS_INT_INACTIVE, TASK_NUM};

    #[inline]
    pub fn os_mem_set_magic_num_and_task_id(node: &mut LosMemDynNode) {
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
    fn os_mem_set_magic(node: &mut LosMemDynNode) {
        // TODO
    }

    #[inline]
    fn os_mem_taskid_set(node: &mut LosMemDynNode, task_id: u32) {
        node.self_node.taskid = task_id;
    }
}

#[cfg(not(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT")))]
mod mem_debug_task_stat {
    use crate::LosMemDynNode;

    #[inline]
    pub fn os_mem_set_magic_num_and_task_id(_node: &mut LosMemDynNode) {}
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
fn Os_Mem_Disp_Ctl_Node(ctl_node: &LosMemCtlNode) {
    let mut checksum: u32;

    checksum = Os_Mem_Node_Checksum_Calculate!(ctl_node);

    println!("node:{:p} checksum={:p}[{:p}] freeNodeInfo.pstPrev={:p} "
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
unsafe fn Os_Mem_Disp_More_Details(node: &mut LosMemDynNode) {
    let task_id: u32;
    //
    println!("************************************************");
    Os_Mem_Disp_Ctl_Node(&node.self_node);
    println!("the address of node :{:p}", &node);

    if !Os_Mem_Node_Get_Used_Flag(node.self_node.size_and_flag) {
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
    if (task_cb.tsk_status & 0x0001U)
        || (task_cb.task_entry == None)
        || (task_cb.task_name == None)
    //OS_TASK_STATUS_UNUSED=0x0001U
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
unsafe fn Os_Mem_Disp_Wild_Pointer_Msg(node: &LosMemDynNode, ptr: *mut std::ffi::c_void) {
    println!("************************************************");
    println!(
        "find an control block at: {:p}, gap size: 0x{:x}, sizeof(LosMemDynNode): 0x{:x}",
        node,
        (*node).self_node.gapsize,
        sizeof(LosMemDynNode)
    );
    println!(
        "the pointer should be: {:p}",
        (node as u32 + (*node).self_node.gap_size + std::mem::size_of::<T>(LosMemDynNode))
    );
    println!("the pointer given is: {:p}", ptr);
    println!("PROBABLY A WILD POINTER");
    Os_Back_Trace(); //TODO
    println!("************************************************");
}

//237行
fn Os_Mem_Checksum_Set(ctl_node: &LosMemDynNode) {
    ctl_node.checksum = Os_Mem_Node_Checksum_Calculate!(ctl_node);
}

//242行
fn Os_Mem_Checksum_Verify(ctl_node: &LosMemDynNode) -> bool {
    ctl_node.checksum == Os_Mem_Node_Checksum_Calculate!(ctl_node)
}

//247行
unsafe fn Os_Mem_Backup_Setup(node: &LosMemDynNode) {
    let node_pre: *mut LosMemDynNode = &node.self_node.prenode;
    //此处有!=NULL的判断，由于rust中不允许空指针，故省略
    (*node_pre).backup_node.Myunion.free_node_info.pst_next =
        node.self_node.free_node_info.pst_next;
    (*node_pre).backup_node.Myunion.free_node_info.pst_prev =
        node.self_node.free_node_info.pst_prev;
    (*node_pre).backup_node.prenode = node.self_node.prenode;
    (*node_pre).backup_node.checksum = node.self_node.checksum;
    (*node_pre).backup_node.gapsize = node.self_node.gapsize;
    (*node_pre).backup_node.size_and_flag = node.self_node.size_and_flag;
}

//260
unsafe fn Os_Mem_Node_Next_Get(pool:*std::ffi::c_void,node:&LosMemDynNode)->*mut LosMemDynNode{
    pool_info: *LosMemPoolInfo = pool as *LosMemPoolInfo;

    if node==Os_Mem_End_Node!(pool,(*pool_info).pool_size)
    {
        return Os_Mem_First_Node!(pool);
    }
    else
    {
        return Os_Mem_Next_Node!(node);
    }
}

//271
fn Os_Mem_Backup_Setup_4_Next(pool:*std::ffi::c_void,node:&LosMemDynNode)->u32{
    let node_next:*mut LosMenDynNode=Os_Mem_Node_Next_Get(pool,node);

    if !Os_Mem_Checksum_Vertify((*node_next).self_node)
    {
        println!("[//TODO:function name]the next node is broken!!");
        Os_Mem_Disp_Ctl_Node(&(*node_next).self_node);
        println!("Current node details:");
        Os_Mem_Disp_More_Details(node);

        return 1;//LOS_NOK
    }

    if !Os_Mem_Checksum_Vertify(&(*node).backup_node)
    {
        node.backup_node.Myunion.free_node_info.pst_next =
            (*node_next).self_node.free_node_info.pst_next;
        node.backup_node.Myunion.free_node_info.pst_prev =
            (*node_next).self_node.free_node_info.pst_prev;
        node.backup_node.prenode = (*node_next).self_node.prenode;
        node.backup_node.checksum = (*node_next).self_node.checksum;
        node.backup_node.gapsize = (*node_next).self_node.gapsize;
        node.backup_node.size_and_flag = (*node_next).self_node.size_and_flag;
    }
    return 0;//LOS_OK
}

//295
fn Os_Mem_Backup_Do_Restore(pool:*std::ffi::c_void,node_pre:&LosMemDynNode,node:&mut LosMemDynNode){

    //省略了node!=NULL的判断

    println!("the backup node information of current node in previous node:");
    Os_Mem_Disp_Ctl_Node(&node_pre.backup_node);
    println!("the detailed information of previous node:");
    Os_Mem_Disp_More_Details(node_pre);

    node.self_node.free_node_info.pst_next = (*node_pre).backup_node.Myunion.free_node_info.pst_next;
    node.self_node.free_node_info.pst_prev = (*node_pre).backup_node.Myunion.free_node_info.pst_prev;
    node.self_node.prenode = node_pre.backup_node.prenode;
    node.self_node.checksum = node_pre.backup_node.checksum;
    node.self_node.gapsize = node_pre.backup_node.gapsize;
    node.self_node.size_and_flag = node_pre.backup_node.size_and_flag;

    /* we should re-setup next node's backup on current node */
    return Os_Mem_Backup_Setup_4_Next(pool, node);
}

//317
fn Os_Mem_First_Node_PrevGet(pool_info:&mut LosMemPoolInfo)->*mut LosMemDynNode{
    let node_pre:*mut LosMemDynNode=Os_Mem_END_Node!(pool_info,pool_info.pool_size);

    if !Os_Mem_Checksum_Verify(&(*node_pre).self_node)
    {
        println!("the current node is THE FIRST NODE !");
        println!("[//TODO:function name]: the node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&(*node_pre).self_node);
    }
    if !Os_Mem_Checksum_Verify(&(*node_pre).backup_node)
    {
        println!("the current node is THE FIRST NODE !");
        println!("[//TODO:function name]: the backup node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&(*node_pre).backup_node);
    }

    return node_pre;
}

//337
unsafe fn Os_Mem_Node_Prev_Get(pool:*std::ffi::c_void,node:&mut LosMemDynNode)->*mut LosMemDynNode{
    let node_cur:*mut LosMemDynNode=Os_Mem_First_Node!(pool);
    // let node_pre:*mut LosMemDynNode=Os_Mem_First_Node_PrevGet(pool);
    pool_info:*mut LosMemPoolInfo=pool as *mut LosMemPoolInfo;

    if node == Os_Mem_First_Node!(pool)
    {
        return Os_Mem_First_Node_PrevGet(pool_info);
    }

    while node_cur<Os_Mem_End_Node!(pool,(*pool_info).pool_size)
    {
        if !Os_Mem_Checksum_Verify(&(*node_cur).self_node)
        {
            println!("[//TODO:function name]the node information of current node is bad !!");
            Os_Mem_Disp_Ctl_Node(&(*node_cur).self_node);

            if node_pre.is_null()
            {
                return ptr::null_mut(); 
            }
            
            println!("the detailed information of previous node:");
            Os_Mem_Disp_More_Details(node_pre);
        }

        if !Os_Mem_Checksum_Verify(&(*node_cur).backup_node)
        {
            println!("[//TODE:function name]the backup node information of current node is bad !!");    
            Os_Mem_Disp_Ctl_Node(&(*node_cur).backup_node);
            
            if node_pre.is_null()
            {
                println!("the detailed information of previous node:");
                Os_Mem_Disp_More_Details(node_pre);
            }

            if(Os_Mem_Backup_Setup_4_Next(pool,node_cur)!=0)//LOS_NOK
            {
                return ptr::null_mut();
            }
        }

        if Os_Mem_Next_Node!(node_cur)==node
        {
            return node_cur;
        }

        if Os_Mem_Next_Node!(node_cur)>node
        {
            break;
        }

        node_pre=node_cur;

        node_cur=Os_Mem_Next_Node!(node_cur);
    }

    return ptr::null_mut();
}

//395
unsafe fn Os_Mem_Node_Prev_Try_Get(pool:*mut std::ffi::c_void,)