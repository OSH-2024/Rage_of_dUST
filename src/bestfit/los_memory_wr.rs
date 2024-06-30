include!("los_memory_internal_h.rs");

/*else */
use std::ptr;
use std::fmt::Write; // For the write! macro

unsafe fn Os_Mem_List_Delete(node: *mut LosDlList, first_node: *const std::ffi::c_void)  {
    // unsafe {
        //let _ = first_node;
        (*(*node).pst_next).pst_prev = (*node).pst_prev;
        (*(*node).pst_prev).pst_next = (*node).pst_next;
        (*node).pst_next = std::ptr::null_mut();
        (*node).pst_prev = std::ptr::null_mut();//空指针，mark
    // }
}

unsafe fn Os_Mem_List_Add(list_node: *mut LosDlList, node: *mut LosDlList, first_node: *const std::ffi::c_void) {
    // unsafe {
        //let _ = first_node; (VOID)firstNode mark
        (*node).pst_next = (*list_node).pst_next;
        (*node).pst_prev = list_node;
        (*(*list_node).pst_next).pst_prev = node;
        (*list_node).pst_next = node;
    // }
}

/* *
    endif
 */
//OsMemLinkRegisterRecord
/* __attribute__((always_inline)) 
#ifdef LOSCFG_MEM_LEAKCHECK

*/
//#ifdef LOSCFG_MEM_LEAKCHECK


/* Internal functions should follow the naming convention of starting with uppercase letters. - Wang Rui, May 27, 2024, 17:38 */
// GPT 4o
// #ifdef LOSCFG_BACKTRACE
#[cfg(Loscfg_Backtrace)]
fn Arch_Get_Fp() -> usize {
    let reg_fp: usize;
    unsafe {
        asm!(
            "mov {0}, fp",
            out(reg) reg_fp,
        );
    }
    reg_fp
}

macro_rules!  Pointer_Size{
    {} => {
        4U
    };
}

