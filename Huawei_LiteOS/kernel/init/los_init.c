/* ----------------------------------------------------------------------------
 * Copyright (c) Huawei Technologies Co., Ltd. 2013-2020. All rights reserved.
 * Description: System Init Implementation
 * Author: Huawei LiteOS Team
 * Create: 2013-01-01
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright notice, this list of
 * conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright notice, this list
 * of conditions and the following disclaimer in the documentation and/or other materials
 * provided with the distribution.
 * 3. Neither the name of the copyright holder nor the names of its contributors may be used
 * to endorse or promote products derived from this software without specific prior written
 * permission.
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
 * THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS;
 * OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 * WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR
 * OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
 * ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 * --------------------------------------------------------------------------- */

#include "los_config.h"
#include "stdio.h"
#ifdef LOSCFG_COMPAT_LINUX
#include "linux/workqueue.h"
#include "linux/module.h"
#endif
#include "los_sys.h"
#include "los_tick_pri.h"
#include "los_task_pri.h"
#include "los_printf.h"
#include "los_swtmr.h"
#include "los_swtmr_pri.h"
#include "los_sched_pri.h"
#include "los_memory_pri.h"
#include "los_sem_pri.h"
#include "los_mux_pri.h"
#include "los_queue_pri.h"
#include "los_hwi_pri.h"
#include "los_spinlock.h"
#include "los_mp_pri.h"
#include "gic_common.h"

#ifdef LOSCFG_FS_VFS
#include "fs/fs.h"
#endif

#include "los_trace.h"
#include "los_perf.h"

#ifdef LOSCFG_KERNEL_CPUP
#include "los_cpup_pri.h"
#endif

#ifdef LOSCFG_KERNEL_TICKLESS
#include "los_tickless.h"
#endif

#ifdef LOSCFG_KERNEL_DYNLOAD
#include "los_ld_initlib_pri.h"
#endif

#ifdef LOSCFG_KERNEL_RUNSTOP
#include "lowpower/los_runstop_pri.h"
#endif

#ifdef LOSCFG_KERNEL_INTERMIT
#include "los_intermit_pri.h"
#endif

#ifdef LOSCFG_KERNEL_LMS
#include "los_lms_pri.h"
#endif

#ifdef LOSCFG_SHELL_DMESG
#include "dmesg_pri.h"
#endif
#ifdef LOSCFG_SHELL_LK
#include "shell_pri.h"
#endif

#ifndef LOSCFG_PLATFORM_OSAPPINIT
#include "los_test_pri.h"
#endif
#ifdef LOSCFG_DRIVERS_BASE
#include "los_driverbase_pri.h"
#endif

#if defined(LOSCFG_KERNEL_TICKLESS) && !defined(LOSCFG_KERNEL_POWER_MGR)
#include "los_lowpower_pri.h"
#endif

#include "arch/exception.h"

