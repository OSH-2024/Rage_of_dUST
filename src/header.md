### 关于头文件使用
#### 在los_memory_internal_h.rs中部分结构体成员使用了Cell可变容器，在引用时需要使用.get()获取值， .set(value)修改值，不可直接修改或者使用，请大家注意查看
#### 在mem_lock_unlock_h.rs中，是关于MEMLOCK(intSave) 与 MEMUNLOCK(intSave)的相关实现，大家可以直接调用(没有验证正确性)
#### 在los_memory_h.rs中，定义了LOS_MEM_POOL_STATUS结构体，大家的代码中应该也会用到，所以我单独写了一份以便于统一名称
