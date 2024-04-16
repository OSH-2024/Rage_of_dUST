include!("mempool.rs");

const POOL_SIZE: u32 = 16;//pool size

fn main() {
    let next: *mut std::ffi::c_void =std::ptr::null_mut();
    let mut node1: LosMemPoolInfo = LosMemPoolInfo{
        pool: std::ptr::null_mut(),
        next_pool: next, 
        pool_size: 0,
    };
    let mut node2: LosMemPoolInfo = LosMemPoolInfo{
        pool: std::ptr::null_mut(),
        next_pool: next, 
        pool_size: 0,
    };
    let mut node3: LosMemPoolInfo = LosMemPoolInfo{
        pool: std::ptr::null_mut(),
        next_pool: next, 
        pool_size: 0,
    };
    let mut node4: LosMemPoolInfo = LosMemPoolInfo{
        pool: std::ptr::null_mut(),
        next_pool: next, 
        pool_size: 0,
    };
    let mut node5: LosMemPoolInfo = LosMemPoolInfo{
        pool: std::ptr::null_mut(),
        next_pool: next, 
        pool_size: 0,
    };
    let node1_ptr: *mut LosMemPoolInfo = &mut node1 ;
    let node2_ptr: *mut LosMemPoolInfo = &mut node2 ;
    let node3_ptr: *mut LosMemPoolInfo = &mut node3 ;
    let node4_ptr: *mut LosMemPoolInfo = &mut node4 ;
    let node5_ptr: *mut LosMemPoolInfo = &mut node5 ;
    let mut ok_or_not: u32 = 1;
        ok_or_not = unsafe{os_mem_mul_pool_init(node1_ptr as *mut std::ffi::c_void, POOL_SIZE)};
        if ok_or_not == 0{
            println!("内存池节点插入成功！");
        }
        else{
            println!("内存池节点插入失败！");
        }
        unsafe{(*node1_ptr).pool_size = POOL_SIZE;}
        unsafe{(*node1_ptr).pool = node1_ptr as *mut std::ffi::c_void;}
        ok_or_not = 1;
        ok_or_not =unsafe{os_mem_mul_pool_init(node2_ptr as *mut std::ffi::c_void, POOL_SIZE)};
        if ok_or_not == 0{
            println!("内存池节点插入成功！");
        }
        else{
            println!("内存池节点插入失败！");
        }
        unsafe{(*node2_ptr).pool_size = POOL_SIZE;}
        unsafe{(*node2_ptr).pool = node2_ptr as *mut std::ffi::c_void;}
        ok_or_not = 1;
        ok_or_not = unsafe{os_mem_mul_pool_init(node3_ptr as *mut std::ffi::c_void, POOL_SIZE)};
        if ok_or_not == 0{
            println!("内存池节点插入成功！");
        }
        else{
            println!("内存池节点插入失败！");
        }
        unsafe{(*node3_ptr).pool_size = POOL_SIZE;}
        unsafe{(*node3_ptr).pool = node3_ptr as *mut std::ffi::c_void;}
        ok_or_not = 1;
        ok_or_not = unsafe{os_mem_mul_pool_init(node4_ptr as *mut std::ffi::c_void, POOL_SIZE)};
        if ok_or_not == 0{
            println!("内存池节点插入成功！");
        }
        else{
            println!("内存池节点插入失败！");
        }
        unsafe{(*node4_ptr).pool_size = POOL_SIZE;}
        unsafe{(*node4_ptr).pool = node4_ptr as *mut std::ffi::c_void;}
        ok_or_not = 1;
        ok_or_not = unsafe{os_mem_mul_pool_init(node5_ptr as *mut std::ffi::c_void, POOL_SIZE)};
        if ok_or_not == 0{
            println!("内存池节点插入成功！");
        }
        else{
            println!("内存池节点插入失败！");
        }
        unsafe{(*node5_ptr).pool_size = POOL_SIZE;}
        unsafe{(*node5_ptr).pool = node5_ptr as *mut std::ffi::c_void;}
        ok_or_not = 1;

    let head: *mut std::ffi::c_void = os_mem_mul_pool_head_get();
    println!("第一个内存池的大小为：{}",unsafe{los_mempoolsizeget(G_POOL_HEAD)});
    println!("内存池信息节点的头指针的值为{:p}",head);
    println!("内存池信息链表的长度为{}",unsafe{los_mem_pool_list()});
    println!("删除一个内存池信息节点后:");
    ok_or_not = unsafe{os_mem_mul_pool_deinit(node3_ptr as *mut std::ffi::c_void)};
    if ok_or_not == 0{
        println!("内存池节点删除成功！");
    }
    else{
        println!("内存池节点删除失败！");
    }
    println!("删除一个节点后内存池信息链表的长度为{}",unsafe{los_mem_pool_list()});  
}
