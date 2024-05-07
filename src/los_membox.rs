use std::sync::{Arc, Mutex};
use std::ops::DerefMut;



pub const LOS_OK: u32 = 0;
pub const LOS_NOK: u32 = 1;
pub const OS_MEMBOX_MAGIC: u64 = 0xa55a5aa5a55a5aa5;



struct LosMemboxNode {
    pst_next: Option<Box<LosMemboxNode>>,
    // Add any other fields if needed
}

pub const OS_MEMBOX_NODE_HEAD_SIZE: u32 = std::mem::size_of::<LosMemboxNode>() as u32;

struct LosMemboxInfo {
    st_free_list: Option<Box<LosMemboxNode>>,
    // Add any other fields if needed
    uw_blk_cnt: usize,
    uw_blk_num: usize,
    uw_blk_size: usize
}

macro_rules! los_membox_check_magic {
    ($addr:expr) => {
        {
            match $addr {
                Some(node) => {
                    match node.pst_next {
                        Some(next) if next == OS_MEMBOX_MAGIC as *mut LosMemboxNode => LOS_OK,
                        _ => LOS_NOK,
                    }
                },
                None => LOS_NOK,
            }
        }
    };
}

macro_rules! los_membox_set_magic {
    ($addr:expr) => {
        {
            let node_ptr: *mut LosMemboxNode = $addr as *mut _;
            let magic_value = OS_MEMBOX_MAGIC as *mut LosMemboxNode;
            std::ptr::write(&mut (*node_ptr).pst_next, Some(magic_value));
        }
    };
}



macro_rules! los_membox_user_addr  {
    ($addr:expr) => {
        {
            let addr_ptr: *const u8 = $addr;
            (addr_ptr as usize + OS_MEMBOX_NODE_HEAD_SIZE) as *const u8
        }
    };
}






fn los_check_box_mem(box_info: &LosMemboxInfo, node: &LosMemboxNode) -> u32 {
    if box_info.uw_blk_size == 0 {
        return LOS_NOK;
    }

    let offset = node as *const _ as usize - (&box_info as *const _ as usize + 1);
    if offset % box_info.uw_blk_size != 0 {
        return LOS_NOK;
    }

    if offset / box_info.uw_blk_size >= box_info.uw_blk_num {
        return LOS_NOK;
    }

    los_membox_check_magic!(Some(node))
}

// #define LOS_MEMBOX_ALLIGNED(memAddr) (((UINTPTR)(memAddr) + sizeof(UINTPTR) - 1) & (~(sizeof(UINTPTR) - 1)))
fn los_membox_aligned(size: u32) -> u32 {
    let align = 4;
    (size + align - 1) & !(align - 1)
}

fn los_membox_free(pool: Arc<Mutex<LosMemboxInfo>>, box_slice: *mut u8) -> u32 {
    let guard = pool.lock().unwrap();
    let box_info = guard.deref_mut();

    let node = (box_slice as usize - OS_MEMBOX_NODE_HEAD_SIZE as usize) as *mut LosMemboxNode;
    if los_check_box_mem(box_info, &*node) == LOS_NOK {
        return LOS_NOK;
    }

    los_membox_set_magic!(node);

    let mut st_free_list = box_info.st_free_list.take();
    let mut new_node = LosMemboxNode { pst_next: st_free_list };
    st_free_list = Some(Box::new(new_node));
    box_info.st_free_list = st_free_list;

    box_info.uw_blk_cnt -= 1;

    LOS_OK
}


