/* ----------------------------------------------------------------------------
 * Copyright (c) Huawei Technologies Co., Ltd. 2013-2019. All rights reserved.
 * Description: LiteOS memory Module Implementation
 * Author: Huawei LiteOS Team
 * Create: 2013-01-01
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright notice, this list of
 * conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright notice, this list
 * of conditions and the following disclaimer in the documentation and/or other materials
 * provided with the distribution.
 * 3. Neither the name of the copyright holder nor the names of its contributors may be used
 * to endorse or promote products derived from this software without specific prior written
 * permission.
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS;
 * OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 * WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR
 * OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
 * ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 * --------------------------------------------------------------------------- */
extern crate libc;
use libc::{c_void, size_t};
use std::convert::TryInto;
extern "C" {
    // 声明 C 库中的 memset 函数
    fn memset(s: *mut c_void, c: i32, n: size_t) -> *mut c_void;
}
//头文件los_membox.h
fn los_membox_aligned(mem_addr: usize) -> usize {
    let size_of_usize = core::mem::size_of::<usize>();
    (mem_addr + size_of_usize - 1) & !(size_of_usize - 1)
}
pub struct LosMemboxNode {
    pub pstNext: Option<*mut LosMemboxNode>,
}
pub const OS_MEMBOX_NODE_HEAD_SIZE: usize = core::mem::size_of::<LosMemboxNode>();

pub struct LosMemboxInfo {
    pub uwBlkSize: u32, // The memory block size of the static memory pool
    pub uwBlkNum: u32,  // The total memory block number of the static memory pool
    pub uwBlkCnt: u32,  // The number of allocated memory blocks in the static memory pool
    // #[cfg(LOSCFG_KERNEL_MEMBOX_STATIC)] //TODO
    pub stFreeList: LosMemboxNode, // The list of free memory block node in the static memory pool
}
//securectype.h
macro_rules! securec_mem_max_len {
    () => {{
        0x7fffffffu64
    }};
}
//securecutil.h
//参考https://course.rs/advance/unsafe/inline-asm.html
use std::arch::asm;
macro_rules! securec_memory_barrier {
    ($dest:expr) => {{
        unsafe {
            asm!("mfence", "memory", "volatile");
        }
    }};
}
macro_rules! securec_memset_func_opt {
    ($dest:expr,$value:expr,$count:expr) => {{
        //此处调用lib/libc/string/memset.c中的memset函数
        unsafe {
            libc::memset($dest as *mut libc::c_void, $value, $count);
        }
    }};
}
macro_rules! securec_memset_prevent_dse {
    ($dest:expr,$value:expr,$count:expr) => {{
        securec_memset_func_opt!($dest, $value, $count as usize);
        securec_memory_barrier!($dest);
    }};
}

//memset_s.c
macro_rules! securec_memset_param_ok {
    ($dest:expr, $destMax:expr, $count:expr) => {{
        (securec_likely!($destMax) <= securec_mem_max_len!())
            && (($dest) != std::ptr::null_mut())
            && (($count) <= ($destMax))
    }};
}
unsafe fn memset_s(dest: *mut (), destMax: u64, c: i32, count: u64) -> i32 {
    if securec_memset_param_ok!(dest, destMax, count) {
        securec_memset_prevent_dse!(dest, c, count as usize);
        return 0; //EOK
    }
    return 0;
    //TODO
    /* Meet some runtime violation, return error code */
    // return SecMemsetError(dest, destMax, c);
}

//对应c中LOSCFG_AARCH64的宏定义
#[cfg(target_arch = "aarch64")]
const OS_MEMBOX_MAGIC: u64 = 0xa55a5aa5a55a5aa5;

#[cfg(not(target_arch = "aarch64"))]
const OS_MEMBOX_MAGIC: u64 = 0xa55a5aa5;

