include!("los_memory_internal_h.rs");

/*else */
use std::ptr;

fn Os_Mem_List_Delete(node: *mut LosDlList, first_node: *const std::ffi::c_void) {
    unsafe {
        let _ = first_node;
        (*(*node).pst_next).pst_prev = (*node).pst_prev;
        (*(*node).pst_prev).pst_next = (*node).pst_next;
        (*node).pst_next = ptr::null_mut();
        (*node).pst_prev = ptr::null_mut();
    }
}

fn Os_Mem_List_Add(list_node: *mut LosDlList, node: *mut LosDlList, first_node: *const std::ffi::c_void) {
    unsafe {
        let _ = first_node;
        (*node).pst_next = (*list_node).pst_next;
        (*node).pst_prev = list_node;
        (*(*list_node).pst_next).pst_prev = node;
        (*list_node).pst_next = node;
    }
}

/* *
    endif
 */
//OsMemLinkRegisterRecord
/* __attribute__((always_inline)) 
#ifdef LOSCFG_MEM_LEAKCHECK

*/
#[inline(always)]
fn Os_Mem_Link_Register_Record(node: &mut LosMemDynNode) {
    let mut frame_ptr: usize;/*UINTPTR */

    unsafe {
        std::ptr::write_bytes(node.self_node.linkreg.as_mut_ptr(), 0, los_record_lr_cnt * std::mem::size_of::<usize>());
        frame_ptr = arch_get_fp();
        arch_backtrace_get(frame_ptr as *mut usize, node.self_node.linkreg.as_mut_ptr(),los_record_lr_cnt);
    }
}

/* Internal functions should follow the naming convention of starting with uppercase letters. - Wang Rui, May 27, 2024, 17:38 */
fn arch_get_fp() -> usize {
    // Implement this function to return the frame pointer.
    unimplemented!()
}

fn arch_backtrace_get(_frame_ptr: *mut usize, _link_reg: *mut u32, _count: usize) {
    // Implement this function to retrieve backtrace information.
    unimplemented!()
}

//#ifdef LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK
fn Os_Mem_Node_Backtrace_Info(tmp_node: &LosMemDynNode, pre_node: &LosMemDynNode) {
    println!("\n broken node head LR info: ");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, tmp_node.self_node.linkreg[i] as *const ());
    }
    println!("\n pre node head LR info: ");
    for i in 0..los_record_lr_cnt {
        println!(" LR[{}]:{:p}", i, pre_node.self_node.linkreg[i] as *const ());
    }
}

/* */
pub fn Os_Mem_Used_Node_Show(pool: *mut std::ffi::c_void) {
    let mut tmpNode: *mut LosMemDynNode = std::ptr::null_mut();
    let poolInfo: *mut LosMemPoolInfo = pool as *mut LosMemPoolInfo;
    let mut intSave: u32;
    let mut count: u32;

    if pool.is_null() {
        println!("input param is NULL\n");
        return;
    }

    if LOS_Mem_Integrity_Check(pool) {
        println!("LOS_Mem_Integrity_Check error\n");
        return;
    }

    Mem_Lock(&mut intSave);

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
                println!(" {:018p} ", unsafe { (*tmpNode).self_node.linkreg[count] });
                #[cfg(not(__LP64__))]
                println!(" {:010p} ", unsafe { (*tmpNode).self_node.linkreg[count] });
            }

            println!("\n");
        }

        tmpNode = Os_Mem_Next_Flag(tmpNode);
    }
    Mem_Unlock(intSave);
    //LOS_SpinLockSave(&g_memSpin, &intSave);
    //LOS_SpinUnlockRestore(&g_memSpin, &intSave);
    //reference to Yang Yibo's code in los_membox_2.rs 
}

//not implemented
#[cfg(LOSCFG_KERNEL_MEM_SLAB_EXTENTION)]
pub fn Os_Mem_Realloc_Slab(pool: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void, isSlabMem: *mut bool, size: u32) -> *mut std::ffi::c_void {
    let mut rc: errno_t;
    let mut blkSz: u32;
    let mut newPtr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut freePtr: *mut std::ffi::c_void = ptr;

    blkSz = Os_Slab_Mem_Check(pool, ptr);
    if blkSz == u32::MAX {
        unsafe {
            *isSlabMem = false;
        }
        return std:ptr::null_mut();
    }

    unsafe {
        *isSlabMem = true;
    }

    if size <= blkSz {
        return ptr;
    }

    /* Unlock the memory spin, to allow the memory alloc API to be called */
    unsafe {
        LOS_Spin_Unlock(&mut g_memSpin);
    }

    newPtr = Los_Mem_Alloc(pool, size);
    if !newPtr.is_null() {
        ptr::copy_nonoverlapping(new_ptr, ptr, blk_sz as usize); // Copy existing data to the new memory block
        if rc != EOK {
            printk!("{}[{}] memcpy_s failed, error type = {}\n", core::file!(), core::line!(), rc);
            freePtr = newPtr;
            newPtr = core::ptr::null_mut();
        }
        if Los_Mem_Free(pool as *const std::ffi::c_void, freePtr) != LOS_OK {
            printk!("{}, {}\n", core::file!(), core::line!());
        }
    }

    /* Reacquire the spin */
    unsafe {
        LOS_SpinLock(&mut g_memSpin);
    }

    newPtr
}

#[cfg(LOSCFG_KERNEL_MEM_SLAB_EXTENTION)]
pub fn OsMemAlloc(pool: *mut core::ffi::c_void, size: u32) -> *mut core::ffi::c_void {
    OsMemAllocWithCheck(pool, size)
}