fn los_membox_init(pool: Arc<Mutex<LosMemboxInfo>>, pool_size: u32, blk_size: u32) -> u32 {

    if pool.is_empty() || blk_size == 0 || pool_size < std::mem::size_of::<LosMemboxInfo>() as u32 {
        return LOS_NOK;
    }

    let guard = pool.lock().unwrap();
    let box_info = guard.deref_mut();

    // let box_info = unsafe { &mut *(pool.as_mut_ptr() as *mut LosMemboxInfo) };
    box_info.uw_blk_size = los_membox_aligned(blk_size + OS_MEMBOX_NODE_HEAD_SIZE);
    box_info.uw_blk_num = (pool_size - std::mem::size_of::<LosMemboxInfo>() as u32) / box_info.uw_blk_size;
    box_info.uw_blk_cnt = 0;

    if box_info.uw_blk_num == 0 || box_info.uw_blk_size < blk_size + OS_MEMBOX_NODE_HEAD_SIZE {
        
        return LOS_NOK;
    }

    box_info.uw_blk_size = los_membox_aligned(blk_size + OS_MEMBOX_NODE_HEAD_SIZE);
    box_info.uw_blk_num = (pool_size - std::mem::size_of::<LosMemboxInfo>() as u32) / box_info.uw_blk_size;
    box_info.uw_blk_cnt = 0;

    if box_info.uw_blk_num == 0 {
        return LOS_NOK;
    }

    let mut node = box_info as *mut LosMemboxNode;
    let mut st_free_list = None;
    for _ in 0..(box_info.uw_blk_num) {
        let new_node = LosMemboxNode { pst_next: st_free_list.take() };
        st_free_list = Some(Box::new(new_node));
    }

    box_info.st_free_list = st_free_list;

    LOS_OK
}






fn los_membox_alloc(pool: Arc<Mutex<LosMemboxInfo>>) -> u32{
    let mut box_info = pool.lock().ok()?;
    if let Some(mut node) = box_info.st_free_list.take() {
        //取出并替换成None
        let node_tmp = node.pst_next.take();
        // Add any necessary logic here
        
        // Update the box info
        box_info.uw_blk_cnt += 1;
        // Put the node back into the pool
        box_info.st_free_list = node_tmp;

        los_membox_set_magic!(node_tmp);
        
        los_membox_user_addr!(node_tmp)
    } else {
        //std::ptr::null()
        LOS_NOK
    }
}

fn los_membox_clr(pool: Arc<Mutex<LosMemboxInfo>>, box_slice : &mut[u8]) {
    let guard = pool.lock().unwrap();

    let box_info = guard.deref_mut();

    if !box_slice.is_empty() {
        let box_size = box_info.uw_blk_size - OS_MEMBOX_NODE_HEAD_SIZE;
        let actual_box_slice = &mut box_slice[..box_size];
        actual_box_slice.iter_mut().for_each(|byte| *byte = 0);
    }
}

fn los_show_box(pool: Arc<Mutex<LosMemboxInfo>>) {
    let guard = pool.lock().unwrap();
    let box_info = guard.deref_mut();
    

    println!("membox(0x{:x},0x{:x}):", box_info.uw_blk_size, box_info.uw_blk_num);
    println!("free node list:");


    let mut index = 0;
    let mut node = &box_info.st_free_list;
    while let Some(n) = node {
        println!("({}, {:p})", index, n);
        node = &n.pst_next;
        index += 1;
    }

    println!("all node list:");
    node = &Some(Box::new(LosMemboxNode { pst_next: None }));
    for i in 0..box_info.uw_blk_num {
        let next = match node {
            Some(n) => n.pst_next.as_ref(),
            None => None,
        };
        println!("({}, {:p}, {:p})", i, node.unwrap(), next);
        node = &next.map_or(None, |n| Some(n.clone()));
    }
}

fn los_membox_statistics_get(box_mem: Option<&LosMemboxInfo>, max_blk: Option<&mut u32>, blk_cnt: Option<&mut u32>, blk_size: Option<&mut u32>) -> u32 {
    match (box_mem, max_blk, blk_cnt, blk_size) {
        (Some(boxs), Some(max), Some(cnt), Some(size)) => {
            *max = boxs.uw_blk_num as u32;
            *cnt = boxs.uw_blk_cnt as u32;
            *size = boxs.uw_blk_size as u32;
            LOS_OK
        },
        _ => LOS_NOK,
    }
}