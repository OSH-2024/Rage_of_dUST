
include!("los_memory_internal_h.rs");

#[macro_use]
mod los_memory_internal_h;

struct LosMemPoolInfo {
    pool: *mut std::ffi::c_void,//begining address of this pool
    next_pool: *mut std::ffi::c_void, 
    pool_size: u32
    // Add other fields if necessary
    //memstat add after changing memstat
    //slab related
}

pub const LOS_OK: u32 = 0;
pub const LOS_NOK: u32 = 1;
pub const OS_NULL_INT: u32 = 0xFFFFFFFF;

