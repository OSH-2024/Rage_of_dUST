
include!("membox.rs");

 fn main() {
    unsafe
    {
        let mut mem: *mut u32 = std::ptr::null_mut();
        let blk_size: u32 = 16;
        //这个blk_size不能轻易改！！！
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
