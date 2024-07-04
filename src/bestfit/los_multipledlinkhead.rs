include!("mempool.rs");


fn Los_List_Init(list: *mut LosDlList)->(){
    (*list).pst_next = list;
    (*list).pst_prev = list;
}

fn Os_Dlnk_Init_Multi_Head(headaddr: *mut std::ffi::c_void)->(){
    let dlinkhead: *mut LosMultipleDlinkHead = headaddr as *mut LosMultipleDlinkHead;
    let mut list_node_head: *mut LosDlList = &mut (*dlinkhead).list_head[0];
    let mut index: u32;
    for index in  1..=Os_Multi_Dlnk_Num!() {
        list_node_head = unsafe { list_node_head.offset(1) };
        Los_List_Init(list_node_head);
    }
}

fn Los_High_Bit_Get(bit_map: u32) -> u32{
    if bit_map == 0{
        return los_invalid_bit_index;
    }
    os_bitmap_mask - bit_map.leading_zeros()
}

fn OsLog2(size: u32) -> u32{
    if size > 0 {
        return Los_High_Bit_Get(size);
    }
    0
}

fn Os_Dlnk_Multi_Head(headaddr: *mut std::ffi::c_void, size: u32) -> *mut LosDlList{
    let  dlinkhead: *mut LosMultipleDlinkHead = headaddr as *mut LosMultipleDlinkHead;
    let mut index: u32 = OsLog2(size);
    if index > Os_Max_Multi_Dlnk_Log2!() {
        return std::ptr::null_mut();
    }
    else if index <= Os_Min_Multi_Dlnk_Log2!(){
        index = Os_Min_Multi_Dlnk_Log2!();
    }
    &mut (*dlinkhead).list_head[(index - Os_Min_Multi_Dlnk_Log2!())as usize]
}

fn Os_Dlnk_Next_Multi_Head(head_addr: *mut std::ffi::c_void, list_node_head: *mut LosDlList) -> *mut LosDlList{
    let head: *mut LosMultipleDlinkHead = head_addr as *mut LosMultipleDlinkHead;
    if (&mut (*head).list_head[Os_Multi_Dlnk_Num!() -1] )as *mut LosDlList == list_node_head {
        return std::ptr::null_mut();
    }
    list_node_head.offset(1) 
}