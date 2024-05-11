include!("los_memory_internal_h.rs");


fn Los_List_Init(list: *mut Los_DL_List)->(){
    list.pst_next = list;
    list.pst_prev = list;
}

fn Os_Dlnk_Init_Multi_Head(headaddr: *mut std::ffi::c_void)->(){
    let  dlinkhead: *mut LosMultipleDlinkHead = headaddr as *mut LosMultipleDlinkHead;
    let list_node_head: *mut Los_DL_List = dlinkhead.list_head;
    let index: u32;
    for index in  1..=Os_Multi_Dlnk_Num!() {
        list_node_head += 1;
        Los_List_Init(list_node_head);
    }
}