#ifdef __cplusplus
#if __cplusplus
extern "C" {
#endif /* __cplusplus */
#endif /* __cplusplus */

/* temp task blocks for booting procedure */
LITE_OS_SEC_BSS STATIC LosTaskCB                g_mainTask[LOSCFG_KERNEL_CORE_NUM];

VOID *OsGetMainTask()
{
    return (g_mainTask + ArchCurrCpuid());
}

VOID OsSetMainTask()
{
    UINT32 i;
    for (i = 0; i < LOSCFG_KERNEL_CORE_NUM; i++) {
        g_mainTask[i].taskStatus = OS_TASK_STATUS_UNUSED;
        g_mainTask[i].taskId = LOSCFG_BASE_CORE_TSK_LIMIT;
        g_mainTask[i].priority = OS_TASK_PRIORITY_LOWEST + 1;
        g_mainTask[i].taskName = "osMain";
#ifdef LOSCFG_KERNEL_SMP_LOCKDEP
        g_mainTask[i].lockDep.lockDepth = 0;
        g_mainTask[i].lockDep.waitLock = NULL;
#endif
    }
}

LITE_OS_SEC_TEXT_INIT STATIC VOID OsRegister(VOID)
{
#ifdef LOSCFG_LIB_CONFIGURABLE
    g_osSysClock            = OS_SYS_CLOCK_CONFIG;
    g_taskLimit             = LOSCFG_BASE_CORE_TSK_LIMIT;
    g_taskMaxNum            = g_taskLimit + 1;
    g_taskMinStkSize        = LOSCFG_BASE_CORE_TSK_MIN_STACK_SIZE;
    g_taskIdleStkSize       = LOSCFG_BASE_CORE_TSK_IDLE_STACK_SIZE;
    g_taskDfltStkSize       = LOSCFG_BASE_CORE_TSK_DEFAULT_STACK_SIZE;
    g_taskSwtmrStkSize      = LOSCFG_BASE_CORE_TSK_SWTMR_STACK_SIZE;
    g_swtmrLimit            = LOSCFG_BASE_CORE_SWTMR_LIMIT;
    g_semLimit              = LOSCFG_BASE_IPC_SEM_LIMIT;
    g_muxLimit              = LOSCFG_BASE_IPC_MUX_LIMIT;
    g_queueLimit            = LOSCFG_BASE_IPC_QUEUE_LIMIT;
#ifdef LOSCFG_BASE_CORE_TIMESLICE
    g_timeSliceTimeOut      = LOSCFG_BASE_CORE_TIMESLICE_TIMEOUT;
#endif
#endif
    g_tickPerSecond         = LOSCFG_BASE_CORE_TICK_PER_SECOND;
    SET_SYS_CLOCK(OS_SYS_CLOCK);

#ifdef LOSCFG_KERNEL_NX
    LOS_SET_NX_CFG(true);
#else
    LOS_SET_NX_CFG(false);
#endif
    LOS_SET_DL_NX_HEAP_BASE(LOS_DL_HEAP_BASE);
    LOS_SET_DL_NX_HEAP_SIZE(LOS_DL_HEAP_SIZE);

    return;
}

LITE_OS_SEC_TEXT_INIT VOID OsStart(VOID)
{
    LosTaskCB *taskCB = NULL;
    UINT32 cpuid = ArchCurrCpuid();

    OsTickStart();

    LOS_SpinLock(&g_taskSpin);
    taskCB = OsGetTopTask();

#ifdef LOSCFG_KERNEL_SMP
    /*
     * attention: current cpu needs to be set, in case first task deletion
     * may fail because this flag mismatch with the real current cpu.
     */
    taskCB->currCpu = (UINT16)cpuid;
#endif
    OS_SCHEDULER_SET(cpuid);

    PRINTK("cpu %u entering scheduler\n", cpuid);

    taskCB->taskStatus = OS_TASK_STATUS_RUNNING;

    OsStartToRun(taskCB);
}

LITE_OS_SEC_TEXT_INIT STATIC UINT32 OsIpcInit(VOID)
{
    UINT32 ret = LOS_OK;
#ifdef LOSCFG_BASE_IPC_SEM
    ret = OsSemInit();
    if (ret != LOS_OK) {
        PRINT_ERR("Sem init err.\n");
        return ret;
    }
#endif

#ifdef LOSCFG_BASE_IPC_MUX
    ret = OsMuxInit();
    if (ret != LOS_OK) {
        PRINT_ERR("Mux init err.\n");
        return ret;
    }
#endif

#ifdef LOSCFG_BASE_IPC_QUEUE
    ret = OsQueueInit();
    if (ret != LOS_OK) {
        PRINT_ERR("Que init err.\n");
        return ret;
    }
#endif
    return ret;
}

#ifdef LOSCFG_PLATFORM_OSAPPINIT
STATIC UINT32 OsAppTaskCreate(VOID)
{
    UINT32 taskId;
    TSK_INIT_PARAM_S appTask;

    (VOID)memset_s(&appTask, sizeof(TSK_INIT_PARAM_S), 0, sizeof(TSK_INIT_PARAM_S));
    appTask.pfnTaskEntry = (TSK_ENTRY_FUNC)app_init;
    appTask.uwStackSize = LOSCFG_BASE_CORE_TSK_DEFAULT_STACK_SIZE;
    appTask.pcName = "app_Task";
    appTask.usTaskPrio = LOSCFG_BASE_CORE_TSK_DEFAULT_PRIO;
    appTask.uwResved = LOS_TASK_STATUS_DETACHED;
#ifdef LOSCFG_KERNEL_SMP
    appTask.usCpuAffiMask = CPUID_TO_AFFI_MASK(ArchCurrCpuid());
#endif
    return LOS_TaskCreate(&taskId, &appTask);
}

UINT32 OsAppInit(VOID)
{
    UINT32 ret;
#ifdef LOSCFG_FS_VFS
    los_vfs_init();
#endif

#ifdef LOSCFG_COMPAT_LINUX

#ifdef LOSCFG_COMPAT_LINUX_HRTIMER
    ret = HrtimersInit();
    if (ret != LOS_OK) {
        PRINT_ERR("HrtimersInit error\n");
        return ret;
    }
#endif
#ifdef LOSCFG_BASE_CORE_SWTMR
    g_pstSystemWq = create_workqueue("system_wq");
#endif

#endif

    ret = OsAppTaskCreate();
    PRINTK("OsAppInit\n");
    if (ret != LOS_OK) {
        return ret;
    }

#ifdef LOSCFG_KERNEL_TICKLESS
    LOS_TicklessEnable();
#endif
    return 0;
}
#endif /* LOSCFG_PLATFORM_OSAPPINIT */

LITE_OS_SEC_TEXT_INIT UINT32 OsMain(VOID)
{
    UINT32 ret;

#ifdef LOSCFG_EXC_INTERACTION
    ret = OsMemExcInteractionInit((UINTPTR)&__exc_heap_start);
    if (ret != LOS_OK) {
        return ret;
    }
#endif

#ifdef LOSCFG_KERNEL_LMS
    OsLmsInit();
#endif

    ret = OsMemSystemInit((UINTPTR)OS_SYS_MEM_ADDR);
    if (ret != LOS_OK) {
        PRINT_ERR("Mem init err.\n");
        return ret;
    }

    OsRegister();

#ifdef LOSCFG_SHELL_LK
    OsLkLoggerInit(NULL);
#endif

#ifdef LOSCFG_SHELL_DMESG
    ret = OsDmesgInit();
    if (ret != LOS_OK) {
        return ret;
    }
#endif

    OsHwiInit();

    ArchExcInit();

#if defined(LOSCFG_KERNEL_TICKLESS) && !defined(LOSCFG_KERNEL_POWER_MGR)
    OsLowpowerInit(NULL);
#endif

    ret = OsTickInit(GET_SYS_CLOCK(), LOSCFG_BASE_CORE_TICK_PER_SECOND);
    if (ret != LOS_OK) {
        PRINT_ERR("Tick init err.\n");
        return ret;
    }

#if defined(LOSCFG_DRIVERS_UART) || defined(LOSCFG_DRIVERS_SIMPLE_UART)
    uart_init();
#ifdef LOSCFG_SHELL
    uart_hwiCreate();
#endif /* LOSCFG_SHELL */
#endif /* LOSCFG_DRIVERS_SIMPLE_UART */

    ret = OsTaskInit();
    if (ret != LOS_OK) {
        PRINT_ERR("Task init err.\n");
        return ret;
    }

#ifdef LOSCFG_KERNEL_TRACE
    ret = LOS_TraceInit(NULL, LOS_TRACE_BUFFER_SIZE);
    if (ret != LOS_OK) {
        PRINT_ERR("Trace init err.\n");
        return ret;
    }
#endif

#ifdef LOSCFG_BASE_CORE_TSK_MONITOR
    OsTaskMonInit();
#endif

    ret = OsIpcInit();
    if (ret != LOS_OK) {
        return ret;
    }

    /*
     * CPUP should be inited before first task creation which depends on the semaphore
     * when LOSCFG_KERNEL_SMP_TASK_SYNC is enabled. So don't change this init sequence
     * if not necessary. The sequence should be like this:
     * 1. OsIpcInit
     * 2. OsCpupInit -> has first task creation
     * 3. other inits have task creation
     */
#ifdef LOSCFG_KERNEL_CPUP
    ret = OsCpupInit();
    if (ret != LOS_OK) {
        PRINT_ERR("Cpup init err.\n");
        return ret;
    }
#endif

#ifdef LOSCFG_BASE_CORE_SWTMR
    ret = OsSwtmrInit();
    if (ret != LOS_OK) {
        PRINT_ERR("Swtmr init err.\n");
        return ret;
    }
#endif

#ifdef LOSCFG_KERNEL_SMP
    (VOID)OsMpInit();
#endif

#ifdef LOSCFG_KERNEL_DYNLOAD
    ret = OsDynloadInit();
    if (ret != LOS_OK) {
        return ret;
    }
#endif

    ret = OsIdleTaskCreate();
    if (ret != LOS_OK) {
        PRINT_ERR("Create idle task err.\n");
        return ret;
    }

#ifdef LOSCFG_KERNEL_RUNSTOP
    ret = OsWowWriteFlashTaskCreate();
    if (ret != LOS_OK) {
        return ret;
    }
#endif

#ifdef LOSCFG_KERNEL_INTERMIT
    ret = OsIntermitDaemonStart();
    if (ret != LOS_OK) {
        return ret;
    }
#endif

#ifdef LOSCFG_DRIVERS_BASE
    ret = OsDriverBaseInit();
    if (ret != LOS_OK) {
        return ret;
    }
#ifdef LOSCFG_COMPAT_LINUX
    (VOID)do_initCalls(LEVEL_ARCH);
#endif
#endif

#ifdef LOSCFG_KERNEL_PERF
    ret = LOS_PerfInit(NULL, LOS_PERF_BUFFER_SIZE);
    if (ret != LOS_OK) {
        return ret;
    }
#endif

#ifdef LOSCFG_PLATFORM_OSAPPINIT
    ret = OsAppInit();
#else /* LOSCFG_TEST */
    ret = OsTestInit();
#endif
    if (ret != LOS_OK) {
        return ret;
    }

    return LOS_OK;
}

#ifdef __cplusplus
#if __cplusplus
}
#endif /* __cplusplus */
#endif /* __cplusplus */
