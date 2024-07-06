// Assuming LOS_MemInit, LOS_MemAlloc, and LOS_MemFree are defined elsewhere and accessible
// Assuming dprintf is replaced with println! macro for simplicity
// Assuming LOS_OK is defined and accessible
// Assuming UINT8, UINT32 are replaced with u8, u32 respectively for Rust
#![warn(unused_imports)]
#![warn(non_snake_case)]

mod DynMemDemo;
mod DynMemDemo_h;

use std::os::raw::c_void;

use DynMemDemo::*;

const TEST_POOL_SIZE: usize = 2*1024;
static mut G_TEST_POOL: [u8; TEST_POOL_SIZE] = [0; TEST_POOL_SIZE];

fn main() {
    example_dyn_mem();
    loop{}
}

fn example_dyn_mem() {
    unsafe {
        let mut mem: *mut u32 = std::ptr::null_mut();
        let mut ret;
        let i: u32 = 828;
        ret = Los_Mem_Init(G_TEST_POOL.as_mut_ptr() as *mut c_void, TEST_POOL_SIZE as u32);

        
        if LOS_OK == ret {
            println!("内存池初始化成功!");
        } 
        else {
            println!("内存池初始化失败!");
            return;
        }

        // 分配内存
        mem = Los_Mem_Alloc(G_TEST_POOL.as_mut_ptr() as *mut c_void, 4) as *mut u32;
        if !mem.is_null() {
            println!("内存分配失败!");
            return;
        }
        println!("内存分配成功");

        // 赋值
        mem = &i as *const u32 as *mut u32;
        println!("*mem = {}", *mem);

        // 释放内存
        ret = Los_Mem_Free(G_TEST_POOL.as_mut_ptr() as *mut c_void, mem as *mut c_void);
        if LOS_OK == ret {
            println!("内存释放成功!");
        } else {
            println!("内存释放失败!");
        }
    }

}


