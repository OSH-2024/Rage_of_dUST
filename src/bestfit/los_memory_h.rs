include!("los_multipledlinkhead.rs");

struct LosMemPoolStatus{
    uw_total_used_size: Cell<u32>,
    uw_total_free_size: Cell<u32>,
    uw_max_free_node_size: Cell<u32>,
    uw_used_node_num: Cell<u32>,
    uw_free_node_num: Cell<u32>,
    uw_usage_waterline: Cell<u32>

}//LOS_MEM_POOL_STATUS

macro_rules! OS_MEM_ALIGN_SIZE{
    () => {
        std::mem::size_of::<usize>()
    };
}