### liteos内存管理实现

---

#### 动态部分

动态内存管理的主要工作是动态分配并管理用户申请到的内存区间。

动态内存管理主要用于用户需要使用大小不等的内存块的场景。当用户需要使用内存时，可以通过操作系统的动态内存申请函数索取指定大小的内存块，一旦使用完毕，通过动态内存释放函数归还所占用内存，使之可以重复使用。

##### 接口描述

| 功能分类                                                     | 接口名                                                       | 描述                                                         |
| ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| 初始化和删除内存池                                           | Los_Mem_Init                                                 | 初始化一块指定的动态内存池，大小为size                       |
| Los_Mem_Deinit                                               | 删除指定内存池，仅打开LOSCFG_MEM_MUL_POOL时有效              |                                                              |
| 申请、释放动态内存                                           | Los_Mem_Alloc                                                | 从指定动态内存池中申请size长度的内存                         |
| Los_Mem_Free                                                 | 释放已申请的内存                                             |                                                              |
| Los_Mem_Realloc                                              | 按size大小重新分配内存块，并将原内存块内容拷贝到新内存块。如果新内存块申请成功，则释放原内存块 |                                                              |
| Los_Mem_Alloc_Align                                          | 从指定动态内存池中申请长度为size且地址按boundary字节对齐的内存 |                                                              |
| 获取内存池信息                                               | Los_Mem_Pool_Size_Get                                        | 获取指定动态内存池的总大小                                   |
| Los_Mem_Total_Used_Get                                       | 获取指定动态内存池的总使用量大小                             |                                                              |
| Los_Mem_Info_Get                                             | 获取指定内存池的内存结构信息，包括空闲内存大小、已使用内存大小、空闲内存块数量、已使用的内存块数量、最大的空闲内存块大小 |                                                              |
| Los_Mem_Pool_List                                            | 打印系统中已初始化的所有内存池，包括内存池的起始地址、内存池大小、空闲内存总大小、已使用内存总大小、最大的空闲内存块大小、空闲内存块数量、已使用的内存块数量。仅打开LOSCFG_MEM_MUL_POOL时有效 |                                                              |
| Los_Mem_Free_Blks_Get                                        | 获取内存块信息                                               | 获取指定内存池的空闲内存块数量                               |
| Los_Mem_Used_Blks_Get                                        | 获取指定内存池已使用的内存块数量                             |                                                              |
| Los_Mem_Task_Id_Get                                          | 获取申请了指定内存块的任务ID                                 |                                                              |
| Los_Mem_Last_Used_Get                                        | 获取内存池最后一个已使用内存块的结束地址                     |                                                              |
| Los_Mem_Node_Size_Check                                      | 获取指定内存块的总大小和可用大小，仅打开LOSCFG_BASE_MEM_NODE_SIZE_CHECK时有效 |                                                              |
| Los_Mem_Free_Node_Show                                       | 打印指定内存池的空闲内存块的大小及数量                       |                                                              |
| Los_Mem_Integrity_Check                                      | 检查指定内存池的完整性                                       | 对指定内存池做完整性检查，仅打开LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK时有效 |
| 设置、获取内存检查级别，仅打开LOSCFG_BASE_MEM_NODE_SIZE_CHECK时有效 | Los_Mem_Check_Level_Set                                      | 设置内存检查级别                                             |
| Los_Mem_Check_Level_Get                                      | 获取内存检查级别                                             |                                                              |
| 为指定模块申请、释放动态内存，仅打开LOSCFG_MEM_MUL_MODULE时有效 | Los_Mem_Malloc                                               | 从指定动态内存池分配size长度的内存给指定模块，并纳入模块统计 |
| Los_Mem_Mfree                                                | 释放已经申请的内存块，并纳入模块统计                         |                                                              |
| Los_Mem_Malloc_Align                                         | 从指定动态内存池中申请长度为size且地址按boundary字节对齐的内存给指定模块，并纳入模块统计 |                                                              |
| Los_Mem_Mrealloc                                             | 按size大小重新分配内存块给指定模块，并将原内存块内容拷贝到新内存块，同时纳入模块统计。如果新内存块申请成功，则释放原内存块 |                                                              |
| 获取指定模块的内存使用量                                     | Los_Mem_Mused_Get                                            | 获取指定模块的内存使用量，仅打开LOSCFG_MEM_MUL_MODULE时有效  |

>  **须知：**
>
> - 上述接口中，通过宏开关控制的都是内存调测功能相关的接口。
> - 对于bestfit_little算法，只支持宏开关LOSCFG_MEM_MUL_POOL控制的多内存池相关接口和宏开关LOSCFG_BASE_MEM_NODE_INTEGRITY_CHECK控制的内存合法性检查接口，不支持其他内存调测功能。
> - 通过Los_Mem_Alloc_Align/Los_Mem_Malloc_Align申请的内存进行Los_Mem_Realloc/Los_Mem_Mrealloc操作后，不能保障新的内存首地址保持对齐。
> - 对于bestfit_little算法，不支持对Los_Mem_Alloc_Align申请的内存进行Los_Mem_Realloc操作，否则将返回失败。

##### 配置项

