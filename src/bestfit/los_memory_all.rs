include!("los_memory_h.rs");

static mut g_int_count: [u32; 1] = [0;1];
static mut g_task_cb_array: *mut LosTaskCB = 0 as *mut LosTaskCB;
macro_rules! Arm_Sysreg_Read {
    ($reg:expr) => {{
        let mut val: u32;
        unsafe {
            asm!("mrc $0, 0, $1, c15, c0, 0"
                : "=&r"(val)
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

macro_rules! Os_Tcb_From_Tid{
    ($task_id:expr)=>{
        (g_task_cb_array as *mut LosTaskCB).offset($task_id as isize)
    };
}


//los_hwi.c 353
fn Int_Active() -> u32 {
    let mut int_count: u32;
    let mut int_save: u32 = Los_Int_Lock();
    int_count = g_int_count[0];
    Los_Int_Restore(int_save);
    int_count
}

//los_hwi.h 64
macro_rules! Os_Int_Active {
    () => {
        Int_Active()
    };
}

//los_hwi.h 74
macro_rules! Os_Int_Inactive {
    () => {
        !Os_Int_Active!() != 0
    };
}

static mut m_auc_sys_mem0: *mut u8 = 0 as *mut u8;
static mut m_auc_sys_mem1: *mut u8 = 0 as *mut u8;

type MallocHook = fn();
static mut g_malloc_hook: Option<MallocHook> = None;

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
    (*node)
        .self_node
        .myunion
        .extend_field
        .moduleid
        .set(module_id);
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(node);
    }
}

//119
#[inline]
#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Modid_Get(node: *mut LosMemDynNode) -> u32 {
    (*node).self_node.myunion.extend_field.moduleid.get()
}

//130
#[inline]
fn Os_Mem_Set_Magic_Num_And_Task_Id(node: *mut LosMemDynNode) {
    #[cfg(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT"))]
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

#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
macro_rules! Checksum_Magicnum {
    () => {
        0xDEADBEEF
    };
}

#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
macro_rules! Os_Mem_Node_Checksum_Calculate {
    ($ctl_node:expr) => {
        ((((*$ctl_node).myunion.free_node_info.pst_prev) as u32)
            ^ (((*$ctl_node).myunion.free_node_info.pst_next) as u32)
            ^ (((*$ctl_node).prenode) as u32)
            ^ ((*$ctl_node).gapsize.get())
            ^ ((*$ctl_node).size_and_flag.get())
            ^ Checksum_Magicnum!())
    };
}

//164行
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Disp_Ctl_Node(ctl_node: *mut LosMemCtlNode) {
    let mut checksum: u32;

    checksum = Os_Mem_Node_Checksum_Calculate!(ctl_node);

    println!(
        "node:{:p} checksum={:x}[{:x}] freeNodeInfo.pstPrev={:p}
        freeNodeInfo.pstNext={:p} preNode={:p} gapSize=0x{:x} sizeAndFlag=0x{:x}",
        ctl_node,
        (*ctl_node).checksum.get(),
        checksum,
        (*ctl_node).myunion.free_node_info.pst_prev,
        (*ctl_node).myunion.free_node_info.pst_next,
        (*ctl_node).prenode,
        (*ctl_node).gapsize.get(),
        (*ctl_node).size_and_flag.get()
    );
}

//182行
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
    if (((*task_cb).task_status & 0x0001u16) != 0)
        || ((*task_cb).task_name == std::ptr::null_mut())
    //OS_TASK_STATUS_UNUSED=0x0001U，taskStatus为UINT16类型
    {
        println!("The task [ID:0x{:x}] is NOT CREATED(ILLEGAL)", task_id);
        println!("************************************************\n");
        return;
    }

    println!(
        "allocated by task: {:?} [ID = 0x{:x}]\n",
        (*task_cb).task_name, task_id
    );
    #[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
    println!("allocted by moduleId:{}", Os_Mem_Modid_Get(node));

    println!("************************************************\n");
}

//224行
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
        node.offset(
            ((*node).self_node.gapsize.get() + std::mem::size_of::<LosMemDynNode>() as u32)
                as isize
        )
    );
    println!("the pointer given is: {:p}", ptr);
    println!("PROBABLY A WILD POINTER");
    //Os_Back_Trace!(); //TODO
    println!("************************************************");
}

//237行
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Checksum_Set(ctl_node: *mut LosMemCtlNode) {
    (*ctl_node)
        .checksum
        .set(Os_Mem_Node_Checksum_Calculate!(ctl_node));
}

//242行
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Checksum_Verify(ctl_node: *mut LosMemCtlNode) -> bool {
    (*ctl_node).checksum.get() == Os_Mem_Node_Checksum_Calculate!(ctl_node)
}

//247行
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Backup_Setup(node: *mut LosMemDynNode) {
    let node_pre: *mut LosMemDynNode = (*node).self_node.prenode;
    if !node_pre.is_null() {
        std::ptr::write((*node_pre).backup_node.myunion.free_node_info.pst_next, 
            *((*node).self_node.myunion.free_node_info.pst_next));
        std::ptr::write((*node_pre).backup_node.myunion.free_node_info.pst_prev, 
            *((*node).self_node.myunion.free_node_info.pst_prev));
        (*node_pre).backup_node.prenode = (*node).self_node.prenode;
        (*node_pre).backup_node.checksum = (*node).self_node.checksum;
        (*node_pre).backup_node.gapsize = (*node).self_node.gapsize;
        (*node_pre).backup_node.size_and_flag = (*node).self_node.size_and_flag;
    }
}

//260
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Backup_Setup_4_Next(pool: *mut std::ffi::c_void, node: *mut LosMemDynNode) -> u32 {
    let node_next: *mut LosMemDynNode = Os_Mem_Node_Next_Get(pool, node);

    if !Os_Mem_Checksum_Verify(&mut (*node_next).self_node as *mut LosMemCtlNode) {
        println!("[function name]the next node is broken!!");
        Os_Mem_Disp_Ctl_Node(&mut (*node_next).self_node as *mut LosMemCtlNode);
        println!("Current node details:");
        Os_Mem_Disp_More_Details(node);

        return 1; //LOS_NOK
    }

    if !Os_Mem_Checksum_Verify(&mut (*node).backup_node as *mut LosMemCtlNode) {
        std::ptr::write((*node).backup_node.myunion.free_node_info.pst_next, 
            *((*node_next).self_node.myunion.free_node_info.pst_next));
            std::ptr::write((*node).backup_node.myunion.free_node_info.pst_prev, 
            *((*node_next).self_node.myunion.free_node_info.pst_prev));
        (*node).backup_node.prenode = (*node_next).self_node.prenode;
        (*node).backup_node.checksum = (*node_next).self_node.checksum;
        (*node).backup_node.gapsize = (*node_next).self_node.gapsize;
        (*node).backup_node.size_and_flag = (*node_next).self_node.size_and_flag;
    }
    return 0; //LOS_OK
}

