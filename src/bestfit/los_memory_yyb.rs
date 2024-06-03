#[allow(unused_macros)]
macro_rules! print_err {
    ($fmt:expr $(, $($arg:tt)+)?) => {
        {
            eprint!("[ERR] ");
            eprint!($fmt $(, $($arg)+)?);
        }
    };
}
use std::cell::Cell;
#[repr(C)]
pub struct LosMemCtlNode {
    prenode: *mut LosMemDynNode,
    /* Size and flag of the current node (the high two bits represent a flag,and the rest bits specify the size) */
    size_and_flag: Cell<u32>,
    //
    gapsize: Cell<u32>,
    checksum: Cell<u32>,
    //
    linkreg: [u32; los_record_lr_cnt],
    //
    reserve2: Cell<u32>,
    //
    myunion: Myunion,
}
pub union Myunion {
    free_node_info: std::mem::ManuallyDrop<Cell<LosDlList>>,
    extend_field: std::mem::ManuallyDrop<Moreinfo>,
}

pub struct Moreinfo {
    magic: Cell<u32>,
    taskid: Cell<u32>,
    //
    moduled: Cell<u32>,
}

pub struct LosMemDynNode {
    #[cfg(LOSCFG_MEM_HEAD_BACKUP)]
    backup_node: LosMemCtlNode,

    self_node: LosMemCtlNode,
}

#[allow(unused_macros)]
macro_rules! node_dump_size {
    () => {
        64
    };
}
#[allow(unused_macros)]
macro_rules! column_num {
    () => {
        8
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

const m_aucSysMem0: Option<Box<u8>> = None;
const m_aucSysMem1: Option<Box<u8>> = None;

type MallocHook = fn() -> ();
static mut g_MALLOC_HOOK: Option<MallocHook> = None;

use std::arch::asm; //?
#[link_section = ".data.init"]
static mut G_SYS_MEM_ADDR_END: usize = 0;

#[cfg(feature = "LOSCFG_EXC_INTERACTION")]
#[link_section = ".data.init"]
static mut G_EXC_INTERACT_MEM_SIZE: usize = 0;

#[cfg(feature = "LOSCFD_BASE_MEM_NODE_SIZE_CHECK")]
static mut G_MEM_CHECK_LEVEL: u8 = 0xff; //LOS_MEM_CHECK_LEVEL_DEFAULT

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
static mut G_MODULE_MEM_USED_SIZE: [u32; MEM_MODULE_MAX + 1] = [0; MEM_MODULE_MAX + 1];

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

pub const fn os_mem_taskid_get(node: &LosMemDynNode) -> u32 {
    node.self_node.taskid
}

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
mod mem_mul_module {
    use crate::LosMemDynNode;

    #[inline]
    pub fn os_mem_modid_set(node: &mut LosMemDynNode, module_id: u32) {
        node.selfNode.moduleId = module_id;

        #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
        {
            os_mem_node_save(node);
        }
    }

    #[inline]
    pub fn os_mem_modid_get(node: &LosMemDynNode) -> u32 {
        node.selfNode.moduleId
    }

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    fn os_mem_node_save(node: &mut LosMemDynNode) {
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