下面是编译所对应的配置项

 | 配置项                                     | 含义                                                         | 取值范围 | 默认值 | 依赖                                                        |
   | ------------------------------------------ | ------------------------------------------------------------ | -------- | ------ | ----------------------------------------------------------- |
   | LOSCFG_KERNEL_MEM_BESTFIT                  | 选择bestfit内存管理算法                                      | YES/NO   | YES    | 无                                                          |
   | LOSCFG_KERNEL_MEM_BESTFIT_LITTLE           | 选择bestfit_little内存管理算法                               | YES/NO   | NO     | 无                                                          |
   | LOSCFG_KERNEL_MEM_SLAB_EXTENTION           | 使能slab功能，可以降低系统持续运行过程中内存碎片化的程度     | YES/NO   | NO     | 无                                                          |
   | LOSCFG_KERNEL_MEM_SLAB_AUTO_EXPANSION_MODE | slab自动扩展，当分配给slab的内存不足时，能够自动从系统内存池中申请新的空间进行扩展 | YES/NO   | NO     | LOSCFG_KERNEL_MEM_SLAB_EXTENTION                            |
   | LOSCFG_MEM_TASK_STAT                       | 使能任务内存统计                                             | YES/NO   | YES    | LOSCFG_KERNEL_MEM_BESTFIT或LOSCFG_KERNEL_MEM_BESTFIT_LITTLE |

##### 编程实例

本实例执行以下步骤：

1. 初始化一个动态内存池。
2. 从动态内存池中申请一个内存块。
3. 在内存块中存放一个数据。
4. 打印出内存块中的数据。
5. 释放该内存块。

```rust
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
```

###### 结果验证

```
内存池初始化成功!
内存分配成功
*mem = 828
内存释放成功!
```

###### 完整实验代码

[Dyntest_main.rs](https://github.com/OSH-2024/Rage_of_dUST/blob/main/test_demo/Dyntest/main.rs)

#### 静态内存

---

当用户需要使用固定长度的内存时，可以通过静态内存分配的方式获取内存，一旦使用完毕，通过静态内存释放函数归还所占用内存，使之可以重复使用。

##### 接口描述

| 功能分类                 | 接口名                                                       | 描述                                                         |
| ------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| 初始化静态内存池         | Los_Membox_Init                                              | 初始化一个静态内存池，根据入参设定其起始地址、总大小及每个内存块大小 |
| 清除静态内存块内容       | Los_Membox_Clr                                               | 清零指定静态内存块的内容                                     |
| 申请、释放静态内存       | Los_Membox_Alloc                                             | 从指定的静态内存池中申请一块静态内存块                       |
| Los_Membox_Free          | 释放指定的一块静态内存块                                     |                                                              |
| 获取、打印静态内存池信息 | Los_Membox_Statistics_Get                                    | 获取指定静态内存池的信息，包括内存池中总内存块数量、已经分配出去的内存块数量、每个内存块的大小 |
| Los_Show_Box             | 打印指定静态内存池所有节点信息（打印等级是LOS_INFO_LEVEL），包括内存池起始地址、内存块大小、总内存块数量、每个空闲内存块的起始地址、所有内存块的起始地址 |                                                              |

##### 配置项

| 配置项                       | 含义                       | 取值范围 | 默认值 | 依赖                 |
| ---------------------------- | -------------------------- | -------- | ------ | -------------------- |
| LOSCFG_KERNEL_MEMBOX         | 使能membox内存管理         | YES/NO   | YES    | 无                   |
| LOSCFG_KERNEL_MEMBOX_STATIC  | 选择静态内存方式实现membox | YES/NO   | YES    | LOSCFG_KERNEL_MEMBOX |
| LOSCFG_KERNEL_MEMBOX_DYNAMIC | 选择动态内存方式实现membox | YES/NO   | NO     | LOSCFG_KERNEL_MEMBOX |

##### 编程实例

本实例执行以下步骤：

1. 初始化一个静态内存池。
2. 从静态内存池中申请一块静态内存。
3. 在内存块存放一个数据。
4. 打印出内存块中的数据。
5. 清除内存块中的数据。
6. 释放该内存块。

```rust
include!("membox.rs");

 fn main() {
    unsafe
    {
        let mut mem: *mut u32 = std::ptr::null_mut();
        let blk_size: u32 = 16;
        //the blk_size must be 2^n
        let box_size: u32 = 100;
        let mut box_mem: [u64; 1000] = [0; 1000];
        let mut ret: u32 = 0;


        println!("box_mem.as_mut_ptr() = {:p} , box_mem.as_mut_ptr() as *mut std::ffi::c_void = {:p}",box_mem.as_mut_ptr(),box_mem.as_mut_ptr() as *mut std::ffi::c_void);
        ret = los_memboxinit(box_mem.as_mut_ptr() as *mut std::ffi::c_void, box_size, blk_size);
        if ret != 0 {
            println!("内存池初始化失败");
            return;
        } else {
            println!("内存池初始化成功");
        }

        mem = los_membox_alloc(box_mem.as_mut_ptr() as *mut std::ffi::c_void) as *mut u32;
        if mem.is_null() {
            println!("内存分配失败");
            return;
        } else {
            println!("内存分配成功");
        }

        *mem = 828;
        println!("mem:{}", *mem);

        los_membox_clr(
            box_mem.as_mut_ptr() as *mut std::ffi::c_void,
            mem as *mut std::ffi::c_void,
        );
        println!("清除内存内容成功 mem:{}", *mem);

        ret = los_membox_free(
            box_mem.as_mut_ptr() as *mut std::ffi::c_void,
            mem as *mut std::ffi::c_void,
        );
        if ret != 0 {
            println!("内存释放失败");
            return;
        } else {
            println!("内存释放成功");
        }

        return;
    }
}
```

###### 结果验证

打印信息前半部分目的为方便调试

```
内存池初始化成功
内存分配成功
mem = 828
清除内存内容成功 mem = 0
内存释放成功
```



