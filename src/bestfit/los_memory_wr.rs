include!("los_multipledlinkhead.rs");

struct DlList {
    prev: Option<*mut DlList>,
    next: Option<*mut DlList>,
}


unsafe fn Os_Mem_List_Delete(node: *mut DlList, first_node: *const DlList) {
    let _ = first_node;  // 忽略 first_node 参数
    if !node.is_null() {
        if let Some(next) = (*node).next {
            (*next).prev = (*node).prev;
        }
        if let Some(prev) = (*node).prev {
            (*prev).next = (*node).next;
        }
        (*node).next = None;
        (*node).prev = None;
    }
}


unsafe fn Os_Mem_List_Add(list_node: *mut DlList, node: *mut DlList, first_node: *const DlList) {
    let _ = first_node;  // 忽略 first_node 参数
    if !list_node.is_null() && !node.is_null() {
        (*node).next = (*list_node).next;
        (*node).prev = Some(list_node);
        if let Some(next) = (*list_node).next {
            (*next).prev = Some(node);
        }
        (*list_node).next = Some(node);
    }
}