//295
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
    Os_Mem_Disp_Ctl_Node(&mut (*node_pre).backup_node as *mut LosMemCtlNode);
    println!("the detailed information of previous node:");
    Os_Mem_Disp_More_Details(node_pre);

    std::ptr::write((*node).self_node.myunion.free_node_info.pst_next, 
        *((*node_pre).backup_node.myunion.free_node_info.pst_next));
    std::ptr::write((*node).self_node.myunion.free_node_info.pst_prev, 
        *((*node_pre).backup_node.myunion.free_node_info.pst_prev));
    (*node).self_node.prenode = (*node_pre).backup_node.prenode;
    (*node).self_node.checksum = (*node_pre).backup_node.checksum;
    (*node).self_node.gapsize = (*node_pre).backup_node.gapsize;
    (*node).self_node.size_and_flag = (*node_pre).backup_node.size_and_flag;

    /* we should re-setup next node's backup on current node */
    return Os_Mem_Backup_Setup_4_Next(pool, node);
}

//317
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_First_Node_PrevGet(pool_info: *mut LosMemPoolInfo) -> *mut LosMemDynNode {
    let node_pre: *mut LosMemDynNode = Os_Mem_End_Node!(pool_info, (*pool_info).pool_size);

    if !Os_Mem_Checksum_Verify(&mut (*node_pre).self_node as *mut LosMemCtlNode) {
        println!("the current node is THE FIRST NODE !");
        println!("[function name]: the node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&mut (*node_pre).self_node as *mut LosMemCtlNode);
    }
    if !Os_Mem_Checksum_Verify(&mut (*node_pre).backup_node as *mut LosMemCtlNode) {
        println!("the current node is THE FIRST NODE !");
        println!("[function name]: the backup node information of previous node is bad !!");
        Os_Mem_Disp_Ctl_Node(&mut (*node_pre).backup_node as *mut LosMemCtlNode);
    }

    return node_pre;
}

//337
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Prev_Get(
    pool: *mut std::ffi::c_void,
    node: *mut LosMemDynNode,
) -> *mut LosMemDynNode {
    let node_cur: *mut LosMemDynNode = Os_Mem_First_Node!(pool);
    let node_pre:*mut LosMemDynNode=Os_Mem_First_Node_PrevGet(pool as *mut LosMemPoolInfo);
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;

    if node == Os_Mem_First_Node!(pool) {
        return Os_Mem_First_Node_PrevGet(pool_info);
    }

    while node_cur < Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        if !Os_Mem_Checksum_Verify(&mut (*node_cur).self_node as *mut LosMemCtlNode) {
            println!("[function name]the node information of current node is bad !!");
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
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
            println!("[function name]the node information of current node is bad !!");
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
            println!("[function name]the backup node information of current node is bad !!");
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
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Backup_Restore(pool: *mut std::ffi::c_void, node: *mut LosMemDynNode) -> u32 {
    let node_pre: *mut LosMemDynNode = Os_Mem_Node_Prev_Get(pool, node);

    if node_pre.is_null() {
        return 1; //LOS_NOK
    }

    return Os_Mem_Backup_Do_Restore(pool, node_pre as *mut LosMemDynNode, node);
}

//471
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Set_Gap_Size(ctl_node: *mut LosMemCtlNode, gap_size: u32) {
    (*ctl_node).gapsize.set(gap_size);
}

//492
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Save(node: *mut LosMemDynNode) {
    Os_Mem_Set_Gap_Size(&mut (*node).self_node as *mut LosMemCtlNode, 0);
    Os_Mem_Checksum_Set(&mut (*node).self_node as *mut LosMemCtlNode);
    Os_Mem_Backup_Setup(node);
}

//499
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_Node_Save_With_Gap_Size(node: *mut LosMemDynNode, gap_size: u32) {
    Os_Mem_Set_Gap_Size(&mut (*node).self_node as *mut LosMemCtlNode, gap_size);
    Os_Mem_Checksum_Set(&mut (*node).self_node as *mut LosMemCtlNode);
    Os_Mem_Backup_Setup(node);
}

//506
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_Mem_List_Delete(node: *mut LosDlList, first_node: *mut std::ffi::c_void) {
    let dyn_node: *mut LosMemDynNode = std::ptr::null_mut();

    (*(*node).pst_next).pst_prev = (*node).pst_prev;
    (*(*node).pst_prev).pst_next = (*node).pst_next;

    if (*node).pst_next as *mut std::ffi::c_void >= first_node {
        //dyn_node = Los_Dl_List_Entry!((*node).pst_next, LosMemDynNode, self_node.free_node_info);
        //dyn_node = ((((*node).pst_next) as *mut char).offset((((&mut ((*(0 as *mut LosMemDynNode)).self_node.myunion.free_node_info)).as_mut_ptr() as *mut LosDlList) as isize) * (-1))) as *mut std::ffi::c_void as *mut LosMemDynNode;
        Os_Mem_Node_Save(dyn_node);
    }
    if (*node).pst_prev as *mut std::ffi::c_void >= first_node {
        //dyn_node = Los_Dl_List_Entry!((*node).pst_prev, LosMemDynNode, self_node.free_node_info);
        //dyn_node = ((((*node).pst_next) as *mut char).offset(-((&mut ((*(0 as *mut LosMemDynNode)).self_node.myunion.free_node_info) ) as u32) as isize)) as *mut std::ffi::c_void as *mut LosMemDynNode;
        Os_Mem_Node_Save(dyn_node);
    }

    (*node).pst_next = std::ptr::null_mut();
    (*node).pst_prev = std::ptr::null_mut();

    //dyn_node = ((node as *mut char).offset(-((&mut ((*(0 as *mut LosMemDynNode)).self_node.myunion.free_node_info) ) as u32) as isize)) as *mut std::ffi::c_void as *mut LosMemDynNode;
    Os_Mem_Node_Save(dyn_node);
}

//530
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
unsafe fn Os_MEM_List_Add(
    list_node: *mut LosDlList,
    node: *mut LosDlList,
    first_node: *mut std::ffi::c_void,
) {
    let dyn_node: *mut LosMemDynNode = std::ptr::null_mut();

    (*node).pst_next = (*list_node).pst_next;
    (*node).pst_prev = list_node;

    //dyn_node = Los_Dl_List_Entry!(node, LosMemDynNode, self_node.free_node_info);
    //dyn_node = ((node as *mut char).offset(-((&mut ((*(0 as *mut LosMemDynNode)).self_node.myunion.free_node_info) ) as u32) as isize)) as *mut std::ffi::c_void as *mut LosMemDynNode;
    Os_Mem_Node_Save(dyn_node);

    (*(*list_node).pst_next).pst_prev = node;
    if (*list_node).pst_next as *mut std::ffi::c_void >= first_node {
        //dyn_node = ((((*list_node).pst_next) as *mut char).offset(-((&mut ((*(0 as *mut LosMemDynNode)).self_node.myunion.free_node_info) ) as u32) as isize)) as *mut std::ffi::c_void as *mut LosMemDynNode;
        Os_Mem_Node_Save(dyn_node);
    }

    (*list_node).pst_next = node;
}

//549
#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
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
/*else */

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

fn Arch_Get_Fp() -> u32 {
    let reg_fp: u32 = 0;
    unsafe {
        asm!(
            "mov {0}, fp",
            out(reg) reg_fp,
        );
    }
    reg_fp
}

const Pointer_Size:u32 = 4;

fn Arch_Back_Trace_Get(fp: u32, call_chain: *mut u32, max_depth: u32) -> u32{
    // Implement this function to retrieve backtrace information.
    unsafe {
        let mut count: u32 = 0;
        let mut back_fp = fp;

        let mut stack_start: u32 = 0; // 示例值
        let mut stack_end: u32 = 0;   // 示例值
        let mut tmp_fp: u32 = 0;
        let mut black_lr: u32 = 0;

        while back_fp > stack_start && back_fp < stack_end && count < max_depth {
            let tmp_fp = back_fp;
            let mut back_lr = tmp_fp as *mut u32 as u32;
            back_fp = (tmp_fp - Pointer_Size) as *mut u32 as u32;

            if !call_chain.is_null() {
                *call_chain.offset(count as isize) = back_lr;
            } else {
                println!("traceback {} -- lr = 0x{:x} fp = 0x{:x}\n", count, back_lr, back_fp);
            }

            count = count + 1;
            if back_fp == tmp_fp {
                break;
            }
        }

        count
    }
}

#[cfg(feature="LOSCFG_MEM_LEAKCHECK")]
fn Os_Mem_Link_Register_Record(node: *mut LosMemDynNode) {
    let mut frame_ptr: u32 = 0;/*UINTPTR */

    unsafe {
        std::ptr::write_bytes((*node).self_node.linkreg.as_mut_ptr(), 0, los_record_lr_cnt * std::mem::size_of::<usize>());
        frame_ptr = Arch_Get_Fp();
        Arch_Back_Trace_Get(frame_ptr as u32, (*node).self_node.linkreg.as_mut_ptr(),los_record_lr_cnt as u32);
    }
}


fn Write_Exc_Buf_Va(buffer: &mut String, formatted_string: &str) {
    buffer.push_str(formatted_string);
}

// Define a macro to handle variadic arguments
macro_rules! Write_Exc_Info_To_Buf {
    ($buffer:expr, $($arg:tt)*) => {
        {
            // Format the string using the arguments provided
            let formatted_string = format!($($arg)*);
            // Call the function to write to the buffer
            Write_Exc_Buf_Va($buffer, &formatted_string);
        }
    };
}

#[cfg(feature="LOSCFG_MEM_LEAKCHECK")]
unsafe fn Os_Mem_Node_Backtrace_Info(tmp_node: *mut LosMemDynNode, pre_node: *mut LosMemDynNode) {
    println!("\n broken node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, (*tmp_node).self_node.linkreg.as_ptr().add(i) as *const ());
    }
    println!("\n pre node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, (*pre_node).self_node.linkreg.as_ptr().add(i) as *const ());
    }

    // #[cfg(Loscfg_Shell_Excinfo_Dump)]
    println!("\n broken node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, (*tmp_node).self_node.linkreg.as_ptr().add(i) as *const ());
    }
    println!("\n pre node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, (*pre_node).self_node.linkreg.as_ptr().add(i) as *const ());
    }

}

#[cfg(feature="LOSCFG_MEM_LEAKCHECK")]
pub fn Os_Mem_Used_Node_Show(pool: *mut std::ffi::c_void) {
    let mut tmpNode: *mut LosMemDynNode = std::ptr::null_mut();
    let mut poolInfo: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut intSave: u32 = 0;
    let mut count: u32 = 0;

    if pool.is_null() {
        println!("input param is NULL\n");
        return;
    }

    if Los_Mem_Integrity_Check(pool as *mut LosMemPoolInfo) != 0 {
        println!("LOS_Mem_Integrity_Check error\n");
        return;
    }

    Mem_Lock!(intSave);

    #[cfg(feature="__LP64__")]
    println!("node                ");
    #[cfg(not(feature="__LP64__"))]
    println!("node        ");

    for count in 0..los_record_lr_cnt {
        #[cfg(feature="__LP64__")]
        println!("        LR[{}]       ", count);
        #[cfg(not(feature="__LP64__"))]
        println!("    LR[{}]   ", count);
    }

    println!("\n");

    tmpNode = Os_Mem_First_Node!(pool);
    while tmpNode < Os_Mem_End_Node!(pool, (*poolInfo).pool_size) {
        /*
        Using the mutable container Cell, retrieve values with the .get method in los_memory_internal_h.rs
        */
        if Os_Mem_Node_Get_Used_Flag!(unsafe { (*tmpNode).self_node.size_and_flag.get() }) {
            #[cfg(feature="__LP64__")]
            println!("{:018p}: ", tmpNode);
            #[cfg(not(feature="__LP64__"))]
            println!("{:010p}: ", tmpNode);

            for count in 0..los_record_lr_cnt {
                #[cfg(feature="__LP64__")]
                println!(" {:018p} ", unsafe { (*tmpNode).self_node.linkreg.as_ptr().add(count) });
                #[cfg(not(feature="__LP64__"))]
                println!(" {:010p} ", unsafe { (*tmpNode).self_node.linkreg.as_ptr().add(count) });
            }

            println!("\n");
        }

        tmpNode = Os_Mem_Next_Flag!(tmpNode);
    }
    Mem_Unlock!(intSave);
    //LOS_SpinLockSave(&g_memSpin, &intSave);
    //LOS_SpinUnlockRestore(&g_memSpin, &intSave);
    //reference to los_membox_2.rs 
}

#[cfg(feature="LOSCFG_KERNAL_MEM_SLAB_EXTENTION")]
fn Os_Mem_Realloc_Slab(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, is_slab_mem: &mut bool, size: u32) -> *mut std::ffi::c_void {
    let mut blk_sz: u32 = 0;
    let mut new_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut free_ptr: *mut std::ffi::c_void = ptr;

    blk_sz = Os_Slab_Mem_Check(pool, ptr);
    /*
    This function is defined in los_slab_pri.h 
    and has not been rewritten in Rust yet. - Wang Rui , May 28th 2024 16:03
    */
    if blk_sz == u32::MAX {
        *is_slab_mem = false;
        return std::ptr::null_mut();
    }
    *is_slab_mem = true;

    if size <= blk_sz {
        return ptr;
    }


    new_ptr = Os_Mem_Alloc(pool, size);

    if !new_ptr.is_null() {

        
        // let rc = memcpy_s(new_ptr, size as usize, ptr, blk_sz as usize);
        /*
        The memcpy_s function is replaced by the std::ptr::copy_nonoverlapping function,
        which copies blk_sz bytes from ptr to new_ptr. - by GPT-4o
        */
        unsafe {
            ptr::copy_nonoverlapping(ptr, new_ptr, blk_sz as usize);
        }
        /*
        this function is defined
        in line 1675  - Wang Rui May 28th 2024 16:45
        */
        // if (rc != EOK) {
        //     PRINT_ERR("%s[%d] memcpy_s failed, error type = %d\n", __FUNCTION__, __LINE__, rc);
        //     freePtr = newPtr;
        //     newPtr = NULL;
        // }
        if Los_Mem_Free(pool, free_ptr) != LOS_OK {
            println!("function name : memcpy_s failed, error type = ");
        }
    }

    // Reacquire the spin
    //Los_Spin_Lock(&G_MEM_SPIN);

    new_ptr
}

#[cfg(feature="LOSCFG_KERNAL_MEM_SLAB_EXTENSION")]
pub fn Os_Mem_Alloc(pool: *mut std::ffi::c_void, size: u32) -> *mut std::ffi::c_void {
    Os_Mem_Alloc_With_Check(pool, size)
    /*
        this function is defined
        in line 1353  - Wang Rui May 28th 2024 16:57
    */
}

#[cfg(not(feature="LOSCFG_KERNAL_MEM_SLAB_EXTENSION"))]
fn Os_Mem_Realloc_Slab(pool: *mut std::ffi::c_void, ptr: *const std::ffi::c_void, is_slab_mem: &mut bool, size: u32) -> *mut std::ffi::c_void {
    *is_slab_mem = false;
    std::ptr::null_mut()
}

#[cfg(feature="LOSCFG_EXC_INTERACTION")]
unsafe fn Os_Mem_Exc_Interaction_Init(mem_start: usize) -> u32 {
    let mut ret: u32 = 0;
    let m_auc_sys_mem0 = mem_start as *mut u8;
    let g_exc_interact_mem_size = EXC_INTERACT_MEM_SIZE;
    ret = Los_Mem_Init(m_auc_sys_mem0, g_exc_interact_mem_size);
    println!(
        "LiteOS kernel exc interaction memory address: {:p}, size: 0x{:x}",
        m_auc_sys_mem0, g_exc_interact_mem_size
    );
    ret
}


/*return NULL*/
fn Os_Mem_Find_Suitable_Free_Block(pool: *mut std::ffi::c_void, alloc_size: u32) -> Option<*mut LosMemDynNode> {
    let mut list_node_head: *mut LosDlList = std::ptr::null_mut();
    let mut tmp_node: *mut LosMemDynNode = std::ptr::null_mut();

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    let mut ret = LOS_OK;
    list_node_head = Os_Mem_Head!(pool, alloc_size);
    while !list_node_head.is_null() {
        /*define only once in los_memory.c line 797*/
        for tmp_node in Los_Dl_List_For_Each_Entry(list_node_head, list_node_head.offset(alloc_size as isize)) {//TODO:
            #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
            if !Os_Mem_Checksum_Verify((*tmp_node).self_node) {
                println!(
                    "the node information of current node is bad !!",
                    
                );
                Os_Mem_Disp_Ctl_Node(tmp_node.self_node);
                ret = Os_Mem_Backup_Restore(pool, tmp_node);
            }
            #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
            if ret != LOS_OK {
                break;
            }
            #[cfg(feature = "LOSCFG_MEM_DEBUG")]
            if tmp_node < pool
                || tmp_node > pool.offset((*(pool as *const LosMemPoolInfo)).pool_size as isize)
                || (tmp_node as u32 & (Os_Mem_Align_Size!() as u32 - 1)) != 0
            {
                println!(
                    "Mem node data error:OS_MEM_HEAD_ADDR(pool)={:?}, list_node_head:{:?},alloc_size={}, tmp_node={:?}",
                    Os_Mem_Head_Addr!(pool),
                    list_node_head,
                    alloc_size,
                    tmp_node
                );
                break;
            }

            if (*tmp_node).size_and_flag >= alloc_size {
                return Some(tmp_node);
            }
        }
        list_node_head = Os_Dlnk_Next_Multi_Head(Os_Mem_Head_Addr!(pool), list_node_head);
    }
    None
}


fn Os_Mem_Clear_Node(node: *mut LosMemDynNode) {
    unsafe {
        std::ptr::write_bytes(node, 0, std::mem::size_of::<LosMemDynNode>());
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

fn Os_Mem_Split_Node(pool: *mut std::ffi::c_void, alloc_node: *mut LosMemDynNode, alloc_size: u32) {
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

fn Os_Mem_Free_Node(node: *mut LosMemDynNode, pool: *mut LosMemPoolInfo) {
    let mut next_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut list_node_head: *mut LosDlList = std::ptr::null_mut();
    let first_node = (Os_Mem_Head_Addr!(pool) as *mut u8).offset(Os_Dlnk_Head_Size!() as isize) as *const std::ffi::c_void;
    (*node).self_node.size_and_flag = Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get()).into();

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

#[cfg(feature = "LOSCFG_MEM_DEBUG")]
fn Os_Mem_Is_Node_Valid(
    node: *mut LosMemDynNode,
    start_node: *mut  LosMemDynNode,
    end_node: *mut LosMemDynNode,
    start_pool: *mut u8,
    end_pool: *mut u8,
) -> bool {
    if !Os_Mem_Middle_Addr(start_node, node, end_node) {
        return false;
    }
    if Os_Mem_Node_Get_Used_Flag(node.self_node.size_and_flag.get()) {
        if !Os_Mem_Magic_Valid(node.self_node.myunion.extend_field.magic.get()) {
            return false;
        }
        return true;
    }
    if !Os_Mem_Middle_Addr_Open_End(start_pool, node.self_node.free_node_info.pst_prev, end_pool) {
        return false;
    }
    true
}

#[cfg(feature = "LOSCFG_MEM_DEBUG")]
fn Os_Mem_Check_Used_Node(pool: *const std::ffi::c_void, node: *mut LosMemDynNode) {
    let pool_info = pool as *const LosMemPoolInfo;
    let start_node = Os_Mem_First_Node!(pool) as *mut LosMemDynNode;
    let end_node = Os_Mem_End_Node!(pool, (*pool_info).pool_size) as *const LosMemDynNode;
    let end_pool = (pool as *const u8).offset((*pool_info).pool_size as isize);
    let next_node: *const LosMemDynNode = std::ptr::null_mut();  

    if (!Os_Mem_Is_Node_Valid(node, start_node, end_node, pool as *mut u8, end_pool))
            || (!Os_Mem_Node_Get_Used_Flag!((*node).self_node.size_and_flag.get()))
        {
            println!("The node:{:?} has been damaged!", node);
        }

        let next_node = Os_Mem_Next_Node!(node);
        if (!Os_Mem_Is_Node_Valid(next_node, start_node, end_node, pool as *mut u8, end_pool))
            || (*next_node).self_node.prenode != node
        {
            println!("The next node:{:?} has been damaged!", next_node);
        }

        if node != start_node {
            let pre_node = (*node).self_node.prenode;
            if (!Os_Mem_Is_Node_Valid(pre_node, start_node, end_node, pool as *mut u8, end_pool))
                || (Os_Mem_Next_Node!(pre_node) != node)
            {
                println!("The previous node:{:?} has been damaged!", pre_node);
            }
        }
    }
#[cfg(not(feature = "LOSCFG_MEM_DEBUG"))]
fn Os_Mem_Check_Used_Node(pool: *const std::ffi::c_void, node: *mut LosMemDynNode)->()
{

}

#[cfg(feature="LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
fn Os_Mem_Pool_Dlink_Check(pool: *const LosMemPoolInfo, list_head: LosDlList) -> u32 {
    let pool_start = (pool  as usize) + std::mem::size_of::<LosMemPoolInfo>();
    let pool_end = (pool as usize) + (*pool).pool_size as usize;

    if (list_head.pst_prev as usize) < pool_start
        || (list_head.pst_prev as usize) >= pool_end
        || (list_head.pst_next as usize) < pool_start
        || (list_head.pst_next as usize) >= pool_end
        || Is_Aligned(list_head.pst_prev as usize, std::mem::size_of::<*const std::ffi::c_void>())
        || Is_Aligned(list_head.pst_next as usize, std::mem::size_of::<*const std::ffi::c_void>())
    {
        return LOS_NOK;
    }

    LOS_OK
}

#[cfg(feature="LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
fn Os_Mem_Pool_Head_Info_Print(pool: *const LosMemPoolInfo) {
    unsafe {
        let pool_info: *const LosMemPoolInfo = pool as *const LosMemPoolInfo ;
        let mut dlink_num:u32 = 0;
        let mut flag :u32 = 0;
        let dlink_head: *const LosMultipleDlinkHead = std::ptr::null_mut();

        if  !Is_ALigned(pool_info, std::mem::size_of::<*const ()>()){
            println!(
                "wrong mem pool addr: {:?}, func:{}",
                pool, "os_mem_pool_head_info_print"
            );
            #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
            println!(
                "wrong mem pool addr: {:?}, func:{}",
                pool, "os_mem_pool_head_info_print"
            );
            return;
        }

        dlink_head = pool.add(1) as *const LosMultipleDlinkHead;
        for dlink_num in 0..Os_Multi_Dlnk_Num {
            if Os_Mem_Pool_Dlink_Check(pool, (*dlink_head).listHead.add(dlink_num)) != 0 {
                flag = 1;
                println!(
                    "DlinkHead[{}]: pst_prev:{:?}, pst_next:{:?}",
                    dlink_num,
                    (*dlink_head).listHead.add(dlink_num).pst_prev,
                    (*dlink_head).listHead.add(dlink_num).pst_next
                );
                #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
                println!(
                    "DlinkHead[{}]: pst_prev:{:?}, pst_next:{:?}",
                    dlink_num,
                    (*dlink_head).listHead.add(dlink_num).pst_prev,
                    (*dlink_head).listHead.add(dlink_num).pst_next
                );
            }
        }
        if flag != 0 {
            println!(
                "mem pool info: poolAddr:{:?}, poolSize:0x{:x}",
                (*pool_info).pool, (*pool_info).pool_size
            );
            #[cfg(feature="LOSCFG_MEM_TASK_STAT")]
            println!(
                // "mem pool info: poolWaterLine:0x{:x}, poolCurUsedSize:0x{:x}",
                // (*pool_info).stat.mem_total_peak, (*pool_info).stat.mem_total_used
                //the stat is undefined.
                "mem pool info: poolWaterLine, poolCurUsedSize"
            );

            #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
            println!(
                "mem pool info: poolAddr:{:?}, poolSize:0x{:x}",
                (*pool_info).pool, (*pool_info).pool_size
            );
            #[cfg(feature="LOSCFG_MEM_TASK_STAT")]
            println!(
                // "mem pool info: poolWaterLine:0x{:x}, poolCurUsedSize:0x{:x}",
                // (*pool_info).stat.mem_total_peak, (*pool_info).stat.mem_total_used
                //the stat is undefined.
                "mem pool info: poolWaterLine, poolCurUsedSize"
            );
        }
    }
}


#[cfg(feature="LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
unsafe fn Os_Mem_Integrity_Check(
    pool: *const LosMemPoolInfo,
    tmp_node: &mut *mut LosMemDynNode,
    pre_node: &mut *mut LosMemDynNode,
) -> u32 {
    let pool_info = pool as *const LosMemPoolInfo;
    let end_pool = (pool as *const u8).add((*pool_info).pool_size as usize);
    Os_Mem_Pool_Head_Info_Print(pool);

    *pre_node = Os_Mem_First_Node!(pool);
    *tmp_node = Os_Mem_First_Node!(pool);
    while *tmp_node < Os_Mem_End_Node!(pool, (*pool_info).pool_size) {
        if Os_Mem_Node_Get_Used_Flag!((**tmp_node).self_node.size_and_flag.get()) {
            if !Os_Mem_Magic_Valid!((**tmp_node).self_node.myunion.extend_field.magic.get()) {
                println!(
                    "[{}], memory check error!\nmemory used but magic num wrong, free_node_info.pst_prev(magic num):{:?}\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.myunion.free_node_info.pst_prev
                );
                #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
                // Write_Exc_Info_To_Buf!(
                println!(
                    "[{}], memory check error!\nmemory used but magic num wrong, free_node_info.pst_prev(magic num):{:?}\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.myunion.free_node_info.pst_prev
                );
                return LOS_NOK;
            }
        } else {
            if !Os_Mem_Middle_Addr_Open_End!(
                pool,
                (**tmp_node).self_node.myunion.free_node_info.pst_prev,
                end_pool
            ) {
                println!(
                    "[{}], memory check error!\nfree_node_info.pst_prev:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.myunion.free_node_info.pst_prev,
                    pool,
                    end_pool
                );
                #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
                // Write_Exc_Info_To_Buf!(
                println!(
                    "[{}], memory check error!\nfree_node_info.pst_prev:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.myunion.free_node_info.pst_prev,
                    pool,
                    end_pool
                );
                return LOS_NOK;
            }
            if !Os_Mem_Middle_Addr_Open_End!(
                pool,
                (**tmp_node).self_node.myunion.free_node_info.pst_next,
                end_pool
            ) {
                println!(
                    "[{}],memory check error!\nfree_node_info.pst_next:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.myunion.free_node_info.pst_next,
                    pool,
                    end_pool
                );
                #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
                // Write_Exc_Info_To_Buf!(
                println!(
                    "[{}], memory check error!\nfree_node_info.pst_next:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.myunion.free_node_info.pst_next,
                    pool,
                    end_pool
                );
                return LOS_NOK;
            }
        }

        *pre_node = *tmp_node;
        *tmp_node = Os_Mem_Next_Node!(*tmp_node);
    }
    LOS_OK
}

#[cfg(feature="LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
unsafe fn Os_Mem_Node_Info(tmp_node: *mut LosMemDynNode, pre_node: *mut LosMemDynNode) {
    if tmp_node == pre_node {
        println!("\n the broken node is the first node\n");
        #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
        println!(
        // Write_Exc_Info_To_Buf(
            "\n the broken node is the first node\n");
    }
    println!(
        "\n broken node head: {:?}  {:?}  {:?}, pre node head: {:?}  {:?}  {:?}\n",
        (*tmp_node).self_node.myunion.free_node_info.pst_prev,
        (*tmp_node).self_node.myunion.free_node_info.pst_next,
        (*tmp_node).self_node.prenode,
        (*pre_node).self_node.myunion.free_node_info.pst_prev,
        (*pre_node).self_node.myunion.free_node_info.pst_next,
        (*pre_node).self_node.prenode
        
    );

    #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
    // Write_Exc_Info_To_Buf!(
    println!(
        "\n broken node head: {:?}  {:?}  {:?}  , pre node head: {:?}  {:?}  {:?}  \n",
        (*tmp_node).self_node.myunion.free_node_info.pst_prev,
        (*tmp_node).self_node.myunion.free_node_info.pst_next,
        (*tmp_node).self_node.prenode,
        (*pre_node).self_node.myunion.free_node_info.pst_prev,
        (*pre_node).self_node.myunion.free_node_info.pst_next,
        (*pre_node).self_node.prenode
    );

    #[cfg(feature="LOSCFG_MEM_LEAKCHECK")]
    Os_Mem_Node_Backtrace_Info(tmp_node, pre_node);

    println!("\n---------------------------------------------\n");
    let dump_end = if (tmp_node as usize).wrapping_add(Node_Dump_Size) > tmp_node as usize {
        tmp_node.add(Node_Dump_Size)
    } else {
        usize::MAX as *const LosMemDynNode
    };
    let dump_size = (dump_end as usize) - (tmp_node as usize);
    println!(" dump mem tmp_node:{:?} ~ {:?}", tmp_node, dump_end);
    Os_Dump_Mem_Byte(dump_size, tmp_node as usize);
    println!("\n---------------------------------------------\n");
    if pre_node != tmp_node {
        println!(
            " dump mem :{:?} ~ tmp_node:{:?}\n",
            (tmp_node as usize).wrapping_sub(Node_Dump_Size),
            tmp_node
        );
        Os_Dump_Mem_Byte(Node_Dump_Size, (tmp_node as usize).wrapping_sub(Node_Dump_Size));
        println!("\n---------------------------------------------\n");
    }
}

#[cfg(feature="LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
unsafe fn Os_Mem_Integrity_Check_Error(tmp_node: *mut LosMemDynNode, pre_node: *mut LosMemDynNode) {
    let mut task_cb: *mut LosTaskCB = ptr::null_mut();
    let mut task_id: u32 = 0;

    Os_Mem_Node_Info(tmp_node, pre_node);

    task_id = Os_Mem_Taskid_Get(pre_node);
    if task_id >= g_task_max_num {
        #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
        // Write_Exc_Info_To_Buf!(
        println!("Task ID {} in pre node is invalid!\n", task_id);
        println!("Task ID {} in pre node is invalid!\n", task_id);
    }

    task_cb = Os_Tcb_From_Tid(task_id);
    //OS_TASK_STATUS_UNUSED = 0x0001U
    if (*task_cb).task_status & OS_TASK_STATUS_UNUSED != 0
        || (*task_cb).task_entry.is_null()
        || (*task_cb).task_name.is_null()
    {
        #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
        // Write_Exc_Info_To_Buf!
        println!("\r\nTask ID {} in pre node is not created or deleted!\n", task_id);
        println!("\r\nTask ID {} in pre node is not created!\n", task_id);
    }

    #[cfg(feature="LOSCFG_SHELL_EXCINFO_DUMP")]
    // Write_Exc_Info_To_Buf!(
    println!(
        "cur node: {:?}\npre node: {:?}\npre node was allocated by task\n",
        tmp_node,
        pre_node
    );
    println!(
        "cur node: {:?}\npre node: {:?}\npre node was allocated by task\n",
        tmp_node,
        pre_node
    );
}

#[cfg(feature="LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
fn Los_Mem_Integrity_Check(pool: *mut LosMemPoolInfo) -> u32 {
    unsafe {
        let mut tmp_node: *mut LosMemDynNode = ptr::null_mut();
        let mut pre_node: *mut LosMemDynNode = ptr::null_mut();
        let mut int_save: u32 = 0;

        if pool.is_null() {
            return LOS_NOK;
        }

        Mem_Lock!(int_save);
        if Os_Mem_Integrity_Check(pool, &mut tmp_node, &mut pre_node) != LOS_OK {
            Goto_Error_Out(pool, tmp_node, pre_node, int_save);
            return LOS_NOK;
        }
        Mem_Unlock!(int_save);
        LOS_OK
    }
}

#[cfg(feature="LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
unsafe fn Goto_Error_Out(
    pool: *mut LosMemPoolInfo,
    tmp_node: *mut LosMemDynNode,
    pre_node: *mut LosMemDynNode,
    int_save: u32,
) {
    Os_Mem_Integrity_Check_Error(tmp_node, pre_node);
    Mem_Unlock!(int_save);
}


#[inline(always)]
fn Os_Slab_Mem_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void) -> bool {
    false
}

#[cfg(feature = "LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK")]
fn Os_Mem_Integrity_Multi_Check() {
    if Los_Mem_Integrity_Check(m_auc_sys_mem1) == LOS_OK {
        println!("system memcheck over, all passed!");
        #[cfg(feature = "LOSCFG_SHELL_EXCINFO_DUMP")]
        Write_Exc_Info_To_Buf("system memcheck over, all passed!\n");
    }

    #[cfg(feature = "LOSCFG_EXC_INTERACTION")]
    {
        if Los_Mem_Integrity_Check(m_auc_sys_mem0).is_ok() {
            println!("exc interaction memcheck over, all passed!");
            #[cfg(feature = "LOSCFG_SHELL_EXCINFO_DUMP")]
            Write_Exc_Info_To_Buf("exc interaction memcheck over, all passed!\n");
        }
    }
}

#[cfg(not(feature = "LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK"))]
fn Los_Mem_Integrity_Check(pool: *mut u8) -> u32 {
    LOS_OK
}
#[cfg(not(feature = "LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK"))]
fn Os_Mem_Integrity_Multi_Check(){
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


#[inline]
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

/*
 * Description : Allocate node from Memory pool
 * Input       : pool  --- Pointer to memory pool
 *               size  --- Size of memory in bytes to allocate
 * Return      : Pointer to allocated memory
 */
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
    alloc_node = Os_Mem_Find_Suitable_Free_Block(pool as *mut std::ffi::c_void, alloc_size as u32).expect("REASON");
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
    alloc_node.offset(1) as *mut std::ffi::c_void
}

/*
 * Description : reAlloc a smaller memory node
 * Input       : pool      --- Pointer to memory pool
 *               allocSize --- the size of new node which will be alloced
 *               node      --- the node which will be realloced
 *               nodeSize  --- the size of old node
 * Output      : node      --- pointer to the new node after realloc
 */
#[inline]
fn Os_Mem_Realloc_Smaller(pool: *mut LosMemPoolInfo, alloc_size: u32, node: *mut LosMemDynNode, node_size: u32){
    if alloc_size + Os_Mem_Node_Head_Size!() + Os_Mem_Align_Size!() as u32<= node_size {
        (*node).self_node.size_and_flag.set(node_size);
        Os_Mem_Split_Node(pool as *mut std::ffi::c_void, node, alloc_size);

        let mut size_and_flag:u32 = (*node).self_node.size_and_flag.get();
        Os_Mem_Node_Set_Used_Flag!(size_and_flag);
        (*node).self_node.size_and_flag.set(size_and_flag);
        
        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        Os_Mem_Node_Save(node);
    }

    #[cfg(feature = "LOSCFG_MEM_LEAKCHECK")]
    Os_Mem_Link_Register_Record(node);
}

/*
 * Description : reAlloc a Bigger memory node after merge node and nextNode
 * Input       : pool      --- Pointer to memory pool
 *               allocSize --- the size of new node which will be alloced
 *               node      --- the node which will be realloced
 *               nodeSize  --- the size of old node
 *               nextNode  --- pointer next node which will be merged
 * Output      : node      --- pointer to the new node after realloc
 */

#[inline]
fn Os_Mem_Merge_Node_For_Realloc_Bigger(pool: *mut LosMemPoolInfo, alloc_size: u32, node: *mut LosMemDynNode, node_size: u32, next_node: *mut LosMemDynNode){
    let first_node: *mut LosMemDynNode = (Os_Mem_Head_Addr!(pool) as *mut u8).add(Os_Dlnk_Head_Size!() as usize) as *mut LosMemDynNode;

    (*node).self_node.size_and_flag.set(node_size);
    Os_Mem_List_Delete(&mut *(*next_node).self_node.myunion.free_node_info, first_node as *mut std::ffi::c_void);
    Os_Mem_Merge_Node(next_node);
    if alloc_size + Os_Mem_Node_Head_Size!() + Os_Mem_Align_Size!() as u32 <= (*node).self_node.size_and_flag.get() {
        Os_Mem_Split_Node(pool as *mut std::ffi::c_void, node, alloc_size);
    }

    let mut size_and_flag:u32 = (*node).self_node.size_and_flag.get();
    Os_Mem_Node_Set_Used_Flag!(size_and_flag);
    (*node).self_node.size_and_flag.set(size_and_flag);

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    Os_Mem_Node_Save(node);

    #[cfg(feature = "LOSCFG_MEM_LEAKCHECK")]
    Os_Mem_Link_Register_Record(node);
}


/*
 * Description : reAlloc a Bigger memory node after merge node and nextNode
 * Input       : pool      --- Pointer to memory pool
 *               allocSize --- the size of new node which will be alloced
 *               node      --- the node which will be realloced
 *               nodeSize  --- the size of old node
 *               nextNode  --- pointer next node which will be merged
 * Output      : node      --- pointer to the new node after realloc
 */
fn Los_List_Add(list: *mut LosDlList, node: *mut LosDlList)->(){
    (*node).pst_next = (*list).pst_next;
    (*node).pst_prev = list;
    (*(*list).pst_next).pst_prev = node;
    (*list).pst_next = node;
}
fn Los_List_Tail_Insert(list: *mut LosDlList, node: *mut LosDlList)->(){
    Los_List_Add((*list).pst_prev, node);
} 
fn Os_Mem_Init(pool: *mut std::ffi::c_void, size: u32) -> u32 {
    let pool_info: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let new_node: *mut LosMemDynNode = std::ptr::null_mut();
    let end_node: *mut LosMemDynNode = std::ptr::null_mut();
    let list_node_head: *mut LosDlList = std::ptr::null_mut();
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
        (*new_node).self_node.size_and_flag.set(pool_size - ((new_node as u32) - (pool as u32)) - Os_Mem_Node_Head_Size!());
        (*new_node).self_node.prenode = Os_Mem_End_Node!(pool, pool_size) as *mut LosMemDynNode;
        list_node_head = Os_Mem_Head!(pool, (*new_node).self_node.size_and_flag.get());
        if list_node_head.is_null() {
            return LOS_NOK;
        }

        Los_List_Tail_Insert(list_node_head, &mut *(*new_node).self_node.myunion.free_node_info);
        end_node = Os_Mem_End_Node!(pool, pool_size) as *mut LosMemDynNode;
        std::ptr::write_bytes(end_node, 0, std::mem::size_of::<LosMemDynNode>());
        (*end_node).self_node.prenode = new_node;
        (*end_node).self_node.size_and_flag.set(Os_Mem_Node_Head_Size!()) ;

        let mut size: u32 = (*end_node).self_node.size_and_flag.get();
        Os_Mem_Node_Set_Used_Flag!(size);
        (*end_node).self_node.size_and_flag.set(size);

        Os_Mem_Set_Magic_Num_And_Task_Id(end_node);

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

fn Los_Mem_Init(pool: *mut std::ffi::c_void, mut size: u32) -> u32 {
    let mut int_save: u32;

    if pool.is_null() || size < Os_Mem_Min_Pool_Size!() {
        return LOS_NOK;
    }

    if !Is_Aligned!(size, Os_Mem_Align_Size!()) || !Is_Aligned!(pool as u32, Os_Mem_Align_Size!()) {
        println!("pool [{:?}, {:?}) size 0x{:x} should be aligned with OS_MEM_ALIGN_SIZE\n",
                 pool, unsafe { pool.offset(size as isize) }, size);
        size = (Os_Mem_Align!(size, Os_Mem_Align_Size!()) - Os_Mem_Align_Size!()) as u32;
    }

    Mem_Lock!(int_save);
    if Os_Mem_Mul_Pool_Init(pool, size) != 0 {
        Mem_Unlock!(int_save);
        return LOS_NOK;
    }

    if Os_Mem_Init(pool, size) != LOS_OK {
        Os_Mem_Mul_Pool_Deinit(pool);
        Mem_Unlock!(int_save);
        return LOS_NOK;
    }
    Mem_Unlock!(int_save);

    //Los_Trace!(MEM_INFO_REQ, pool);
    LOS_OK
}

fn Os_Mem_System_Init(mem_start: usize) -> u32 {
    let mut ret: u32 = 0;
    let pool_size: u32 = 0;

    unsafe {
        m_auc_sys_mem1 = mem_start as *mut u8;
    }
    let pool_size:u32 = 2*1024*1024;
    let ret = Los_Mem_Init(m_auc_sys_mem1 as *mut std::ffi::c_void, pool_size);
    println!(
        "LiteOS system heap memory address:{:?},size:0x{:x}",
        m_auc_sys_mem1, pool_size
    );
    #[cfg(not(feature="LOSCFG_EXC_INTERACTION"))]
    {
        unsafe {
            m_auc_sys_mem0 = m_auc_sys_mem1;
        }
    }
    ret
}

fn Los_Mem_Alloc(pool: *mut std::ffi::c_void, size: u32) -> *mut std::ffi::c_void {
    let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut int_save: u32;

    if pool.is_null() || size == 0 {
        return std::ptr::null_mut();
    }

    Mem_Lock!(int_save);
    loop {
        if Os_Mem_Node_Get_Used_Flag!(size) || Os_Mem_Node_Get_Aligned_Flag!(size) {
            break;
        }

        ptr = std::ptr::null_mut();
        if ptr.is_null() {
            ptr = Os_Mem_Alloc_With_Check(pool as *mut LosMemPoolInfo, size);
        }
        break;
    }

    Mem_Unlock!(int_save);

    //Los_Trace!(Mem_Alloc, pool, ptr as u32, size);
    ptr
}

fn Los_Mem_Alloc_Align(pool: *mut std::ffi::c_void, size: u32, boundary: u32) -> *mut std::ffi::c_void {
    let mut use_size: u32;
    let mut gap_size: u32;
    let ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let aligned_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let alloc_node: *mut LosMemDynNode = std::ptr::null_mut();
    let mut int_save: u32;

    if pool.is_null() || size == 0 || boundary == 0 || !Is_Pow_Two!(boundary) ||
        !Is_Aligned!(boundary, std::mem::size_of::<*mut std::ffi::c_void>() as u32) {
        return std::ptr::null_mut();
    }

    Mem_Lock!(int_save);
    loop {
        /*
         * sizeof(gapSize) bytes stores offset between alignedPtr and ptr,
         * the ptr has been OS_MEM_ALIGN_SIZE(4 or 8) aligned, so maximum
         * offset between alignedPtr and ptr is boundary - OS_MEM_ALIGN_SIZE
         */
        if (boundary - std::mem::size_of::<u32>() as u32) > (u32::MAX - size) {
            break;
        }

        use_size = (size + boundary) - std::mem::size_of::<u32>() as u32;
        if Os_Mem_Node_Get_Used_Flag!(use_size) || Os_Mem_Node_Get_Aligned_Flag!(use_size) {
            break;
        }

        ptr = Os_Mem_Alloc_With_Check(pool as *mut LosMemPoolInfo, use_size);

        aligned_ptr = Os_Mem_Align!(ptr, boundary as usize) as *mut std::ffi::c_void;
        if ptr == aligned_ptr {
            break;
        }
        /* store gapSize in address (ptr -4), it will be checked while free */
        gap_size = (aligned_ptr as u32) - (ptr as u32);
        unsafe {alloc_node = (ptr as *mut LosMemDynNode).offset(-1);}

        let mut size: u32 = (*alloc_node).self_node.size_and_flag.get();
        Os_Mem_Node_Set_Aligned_Flag!(size);
        (*alloc_node).self_node.size_and_flag.set(size);

        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        Os_Mem_Node_Save_With_Gap_Size(alloc_node, gap_size);

        Os_Mem_Node_Set_Aligned_Flag!(gap_size);
        unsafe{*((aligned_ptr as u32 - std::mem::size_of::<u32>() as u32) as *mut u32) = gap_size;}

        ptr = aligned_ptr;
        break;
    }

    Mem_Unlock!(int_save);
    //Los_Trace!(Mem_Alloc_Align, pool, ptr as u32, size, boundary);
    ptr
}

fn Os_Do_Mem_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, node: *mut LosMemDynNode)->(){
    Os_Mem_Check_Used_Node(pool, node);
    Os_Mem_Free_Node(node, pool as *mut LosMemPoolInfo);

    #[cfg(feature = "LOSCFG_KERNEL_LMS")]
    {
        if !g_lms_Free_Hook.is_null() {
            g_lms_Free_Hook(ptr);
        }
    }
} 

fn Os_Mem_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void) -> u32 {
    let mut ret: u32 = LOS_OK;
    let mut gap_size: u32;
    let node: *mut LosMemDynNode;

    loop {
        unsafe {gap_size = *((ptr as u32 - std::mem::size_of::<u32>() as u32) as *mut u32);}
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

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        ret = Os_Mem_Backup_Check_And_Restore(pool, node, ptr as *mut std::ffi::c_void);
        if ret == 0 {
            Os_Do_Mem_Free(pool, ptr, node);
        }
    }

    ret
}

fn Los_Mem_Free(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void) -> u32 {
    let mut ret: u32;
    let mut int_save: u32;

    if pool.is_null() || ptr.is_null() || !Is_Aligned!(pool as u32, std::mem::size_of::<*mut std::ffi::c_void>()) || !Is_Aligned!(ptr as u32, std::mem::size_of::<*mut std::ffi::c_void>()) {
        return LOS_NOK;
    }

    Mem_Lock!(int_save);

    if Os_Slab_Mem_Free(pool, ptr) {
        ret = LOS_OK;
    } 
    else {
        ret = Os_Mem_Free(pool, ptr);
    }

    Mem_Unlock!(int_save);

    //Los_Trace!(MEM_FREE, pool, ptr as u32);
    ret
}

fn Os_Get_Real_Ptr(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    let real_ptr: *mut std::ffi::c_void = ptr;
    let mut gap_size: u32 = *((ptr as u32 - std::mem::size_of::<u32>() as u32) as *mut u32);
    if Os_Mem_Node_Get_Aligned_Flag!(gap_size) && Os_Mem_Node_Get_Used_Flag!(gap_size) {
        eprintln!("[{}:{}]: gapSize:0x{:x} error", "Os_Get_Real_Ptr()", line!(), gap_size);
        return std::ptr::null_mut();
    }
    if Os_Mem_Node_Get_Aligned_Flag!(gap_size) {
        gap_size = Os_Mem_Node_Get_Aligned_Gapsize!(gap_size);
        if (gap_size & (Os_Mem_Align_Size!() as u32 - 1)) != 0 ||
            gap_size > (ptr as u32 - Os_Mem_Node_Head_Size!() - pool as u32) {
            eprintln!("[{}:{}]: gapSize:0x{:x} error", "Os_Get_Real_Ptr()", line!(), gap_size);
            return std::ptr::null_mut();
        }
        real_ptr = (ptr as u32 - gap_size) as *mut std::ffi::c_void;
    }
    real_ptr
}

fn Os_Mem_Realloc(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, size: u32) -> *mut std::ffi::c_void {
    let node: *mut LosMemDynNode = std::ptr::null_mut();
    let next_node: *mut LosMemDynNode = std::ptr::null_mut();
    let tmp_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let real_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut node_size: u32;
    let alloc_size: u32 = Os_Mem_Align!(size + Os_Mem_Node_Head_Size!(), Os_Mem_Align_Size!()) as u32;

    real_ptr = Os_Get_Real_Ptr(pool, ptr);
    if real_ptr.is_null() {
        return std::ptr::null_mut();
    }

    node = (real_ptr as u32 - Os_Mem_Node_Head_Size!()) as *mut LosMemDynNode;
    Os_Mem_Check_Used_Node(pool, node);

    node_size = Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get());
    if node_size >= alloc_size {
        Os_Mem_Realloc_Smaller(pool as *mut LosMemPoolInfo, alloc_size, node, node_size);
        return ptr;
    }

    next_node = Os_Mem_Next_Node!(node);
    unsafe {
        if !Os_Mem_Node_Get_Used_Flag!((*next_node).self_node.size_and_flag.get()) &&
        ((*next_node).self_node.size_and_flag.get() + node_size) >= alloc_size {
        Os_Mem_Merge_Node_For_Realloc_Bigger(pool as *mut LosMemPoolInfo, alloc_size, node, node_size, next_node);
        return ptr;
    }
    }
    tmp_ptr = Os_Mem_Alloc_With_Check(pool as *mut LosMemPoolInfo, size);
    if !tmp_ptr.is_null() {
        let gap_size: u32 = ptr as u32 - real_ptr as u32;
        if size < node_size - Os_Mem_Node_Head_Size!() - gap_size {
            Os_Mem_Free(pool, tmp_ptr);
            return std::ptr::null_mut();
        }
        std::ptr::copy(ptr, tmp_ptr, (node_size - Os_Mem_Node_Head_Size!() - gap_size) as usize);
        Os_Mem_Free_Node(node, pool as *mut LosMemPoolInfo);
    }
    tmp_ptr
}
////////////////////////////
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

    #[cfg(feature = "LOSCFG_EXC_INTERACTION")]
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
    Os_Mem_Set_Magic_Num_And_Task_Id(end_node);

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(end_node);
    }
    /*********/
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
    ret = Os_Mem_Info_Get(pool_info as *mut std::ffi::c_void, pool_status);
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

#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
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

#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
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

#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
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

#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
fn Los_Mem_Check_Level_Get() -> u8 {
    g_mem_check_level
}

#[cfg(feature = "LOSCFG_BASE_MEM_NODE_SIZE_CHECK")]
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

    #[cfg(feature = "LOSCFG_EXC_INTERACTION")]
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
        #[cfg(feature = "LOSCFG_EXC_INTERACTION")]
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

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Mod_Check(moduleid: u32) -> u32 {
    if moduleid > Mem_Module_Max!() {
        println!("error module ID input!\n");
        return LOS_NOK;
    }
    return LOS_OK;
}

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
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
        gapsize = Os_Mem_Node_Get_Aligned_Gapsize!(gapsize);
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

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Os_Mem_Node_Size_Get(ptr: *mut std::ffi::c_void) -> u32 {
    let node: *mut LosMemDynNode = Os_Mem_Ptr_To_Node(ptr) as *mut LosMemDynNode;
    if node == std::ptr::null_mut() {
        return 0;
    }

    return Os_Mem_Node_Get_Size!((*node).self_node.size_and_flag.get());
}

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
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

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
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

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
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

    ret = Los_Mem_Free(pool, ptr);
    if ret == LOS_OK {
        Mem_Lock!(int_save);
        g_module_mem_used_size[moduleid as usize] =
            g_module_mem_used_size[moduleid as usize] - size;
        Mem_Unlock!(int_save);
    }
    return ret;
}

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
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
        return Los_Mem_M_Alloc(pool, size, moduleid);
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

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
fn Los_Mem_M_Used_Get(moduleid: u32) -> u32 {
    if Os_Mem_Mod_Check(moduleid) == LOS_NOK {
        return Os_Null_Int!();
    }
    g_module_mem_used_size[moduleid as usize]
}
