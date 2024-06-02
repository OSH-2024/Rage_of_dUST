const LOS_OK: u32 = 0;

//处在条件编译内
fn Os_Mem_Integrity_Multi_Check() {
    if Los_Mem_Integrity_Check(M_AUC_SYS_MEM1).is_ok() {
        println!("system memcheck over, all passed!");
        #[cfg(feature = "LOSCFG_SHELL_EXCINFO_DUMP")]
        Write_Exc_Info_To_Buf("system memcheck over, all passed!\n");
    }

    #[cfg(feature = "LOSCFG_EXC_INTERACTION")]
    {
        if Los_Mem_Integrity_Check(M_AUC_SYS_MEM0).is_ok() {
            println!("exc interaction memcheck over, all passed!");
            #[cfg(feature = "LOSCFG_SHELL_EXCINFO_DUMP")]
            Write_Exc_Info_To_Buf("exc interaction memcheck over, all passed!\n");
        }
    }
}

//TODO:这里有个条件编译条件else， 以及下面有#endif， 但#ifdef不在范围中， 最后拼起来的时候需要注意。

fn Los_Mem_Integrity_Check(pool: &[u8]) -> u32 {
    LOS_OK
}
//TODO:与上面方法重名，还不清楚使用逻辑
fn Os_Mem_Integrity_Multi_Check(){}

fn Os_Mem_Node_Debug_Operate(pool: &mut [u8], alloc_node: &mut Los_Mem_Dyn_Node, size: u32) {
    #[cfg(feature = "LOSCFG_MEM_HEAD_BACKUP")]
    {
        Os_Mem_Node_Save(alloc_node);
    }

    #[cfg(feature = "LOSCFG_MEM_LEAKCHECK")]
    {
        Os_Mem_Link_Register_Record(alloc_node);
    }
}

fn Os_Mem_Info_Get(pool: *const c_void, pool_status: &mut Los_Mem_Pool_Status) -> u32 {
    unsafe {
        let pool_info = &*(pool as *const Los_Mem_Pool_Info);
        let mut tmp_node: *mut Los_Mem_Dyn_Node = Os_Mem_End_Node(pool, pool_info.pool_size);
        tmp_node = Os_Mem_Align(tmp_node, OS_MEM_ALIGN_SIZE);

        if !Os_Mem_Magic_Valid((*tmp_node).self_node.magic) {
            Print_Err("wrong mem pool", pool_info, line!() as i32);
            return LOS_NOK;
        }

        let mut total_used_size = 0;
        let mut total_free_size = 0;
        let mut max_free_node_size = 0;
        let mut used_node_num = 0;
        let mut free_node_num = 0;

        tmp_node = Os_Mem_First_Node(pool_info);

        while tmp_node <= Os_Mem_End_Node(pool_info as *const _, pool_info.pool_size) {
            if !Os_Mem_Node_Get_Used_Flag((*tmp_node).self_node.size_and_flag) {
                free_node_num += 1;
                let node_size = Os_Mem_Node_Get_Size((*tmp_node).self_node.size_and_flag);
                total_free_size += node_size;
                if (max_free_node_size < node_size) {
                    max_free_node_size = node_size;
                }
            } else {
                used_node_num += 1;
                total_used_size += Os_Mem_Node_Get_Size((*tmp_node).self_node.size_and_flag);
            }

            tmp_node = Os_Mem_Next_Node(tmp_node);
        }

        pool_status.uw_total_used_size = total_used_size;
        pool_status.uw_total_free_size = total_free_size;
        pool_status.uw_max_free_node_size = max_free_node_size;
        pool_status.uw_used_node_num = used_node_num;
        pool_status.uw_free_node_num = free_node_num;
        #[cfg(feature = "mem_task_stat")]
        {
            pool_status.uw_usage_water_line = pool_info.stat.mem_total_peak;
        }

        LOS_OK
    }
}