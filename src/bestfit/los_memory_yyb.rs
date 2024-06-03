macro_rules! print_err {
    ($fmt:expr $(, $($arg:tt)+)?) => {
        {
            eprint!("[ERR] ");
            eprint!($fmt $(, $($arg)+)?);
        }
    };
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
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn with_name(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);

    let fn_name = input.ident.to_string();
    let const_decl = quote::quote! {
        const THIS_FN: &str = #fn_name;
    };

    input
        .block
        .stmts
        .insert(0, syn::parse(const_decl.into()).unwrap());

    let output = quote::quote! {
        #input
    };

    output.into()
}

macro_rules! function_name {
    () => {{
        // 使用core::any::type_name来获取f函数的类型名称，
        // 注意：这个函数返回的是一个包含了路径的完全限定名称，如"module::path::to::function_name"。
        let full_type_name = core::any::type_name::<fn()>();

        // 通过split("::")将完全限定名称分割成多个部分，然后取最后一部分作为函数名。
        // unwrap_or("<unknown>")是为了防止在极特殊情况下（比如分割后为空）返回一个默认值。
        let function_name = full_type_name.split("::").last().unwrap_or("<unknown>");

        // 最终宏展开会返回当前函数的名称字符串。
        function_name
    }};
}

use std::panic::Location; //用于获取行数
macro_rules! os_check_null_return {
    ($param:expr) => {
        if $param.is_null() {
            let location = Location::caller();
            print_err!("{} {}\n", THIS_FN, location.line());
            return;
        }
    };
}

let m_aucSysMem0: Option<Box<u8>> = None;
let m_aucSysMem1: Option<Box<u8>> = None;

type MallocHook = fn() -> ();
MallocHook g_MALLOC_HOOK = None;

use core::arch::asm;

#[section = ".data.init"]
static mut G_SYS_MEM_ADDR_END: usize = 0;

#[cfg(feature = "LOSCFG_EXC_INTERACTION")]
#[section = ".data.init"]
static mut G_EXC_INTERACT_MEM_SIZE: usize = 0;

#[cfg(feature = "LOSCFD_BASE_MEM_NODE_SIZE_CHECK")]
static mut G_MEM_CHECK_LEVEL: u8 = 0xff;//LOS_MEM_CHECK_LEVEL_DEFAULT

#[cfg(feature = "LOSCFG_MEM_MUL_MODULE")]
static mut G_MODULE_MEM_USED_SIZE: [u32; MEM_MODULE_MAX + 1] = [0; MEM_MODULE_MAX + 1];

#[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
fn Os_Mem_Node_Save(node: &mut LosMemDynNode) {
    // TODO
}

#[inline]
fn os_mem_taskid_set(node: &mut LosMemDynNode, task_id: u32) {
    node.selfNode.taskId = task_id;

    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(node);
    }
}

pub const fn os_mem_taskid_get(node: &LosMemDynNode) -> u32 {
    node.selfNode.taskId
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
                os_mem_taskid_set(node, run_task.taskId);
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
        node.selfNode.taskId = task_id;
    }
}

#[cfg(not(any(feature = "LOSCFG_MEM_DEBUG", feature = "LOSCFG_MEM_TASK_STAT")))]
mod mem_debug_task_stat {
    use crate::LosMemDynNode;

    #[inline]
    pub fn os_mem_set_magic_num_and_task_id(_node: &mut LosMemDynNode) {
        // 如果未定义LOSCFG_MEM_DEBUG和LOSCFG_MEM_TASK_STAT feature，则什么都不做
    }
}
