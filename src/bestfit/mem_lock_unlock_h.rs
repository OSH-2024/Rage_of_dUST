extern crate cortex_m;
use cortex_m::asm;


struct SpinLockS{
    raw_lock: u32   
}

pub static mut g_mem_spin: SpinLockS = SpinLockS{raw_lock: 0};

fn Arch_Int_Lock()->u32{
    let int_save: u32;
    let temp: u32;
    unsafe {
        asm!(
            "mrs    $0, cpsr",
            "orr    $1, $0, #0xc0",
            "msr    cpsr_c, $1",
            lateout("r") int_save,
            lateout("r") temp,
        );
    }
    int_save
}

fn Arch_Int_Restore(int_save: u32) ->(){
    unsafe {
        asm!(
            "msr cpsr_c, $0", // 将 intSave 的值写入 CPSR
            in(reg) int_save, // 使用 in(reg) 来指定输入寄存器
        );
    }
}

fn Los_Int_Lock()-> u32{
    Arch_Int_Lock()
}

fn Los_Int_Restore(int_save: u32)->(){
    Arch_Int_Restore(int_save);
}

fn Los_Spin_Lock_Save(lock: *mut SpinLockS, int_save: *mut u32) ->() {
    ////lock as std::ffi::c_void;
    *int_save = Los_Int_Lock();
}

fn Los_Spin_Unlock_Restore(lock: *mut SpinLockS, int_save: u32) ->() {
    ////lock as std::ffi::c_void;
    Los_Int_Restore(int_save);
}

#[macro_export]
macro_rules! Mem_Lock {
    ($int_save: expr) =>{
        Los_Spin_Lock_Save(std::ptr::addr_of_mut!(g_mem_spin), &mut ($int_save));
    };
}

#[macro_export]
macro_rules! Mem_Unlock {
    ($int_save: expr) =>{
        Los_Spin_Unlock_Restore(std::ptr::addr_of_mut!(g_mem_spin), $int_save);
    };
}