unsafe fn os_membox_set_magic<T>(addr: *mut T) {
    (*(addr as *mut LosMemboxNode)).pstNext = Some(OS_MEMBOX_MAGIC as *mut LosMemboxNode);
}
unsafe fn os_membox_check_magic<T>(addr: *mut T) -> u32 {
    if (*(addr as *mut LosMemboxNode)).pstNext == Some(OS_MEMBOX_MAGIC as *mut LosMemboxNode) {
        0 //LOS_OK
    } else {
        1 //LOS_NOK
    }
}
unsafe fn os_membox_user_addr<T>(addr: *mut T) -> *mut () {
    (addr as *mut u8).offset(OS_MEMBOX_NODE_HEAD_SIZE as isize) as *mut ()
}
unsafe fn os_membox_node_addr<T>(addr: *mut T) -> *mut LosMemboxNode {
    (((addr as *mut u8).wrapping_sub(OS_MEMBOX_NODE_HEAD_SIZE)) as *mut std::ffi::c_void)
        as *mut LosMemboxNode
}

//TODO
use libc::c_uint;
#[repr(C)]
#[derive(Debug)]
struct SPIN_LOCK_S {
    rawLock: libc::c_uint,
    //TODO
}
extern "C" {
    fn LOS_SpinLockSave(g_memboxSpin: *mut SPIN_LOCK_S, state: &c_uint);
    fn LOS_SpinUnlockRestore(g_memboxSpin: *mut SPIN_LOCK_S, state: &c_uint);
}
fn membox_lock(state: u32) {
    unsafe {
        LOS_SpinLockSave(core::ptr::null_mut(), &state);
    }
}
fn membox_unlock(state: u32) {
    unsafe {
        LOS_SpinUnlockRestore(core::ptr::null_mut(), &state);
    }
}

unsafe fn os_check_box_mem(boxInfo: *mut LosMemboxInfo, node: *mut ()) -> u32 {
    let mut offset;
    if (*boxInfo).uwBlkSize == 0 {
        return 1; //LOS_NOK
    }

    offset = node as u32 - boxInfo as u32;

    if offset % (*boxInfo).uwBlkSize != 0 {
        return 1; //LOS_NOK
    }

    if offset / (*boxInfo).uwBlkSize >= (*boxInfo).uwBlkNum {
        return 1; //LOS_NOK
    }

    os_membox_check_magic(node)
}

unsafe fn los_memboxinit(pool: *mut (), poolSize: u32, blkSize: u32) -> u32 {
    let mut boxInfo = pool as *mut LosMemboxInfo;
    let mut node: *mut LosMemboxNode;
    let mut index: u32;
    let mut intSave: u32;

    if Some(pool as *mut LosMemboxInfo) == None {
        return 1; //LOS_NOK
    }

    if blkSize == 0 {
        return 1; //LOS_NOK
    }

    //usize 转 u32
    if poolSize < core::mem::size_of::<LosMemboxInfo>().try_into().unwrap() {
        return 1; //LOS_NOK
    }

    membox_lock(intSave);

    (*boxInfo).uwBlkSize = los_membox_aligned(blkSize as usize + OS_MEMBOX_NODE_HEAD_SIZE)
        .try_into()
        .unwrap();
    (*boxInfo).uwBlkNum =
        (poolSize - core::mem::size_of::<LosMemboxInfo>() as u32) / (*boxInfo).uwBlkSize;
    (*boxInfo).uwBlkCnt = 0;

    if (*boxInfo).uwBlkNum == 0 || (*boxInfo).uwBlkSize < blkSize {
        membox_unlock(intSave);
        return 1; //LOS_NOK
    }

    node = boxInfo.wrapping_add(1) as *mut LosMemboxNode;
    (*boxInfo).stFreeList.pstNext = Some(node);

    index = 0;
    while index < ((*boxInfo).uwBlkNum - 1) {
        //TODO
        // (*node).pstNext = os_membox_next(node, (*boxInfo).uwBlkSize);
        node = match (*node).pstNext {
            Some(p) => p,
            None => return 1, //LOS_NOK
        };
        index += 1;
    }

    (*node).pstNext = None;

    membox_unlock(intSave);

    return 0; //LOS_OK
}
unsafe fn los_membox_alloc(pool: *mut ()) -> *mut () {
    let mut boxInfo = pool as *mut LosMemboxInfo;
    let mut node: *mut LosMemboxNode;
    let mut nodeTmp: Option<*mut LosMemboxNode> = None;
    let mut intSave: u32;

    if Some(pool as *mut LosMemboxInfo) == None {
        return core::ptr::null_mut();
    }

    membox_lock(intSave);
    node = &mut (*boxInfo).stFreeList as *mut LosMemboxNode;
    if (*node).pstNext != None {
        nodeTmp = (*node).pstNext;
        if let Some(node_tmp_inner) = nodeTmp {
            (*node).pstNext = (*node_tmp_inner).pstNext;
            os_membox_set_magic(node_tmp_inner);
        }
        (*boxInfo).uwBlkCnt += 1;
    }
    membox_unlock(intSave);

    if let Some(node_tmp_inner) = nodeTmp {
        return os_membox_user_addr(node_tmp_inner);
    } else {
        return core::ptr::null_mut();
    }
}
unsafe fn los_membox_free(pool: *mut (), Box: *mut ()) -> u32 {
    let mut boxInfo = pool as *mut LosMemboxInfo;
    let mut node: *mut LosMemboxNode;
    let mut ret: u32 = 1; //LOS_NOK
    let mut intSave: u32;

    if Some(pool as *mut LosMemboxInfo) == None || Some(Box as *mut LosMemboxInfo) == None {
        return 1; //LOS_NOK
    }

    membox_lock(intSave);
    loop {
        node = os_membox_node_addr(Box);
        if os_check_box_mem(boxInfo, node as *mut ()) != 0 {
            break;
        }
        (*node).pstNext = (*boxInfo).stFreeList.pstNext;
        (*boxInfo).stFreeList.pstNext = Some(node);
        (*boxInfo).uwBlkCnt -= 1;
        break;
    }
    membox_unlock(intSave);

    return ret;
}