fn Arch_Back_Trace_Get(fp: mut u32, call_chain: *mut u32, max_depth: u32) -> u32{
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
            let back_lr = tmp_fp as *mut u32 as mut u32;
            back_fp = (tmp_fp - Pointer_Size) as *mut u32 as mut u32;

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

#[cfg(Loscfg_Mem_Leakcheck)]
#[inline(always)]
fn Os_Mem_Link_Register_Record(node: *mut LosMemDynNode) {
    let mut frame_ptr: u32;/*UINTPTR */

    unsafe {
        std::ptr::write_bytes(node.self_node.linkreg.as_mut_ptr(), 0, los_record_lr_cnt * std::mem::size_of::<usize>());
        frame_ptr = Arch_Get_Fp();
        Arch_Back_Trace_Get(frame_ptr as mut u32, node.self_node.linkreg as *mut u32,los_record_lr_cnt as u32);
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

//#ifdef LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK
#[cfg(Loscfg_Base_Mem_Node_Integrity_check)]
unsafe fn Os_Mem_Node_Backtrace_Info(tmp_node: *mut LosMemDynNode, pre_node: &mut LosMemDynNode) {
    println!("\n broken node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, tmp_node.self_node.linkreg.add(i) as *const ());
    }
    println!("\n pre node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, pre_node.self_node.linkreg.add(i) as *const ());
    }

    #[cfg(Loscfg_Shell_Excinfo_Dump)]
    println!("\n broken node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, tmp_node.self_node.linkreg.add(i) as *const ());
    }
    println!("\n pre node head LR info: \n");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, pre_node.self_node.linkreg.add(i) as *const ());
    }

}

/* */
pub fn Os_Mem_Used_Node_Show(pool: *mut std::ffi::c_void) {
    let mut tmpNode: *mut LosMemDynNode = std::ptr::null_mut();
    let mut poolInfo: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut intSave: u32 = 0;
    let mut count: u32 = 0;

    if pool.is_null() {
        println!("input param is NULL\n");
        return;
    }

    if LOS_Mem_Integrity_Check(pool) {
        println!("LOS_Mem_Integrity_Check error\n");
        return;
    }

    Mem_Lock!(&mut intSave);

    #[cfg(__LP64__)]
    /*
    conditional compilation attribute in Rust.
    It checks if a certain configuration flag, 
    in this case __LP64__, is defined. If it is defined, 
    the code within the block following the #[cfg] attribute 
    will be included during compilation; otherwise, 
    it will be excluded  - by chatgpt 3.5 
     */
    println!("node                ");
    #[cfg(not(__LP64__))]
    println!("node        ");

    for count in 0..los_record_lr_cnt {
        #[cfg(__LP64__)]
        println!("        LR[{}]       ", count);
        #[cfg(not(__LP64__))]
        println!("    LR[{}]   ", count);
    }

    println!("\n");

    tmpNode = Os_Mem_First_Node(pool);
    while tmpNode < Os_Mem_End_Node(pool, (*poolInfo).poolSize) {
        /*
        Using the mutable container Cell, retrieve values with the .get method in los_memory_internal_h.rs
        */
        if Os_Mem_Node_Get_Used_Flag(unsafe { (*tmpNode).self_node.size_and_flag.get() }) != 0 {
            #[cfg(__LP64__)]
            //
            println!("{:018p}: ", tmpNode);
            #[cfg(not(__LP64__))]
            println!("{:010p}: ", tmpNode);

            for count in 0..los_record_lr_cnt {
                #[cfg(__LP64__)]
                println!(" {:018p} ", unsafe { (*tmpNode).self_node.linkreg.add(count) });
                #[cfg(not(__LP64__))]
                println!(" {:010p} ", unsafe { (*tmpNode).self_node.linkreg.add(count) });
            }

            println!("\n");
        }

        tmpNode = Os_Mem_Next_Flag(tmpNode);
    }
    Mem_Unlock!(intSave);
    //LOS_SpinLockSave(&g_memSpin, &intSave);
    //LOS_SpinUnlockRestore(&g_memSpin, &intSave);
    //reference to los_membox_2.rs 
}



/*
the print_err macro writes an error message 
to the standard error output, 
including the current 
function name and line number. 
The first time this function 
appears is in line 61  - Wang Rui May 28th 2024 16:16
*/

// macro_rules! print_err {
//     ($($arg:tt)*) => ({
//         use std::io::Write;
//         let stderr = std::io::stderr();
//         let mut handle = stderr.lock();
//         writeln!(handle, $($arg)*).unwrap();
//     })
// }

// macro_rules! function_name {
//     () => {{
//         fn f() {}
//         fn type_name_of<T>(_: T) -> &'static str {
//             std::any::type_name::<T>()
//         }
//         &type_name_of(f)[..type_name_of(f).len() - 3]
//     }}
// }

#[cfg(Loscfg_Kernal_Mem_Slab_Extension)]
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

    // Unlock the memory spin, to allow the memory alloc API to be called
    // Los_Spin_Unlock(&G_MEM_SPIN);
    /*
    This function is defined in los_spinlock.h line 211
    and has not been rewritten in Rust yet (too complicated). - Wang Rui , May 28th 2024 16:39
    removed June 24th
    */


    /*
    this function is defined
    in line 1535  - Wang Rui May 28th 2024 16:42
    */
    new_ptr = Los_Mem_Alloc(pool, size);

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

#[cfg(Loscfg_Kernal_Mem_Slab_Extension)]
pub fn Os_Mem_Alloc(pool: *mut std::ffi::c_void, size: u32) -> *mut std::ffi::c_void {
    Os_Mem_Alloc_With_Check(pool, size)
    /*
        this function is defined
        in line 1353  - Wang Rui May 28th 2024 16:57
    */
}

fn Os_Mem_Realloc_Slab(pool: *mut std::ffi::c_void, ptr: *const std::ffi::c_void, is_slab_mem: &mut bool, size: u32) -> *mut std::ffi::c_void {
    *is_slab_mem = false;
    std::ptr::null_mut()
}

#[cfg(LOSCFG_EXC_INTERACTION)]
pub fn Os_Mem_Exc_Interaction_Init(mem_start: usize) -> u32 {
    let mut ret: u32 = 0;
    unsafe {
        m_auc_sys_mem0 = mem_start as *mut u8;
    }
    g_exc_interact_mem_size = EXC_INTERACT_MEM_SIZE;
    ret = Los_Mem_Init(m_auc_sys_mem0, g_exc_interact_mem_size);
    println!(
        "LiteOS kernel exc interaction memory address: {:p}, size: 0x{:x}",
        m_auc_sys_mem0, g_exc_interact_mem_size
    );
    ret
}



fn Os_Mem_System_Init(mem_start: usize) -> u32 {
    let mut ret: u32 = 0;
    let pool_size: u32 = 0;

    unsafe {
        m_aucSysMem1 = mem_start as *mut u8;
        /*69 LINE raw*/
    }
    pool_size = OS_SYS_MEM_SIZE;/*mark*/
    ret = Los_Mem_Init(m_aucSysMem1, pool_size);
    println!(
        "LiteOS system heap memory address:{:?},size:0x{:x}",
        m_aucSysMem1, pool_size
    );
    #[cfg(not(Loscfg_Exc_Interaction))]
    {
        unsafe {
            m_aucSysMem0 = m_aucSysMem1;
        }
    }
    ret
}



/*return NULL*/
fn Os_Mem_Find_Suitable_Free_Block(pool: *mut std::ffi::c_void, alloc_size: u32) -> Option<*mut LosMemDynNode> {
    let mut list_node_head: *mut LosDLList = std::ptr::null_mut();
    let mut tmp_node: *mut LosMemDynNode = std::ptr::null_mut();

    #[cfg(feature = "Loscfg_Mem_Head_Backup")]
    let mut ret = LOS_OK;

    while !list_node_head.is_null() {
        /*define only once in los_memory.c line 797*/
        for tmp_node in Los_Dl_List_For_Each_Entry(list_node_head, list_node_head.offset(alloc_size as isize)) {
            #[cfg(feature = "Loscfg_Mem_Head_Backup")]
            if !Os_Mem_Checksum_Verify(tmp_node.self_node) {
                println!(
                    "the node information of current node is bad !!",
                    
                );
                Os_Mem_Disp_Ctl_Node(tmp_node.self_node);
                ret = Os_Mem_Backup_Restore(pool, tmp_node);
            }
            #[cfg(feature = "Loscfg_Mem_Head_Backup")]
            if ret != LOS_OK {
                break;
            }
            #[cfg(feature = "Loscfg_Mem_Debug")]
            if tmp_node < pool
                || tmp_node > pool.offset((*(pool as *const LosMemPoolInfo)).pool_size as isize)
                || (tmp_node as u32 & (OS_MEM_ALIGN_SIZE - 1)) != 0
            {
                println!(
                    "Mem node data error:OS_MEM_HEAD_ADDR(pool)={:?}, list_node_head:{:?},alloc_size={}, tmp_node={:?}",
                    OS_MEM_HEAD_ADDR(pool),
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
        list_node_head = Os_Dlnk_Next_Multi_Head(Os_Mem_Head_Addr(pool), list_node_head);
    }
    None
}


fn Os_Mem_Clear_Node(node: &mut LosMemDynNode) {
    unsafe {
        /* write bytes unimplemented,mark
        */
        std::ptr::write_bytes(node as *mut LosMemDynNode, 0, 1);
    }
}

fn Os_Mem_Merge_Node(node: &mut LosMemDynNode) {
    let mut next_node: *mut LosMemDynNode = std::ptr::null_mut();

    unsafe {
        (*node).self_node.pre_node.size_and_flag = (*node).self_node.size_and_flag + (*node).self_node.pre_node.size_and_flag;
        next_node = (node as *mut u32).offset((*node).self_node.size_and_flag as isize) as *mut LosMemDynNode;
        (*next_node).self_node.pre_node = node.self_node.pre_node;
    }
    #[cfg(feature = "Loscfg_Mem_Head_Backup")]
    {
        Os_Mem_Node_Save(node.self_node.pre_node);
        Os_Mem_Node_Save(next_node);
    }
    Os_Mem_Clear_Node(node);
}

fn Os_Mem_Split_Node(pool: *mut std::ffi::c_void, alloc_node: &mut LosMemDynNode, alloc_size: u32) {
    let mut new_free_node: *mut LosMemDynNode = std::ffi::c_void;
    let mut next_node: *mut LosMemDynNode = std::ffi::c_void;
    let mut list_node_head: *mut LosDLList = std::ffi::c_void;
    let first_node = (Os_Mem_Head_Addr(pool) as *mut u8).offset(OS_DLNK_HEAD_SIZE as isize) as *const std::ffi::c_void;

    new_free_node = (alloc_node as *mut u8).offset(alloc_size as isize) as *mut LosMemDynNode;
    unsafe {
        /*self_node啥用，mark*/
        (*new_free_node).self_node.pre_node = alloc_node;
        (*new_free_node).self_node.size_and_flag = (*alloc_node).self_node.size_and_flag - alloc_size;
        (*alloc_node).self_node.size_and_flag = alloc_size;
        next_node = Os_Mem_Next_Node(new_free_node);
        (*next_node).self_node.pre_node = new_free_node;
    }
    if !Os_Mem_Node_Get_Used_Flag(next_node.self_node.size_and_flag) {
        Os_Mem_List_Delete(&(*next_node).self_node.free_node_info, first_node);
        Os_Mem_Merge_Node(next_node);
    }
    #[cfg(feature = "Loscfg_Mem_Head_Backup")]
    {
        Os_Mem_Node_Save(next_node);
    }
    list_node_head = os_mem_head(pool, (*new_free_node).self_node.size_and_flag);
    if list_node_head.is_null() {
        return;
    }
    Os_Mem_List_Add(list_node_head, &(*new_free_node).free_node_info, first_node);
    #[cfg(feature = "Loscfg_Mem_Head_Backup")]
    {
        Os_Mem_Node_Save(new_free_node);
    }
}

fn Os_Mem_Free_Node(node: &mut LosMemDynNode, pool: &mut LosMemPoolInfo) {
    let mut next_node: *mut LosMemDynNode = std::ffi::c_void;
    let mut list_node_head: *mut LosDLList = std::ffi::c_void;
    let first_node = (os_mem_head_addr(pool) as *mut u8).offset(OS_DLNK_HEAD_SIZE as isize) as *const std::ffi::c_void;
    Os_Mem_Reduce_used(
        &mut pool.stat,
        Os_Mem_Node_Get_Size(node.self_node.size_and_flag),
        Os_Mem_Taskid_Get(node),
    );
    node.self_node.size_and_flag = Os_Mem_Node_Get_Size(node.self_node.size_and_flag);
    #[cfg(feature = "Loscfg_Mem_Head_Backup")]
    {
        Os_Mem_Node_Save(node);
    }
    #[cfg(feature = "Loscfg_Mem_Leakcheck")]
    {
        Os_Mem_Link_Register_Record(node);
    }
    if !Os_Mem_Node_Get_Used_Flag((*node.self_node.pre_node).self_node.size_and_flag) {
        let pre_node = node.self_node.pre_node;
        Os_Mem_Merge_Node(node);
        next_node = Os_Mem_Next_Node(pre_node);
        if !Os_Mem_Node_Get_Used_Flag((*next_node).self_node.size_and_flag) {
            Os_Mem_List_Delete(&(*next_node).self_node.free_node_info, first_node);
            Os_Mem_Merge_Node(next_node);
        }
        Os_Mem_List_Delete(&(*pre_node).self_node.free_node_info, first_node);
        list_node_head = Os_Mem_Head(pool, (*pre_node).self_node.size_and_flag);
        if list_node_head.is_null() {
            return;
        }
        Os_Mem_List_Add(list_node_head, &(*pre_node).self_node.free_node_info, first_node);
    } else {
        next_node = Os_Mem_Next_Node(node);
        if !Os_Mem_Node_Get_Used_Flag((*next_node).self_node.size_and_flag) {
            Os_Mem_List_Delete(&(*next_node).self_node.free_node_info, first_node);
            Os_Mem_Merge_Node(next_node);
        }
        list_node_head = Os_Mem_Head(pool, node.self_node.size_and_flag);
        if list_node_head.is_null() {
            return;
        }
        Os_Mem_List_Add(list_node_head, &node.self_node.free_node_info, first_node);
    }
}

// #ifdef LOSCFG_MEM_DEBUG
#[cfg(feature = "LOSCFG_MEM_DEBUG")]
fn Os_Mem_Is_Node_Valid(
    node: &const LosMemDynNode,
    start_node: &const LosMemDynNode,
    end_node: &const LosMemDynNode,
    start_pool: &const u8,
    end_pool: &u8,
) -> bool {
    if !Os_Mem_Middle_Addr(start_node, node, end_node) {
        return false;
    }
    if Os_Mem_Node_Get_Used_Flag(node.self_node.size_and_flag) {
        if !Os_Mem_Magic_Valid(node.self_node.magic) {
            return false;
        }
        return true;
    }
    if !Os_Mem_Middle_Addr_Open_End(start_pool, node.self_node.free_node_info.pst_prev, end_pool) {
        return false;
    }
    true
}

#[cfg(feature = "loscfg_mem_debug")]
fn Os_Mem_Check_Used_Node(pool: *const std::ffi::c_void, node: &mut LosMemDynNode) {
    let pool_info = pool as *const LosMemPoolInfo;
    let start_node = Os_Mem_First_Node(pool) as *const LosMemDynNode;
    let end_node = Os_Mem_End_Node(pool, (*pool_info).pool_size) as *const LosMemDynNode;
    let end_pool = (pool as *const u8).offset((*pool_info).pool_size as isize);
    let next_node: *const LosMemDynNode = std::ffi::c_void;  

    if (!Os_Mem_Is_Node_Valid(node, start_node, end_node, pool as *mut u8, end_pool))
            || (!Os_Mem_Node_Get_Used_Flag((*node).self_node.size_and_flag))
        {
            println!("The node:{:?} has been damaged!", node);
        }

        let next_node = Os_Mem_Next_Node(node);
        if (!Os_Mem_Is_Node_Valid(next_node, start_node, end_node, pool as *mut u8, end_pool))
            || (*next_node).self_node.pre_node != node
        {
            println!("The next node:{:?} has been damaged!", next_node);
        }

        if node != start_node {
            let pre_node = (*node).self_node.pre_node;
            if (!Os_Mem_Is_Node_Valid(pre_node, start_node, end_node, pool as *mut u8, end_pool))
                || (Os_Mem_Next_Node(pre_node) != node)
            {
                println!("The previous node:{:?} has been damaged!", pre_node);
            }
        }
    }
/*#[cfg(feature = "loscfg_mem_debug")]不成立*/
fn Os_Mem_Check_Used_Node(pool: *const std::ffi::c_void, node: &mut LosMemDynNode)
{

}


fn Os_mem_pool_dlinkcheck(pool: &LosMemPoolInfo, list_head: LosDlList) -> u32 {
    let pool_start = (pool as *const LosMemPoolInfo as usize) + std::mem::size_of::<LosMemPoolInfo>();
    let pool_end = (pool as *const LosMemPoolInfo as usize) + pool.pool_size as usize;

    if (list_head.pst_prev as usize) < pool_start
        || (list_head.pst_prev as usize) >= pool_end
        || (list_head.pst_next as usize) < pool_start
        || (list_head.pst_next as usize) >= pool_end
        || Is_Aligned(list_head.pst_prev as usize, std::mem::size_of::<*const c_void>())
        || Is_Aligned(list_head.pst_next as usize, std::mem::size_of::<*const c_void>())
    {
        return LOS_NOK;
    }

    LOS_OK
}


fn Os_Mem_Pool_Head_Info_Print(pool: *const LosMemPoolInfo) {
    unsafe {
        let pool_info = pool as *const LosMemPoolInfo ;
        let mut dlink_num:u32 = 0;
        let mut flag :u32 = 0;
        let dlink_head: *const LosMultipleDlinkHead = std::ffi::c_void;

        if  !Is_ALigned(pool_info, std::mem::size_of::<*const ()>()){
            println!(
                "wrong mem pool addr: {:?}, func:{}",
                pool, "os_mem_pool_head_info_print"
            );
            #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
            println!(
                "wrong mem pool addr: {:?}, func:{}",
                pool, "os_mem_pool_head_info_print"
            );
            return;
        }

        dlink_head = pool.add(1) as *const LosMultipleDlinkHead;
        for dlink_num in 0..Os_Multi_Dlnk_Num {
            if Os_Mem_Pool_Dlink_Check(pool, (*dlink_head).listHead.add(dlink_num)) {
                flag = 1;
                println!(
                    "DlinkHead[{}]: pst_prev:{:?}, pst_next:{:?}",
                    dlink_num,
                    (*dlink_head).listHead.add(dlink_num).pst_prev,
                    (*dlink_head).listHead.add(dlink_num).pst_next
                );
                #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
                println!(
                    "DlinkHead[{}]: pst_prev:{:?}, pst_next:{:?}",
                    dlink_num,
                    (*dlink_head).listHead.add(dlink_num).pst_prev,
                    (*dlink_head).listHead.add(dlink_num).pst_next
                );
            }
        }
        if flag {
            println!(
                "mem pool info: poolAddr:{:?}, poolSize:0x{:x}",
                pool_info.pool, pool_info.pool_size
            );
            #[cfg(LOSCFG_MEM_TASK_STAT)]
            println!(
                "mem pool info: poolWaterLine:0x{:x}, poolCurUsedSize:0x{:x}",
                pool_info.stat.mem_total_peak, pool_info.stat.mem_total_used
            );

            #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
            println!(
                "mem pool info: poolAddr:{:?}, poolSize:0x{:x}",
                pool_info.pool, pool_info.pool_size
            );
            #[cfg(LOSCFG_MEM_TASK_STAT)]
            println!(
                "mem pool info: poolWaterLine:0x{:x}, poolCurUsedSize:0x{:x}",
                pool_info.stat.mem_total_peak, pool_info.stat.mem_total_used
            );
        }
    }
}



unsafe fn Os_mem_Integrity_Check(
    pool: *const LosMemPoolInfo,
    tmp_node: &mut *mut LosMemDynNode,
    pre_node: &mut *mut LosMemDynNode,
) -> u32 {
    let pool_info = pool as *const LosMemPoolInfo;
    let end_pool = (pool as *const u8).add(pool_info.pool_size);

    Os_Mem_Pool_Head_Info_Print(pool);

    *pre_node = Os_Mem_First_Node(pool);
    *tmp_node = Os_Mem_First_Node(pool);
    while *tmp_node < Os_Mem_End_Node(pool, pool_info.pool_size) {
        if Os_Mem_Node_Get_Used_Flag((**tmp_node).self_node.size_and_flag) {
            if !Os_Mem_Magic_Valid((**tmp_node).self_node.magic) {
                println!(
                    "[{}], memory check error!\nmemory used but magic num wrong, free_node_info.pst_prev(magic num):{:?}\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.free_node_info.pst_prev
                );
                #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
                Write_Exc_Info_To_Buf(
                    "[{}], memory check error!\nmemory used but magic num wrong, free_node_info.pst_prev(magic num):{:?}\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.free_node_info.pst_prev
                );
                return LOS_NOK;
            }
        } else {
            if !Os_Mem_Middle_Addr_Open_End(
                pool,
                (**tmp_node).self_node.free_node_info.pst_prev,
                end_pool,
            ) {
                println!(
                    "[{}], memory check error!\nfree_node_info.pst_prev:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.free_node_info.pst_prev,
                    pool,
                    end_pool
                );
                #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
                Write_Exc_Info_To_Buf(
                    "[{}], memory check error!\nfree_node_info.pst_prev:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.free_node_info.pst_prev,
                    pool,
                    end_pool
                );
                return LOS_NOK;
            }
            if !Os_Mem_Middle_Addr_Open_End(
                pool,
                (**tmp_node).self_node.free_node_info.pst_next,
                end_pool,
            ) {
                println!(
                    "[{}],memory check error!\nfree_node_info.pst_next:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.free_node_info.pst_next,
                    pool,
                    end_pool
                );
                #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
                Write_Exc_Info_To_Buf(
                    "[{}], memory check error!\nfree_node_info.pst_next:{:?} is out of legal mem range[{:?}, {:?}]\n",
                    "os_mem_integrity_check",
                    (**tmp_node).self_node.free_node_info.pst_next,
                    pool,
                    end_pool
                );
                return LOS_NOK;
            }
        }

        *pre_node = *tmp_node;
        *tmp_node = Os_Mem_Next_Node(*tmp_node);
    }
    LOS_OK
}

unsafe fn Os_Mem_Node_Info(tmp_node: *const LosMemDynNode, pre_node: *const LosMemDynNode) {
    if tmp_node == pre_node {
        println!("\n the broken node is the first node\n");
        #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
        Write_Exc_Info_To_Buf("\n the broken node is the first node\n");
    }
    println!(
        "\n broken node head: {:?}  {:?}  {:?}  0x{:x}, pre node head: {:?}  {:?}  {:?}  0x{:x}\n",
        (*tmp_node).self_node.free_node_info.pst_prev,
        (*tmp_node).self_node.free_node_info.pst_next,
        (*tmp_node).self_node.pre_node,
        (*tmp_node).self_node.size_and_flag,
        (*pre_node).self_node.free_node_info.pst_prev,
        (*pre_node).self_node.free_node_info.pst_next,
        (*pre_node).self_node.pre_node,
        (*pre_node).self_node.size_and_flag,
    );

    #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
    Write_Exc_Info_To_Buf(
        "\n broken node head: {:?}  {:?}  {:?}  0x{:x}, pre node head: {:?}  {:?}  {:?}  0x{:x}\n",
        (*tmp_node).self_node.free_node_info.pst_prev,
        (*tmp_node).self_node.free_node_info.pst_next,
        (*tmp_node).self_node.pre_node,
        (*tmp_node).self_node.size_and_flag,
        (*pre_node).self_node.free_node_info.pst_prev,
        (*pre_node).self_node.free_node_info.pst_next,
        (*pre_node).self_node.pre_node,
        (*pre_node).self_node.size_and_flag,
    );

    #[cfg(LOSCFG_MEM_LEAKCHECK)]
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
        os_dump_mem_byte(Node_Dump_Size, (tmp_node as usize).wrapping_sub(Node_Dump_Size));
        println!("\n---------------------------------------------\n");
    }
}

unsafe fn Os_Mem_Integrity_Check_Error(tmp_node: *const LosMemDynNode, pre_node: *const LosMemDynNode) {
    let mut task_cb: *mut LosTaskCB = ptr::null_mut();
    let mut task_id: u32 = 0;

    Os_Mem_Node_Info(tmp_node, pre_node);

    task_id = Os_Mem_Taskid_Get(pre_node);
    if task_id >= g_task_max_num {
        #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
        Write_Exc_Info_To_Buf("Task ID {} in pre node is invalid!\n", task_id);
        println!("Task ID {} in pre node is invalid!\n", task_id);
    }

    task_cb = Os_Tcb_From_Tid(task_id);
    //OS_TASK_STATUS_UNUSED = 0x0001U
    if (*task_cb).task_status & 0x0001U != 0
        || (*task_cb).task_entry.is_null()
        || (*task_cb).task_name.is_null()
    {
        #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
        Write_Exc_Info_To_Buf("\r\nTask ID {} in pre node is not created or deleted!\n", task_id);
        println!("\r\nTask ID {} in pre node is not created!\n", task_id);
    }

    #[cfg(LOSCFG_SHELL_EXCINFO_DUMP)]
    Write_Exc_Info_To_Buf(
        "cur node: {:?}\npre node: {:?}\npre node was allocated by task\n",
        tmp_node,
        pre_node
    );
    panic!(
        "cur node: {:?}\npre node: {:?}\npre node was allocated by task\n",
        tmp_node,
        pre_node
    );
}

fn Los_Mem_Integrity_Check(pool: *mut LosMemPoolInfo) -> u32 {
    unsafe {
        let mut tmp_node: *mut LosMemDynNode = ptr::null_mut();
        let mut pre_node: *mut LosMemDynNode = ptr::null_mut();
        let mut int_save: u32 = 0;

        if pool.is_null() {
            return LOS_NOK;
        }

        Mem_Lock!(&mut int_save);
        if Os_Mem_Integrity_Check(pool, &mut tmp_node, &mut pre_node) != LOS_OK {
            Goto_Error_Out(pool, tmp_node, pre_node, int_save);
            return LOS_NOK;
        }
        Mem_Unlock!(&mut int_save);
        LOS_OK
    }
}

unsafe fn Goto_Error_Out(
    pool: *mut LosMemPoolInfo,
    tmp_node: *mut LosMemDynNode,
    pre_node: *mut LosMemDynNode,
    int_save: u32,
) {
    Os_Mem_Integrity_Check_Error(tmp_node, pre_node);
    Mem_Unlock!(&mut int_save);
}
        
  

