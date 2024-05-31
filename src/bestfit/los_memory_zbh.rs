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
    const LOS_OK: u32 = 0;
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