unsafe fn los_membox_clr(pool: *mut (), Box: *mut ()) {
    let mut boxInfo = pool as *mut LosMemboxInfo;
    let mut intSave: u32;

    if Some(pool as *mut LosMemboxInfo) == None || Some(Box as *mut LosMemboxInfo) == None {
        return;
    }

    membox_lock(intSave);
    memset_s(
        Box,
        (*boxInfo).uwBlkSize as u64 - OS_MEMBOX_NODE_HEAD_SIZE as u64,
        0,
        (*boxInfo).uwBlkSize as u64 - OS_MEMBOX_NODE_HEAD_SIZE as u64,
    );
    membox_unlock(intSave);
}

unsafe fn los_show_box(pool: *mut ()) {
    let mut index: u32;
    let mut intSave: u32;
    let mut boxInfo = pool as *mut LosMemboxInfo;
    let mut node: *mut LosMemboxNode;

    if Some(pool as *mut LosMemboxInfo) == None {
        return;
    }

    membox_lock(intSave);
    print_info(
        "membox({:p},0x{:x},0x{:x}):\r\n",
        pool,
        (*boxInfo).uwBlkSize,
        (*boxInfo).uwBlkNum,
    );
    print_info("free node list:\r\n");

    index = 0;
    if (*boxInfo).stFreeList.pstNext != None {
        node = match (*boxInfo).stFreeList.pstNext {
            Some(p) => p,
            None => return,
        };
    }
    while (*boxInfo).stFreeList.pstNext != None {
        print_info("({},{:p})\r\n", index, node);
        node = match (*node).pstNext {
            Some(p) => p,
            None => break,
        };
        index += 1;
    }
    print_info("all node list:\r\n");
    node = boxInfo.wrapping_add(1) as *mut LosMemboxNode;
    index = 0;
    while index < (*boxInfo).uwBlkNum {
        print_info("({},{:p},{:p})\r\n", index, node, (*node).pstNext);
        index += 1;
        node = os_membox_next(node, (*boxInfo).uwBlkSize);
    }
    membox_unlock(intSave);
}
unsafe fn los_membox_statistics_get(
    boxMem: *const (),
    maxBlk: *mut u32,
    blkCnt: *mut u32,
    blkSize: *mut u32,
) -> u32 {
    if Some(boxMem as *const LosMemboxInfo) == None
        || Some(maxBlk as *mut u32) == None
        || Some(blkCnt as *mut u32) == None
        || Some(blkSize as *mut u32) == None
    {
        return 1; //LOS_NOK
    }

    (*maxBlk) = (*(boxMem as *const LosMemboxInfo)).uwBlkNum;
    (*blkCnt) = (*(boxMem as *const LosMemboxInfo)).uwBlkCnt;
    (*blkSize) = (*(boxMem as *const LosMemboxInfo)).uwBlkSize;

    return 0; //LOS_OK
}
