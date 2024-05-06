/* ----------------------------------------------------------------------------
 * Copyright (c) Huawei Technologies Co., Ltd. 2013-2021. All rights reserved.
 * Description: Interrupt Abstraction Layer And API Implementation
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

#include "los_hwi_pri.h"
#include "los_hwi.h"
#include "los_memory.h"
#include "los_spinlock.h"
#include "los_trace.h"
#ifdef LOSCFG_KERNEL_CPUP
#include "los_cpup_pri.h"
#endif
#ifdef LOSCFG_DEBUG_SCHED_STATISTICS
#include "los_sched_debug_pri.h"
#endif
#include "los_err_pri.h"

#ifdef __cplusplus
#if __cplusplus
extern "C" {
#endif /* __cplusplus */
#endif /* __cplusplus */

/* spinlock for hwi module, only available on SMP mode */
LITE_OS_SEC_BSS  SPIN_LOCK_INIT(g_hwiSpin);

#define HWI_LOCK(state)       LOS_SpinLockSave(&g_hwiSpin, &(state))
#define HWI_UNLOCK(state)     LOS_SpinUnlockRestore(&g_hwiSpin, (state))

size_t g_intCount[LOSCFG_KERNEL_CORE_NUM] = {0};

#ifdef LOSCFG_KERNEL_LOWPOWER
STATIC WAKEUPFROMINTHOOK g_intWakeupHook = NULL;
#endif

const HwiControllerOps *g_hwiOps = NULL;

typedef VOID (*HWI_PROC_FUNC0)(VOID);
typedef VOID (*HWI_PROC_FUNC2)(INT32, VOID *);

STATIC INLINE VOID OsIrqNestingActive(UINT32 hwiNum)
{
#ifdef LOSCFG_ARCH_INTERRUPT_PREEMPTION
    /* preemption not allowed when handling tick interrupt */
    if (hwiNum != OS_TICK_INT_NUM) {
        (VOID)LOS_IntUnLock();
    }
#endif
}

STATIC INLINE VOID OsIrqNestingInactive(UINT32 hwiNum)
{
#ifdef LOSCFG_ARCH_INTERRUPT_PREEMPTION
    if (hwiNum != OS_TICK_INT_NUM) {
        (VOID)LOS_IntLock();
    }
#endif
}

size_t OsIrqNestingCntGet(VOID)
{
    return g_intCount[ArchCurrCpuid()];
}

VOID OsIrqNestingCntSet(size_t val)
{
    g_intCount[ArchCurrCpuid()] = val;
}

STATIC INLINE VOID InterruptHandle(HwiHandleInfo *hwiForm)
{
    hwiForm->respCount++;
#ifdef LOSCFG_SHARED_IRQ
    while (hwiForm->next != NULL) {
        hwiForm = hwiForm->next;
#endif
        if (hwiForm->registerInfo) {
            HWI_PROC_FUNC2 func = (HWI_PROC_FUNC2)hwiForm->hook;
            if (func != NULL) {
                UINTPTR *param = (UINTPTR *)(hwiForm->registerInfo);
                func((INT32)(*param), (VOID *)(*(param + 1)));
            }
        } else {
            HWI_PROC_FUNC0 func = (HWI_PROC_FUNC0)hwiForm->hook;
            if (func != NULL) {
                func();
            }
        }
#ifdef LOSCFG_SHARED_IRQ
    }
#endif
}

VOID OsIntHandle(UINT32 hwiNum, HwiHandleInfo *hwiForm)
{
    size_t *intCnt = NULL;

#ifdef LOSCFG_CPUP_INCLUDE_IRQ
    OsCpupIrqStart();
#endif
    intCnt = &g_intCount[ArchCurrCpuid()];
    *intCnt = *intCnt + 1;

#ifdef LOSCFG_DEBUG_SCHED_STATISTICS
    OsHwiStatistics(hwiNum);
#endif

#ifdef LOSCFG_KERNEL_LOWPOWER
    if (g_intWakeupHook != NULL) {
        g_intWakeupHook(hwiNum);
    }
#endif
    LOS_TRACE(HWI_RESPONSE_IN, hwiNum);

    OsIrqNestingActive(hwiNum);
    InterruptHandle(hwiForm);
    OsIrqNestingInactive(hwiNum);

    LOS_TRACE(HWI_RESPONSE_OUT, hwiNum);

    *intCnt = *intCnt - 1;

#ifdef LOSCFG_CPUP_INCLUDE_IRQ
    OsCpupIrqEnd(hwiNum);
#endif
}

VOID OsIntEntry(VOID)
{
    if ((g_hwiOps != NULL) && (g_hwiOps->handleIrq != NULL)) {
        g_hwiOps->handleIrq();
    }
    return;
}

STATIC HWI_ARG_T OsHwiCpIrqParam(const HWI_IRQ_PARAM_S *irqParam)
{
    HWI_IRQ_PARAM_S *paramByAlloc = NULL;

    paramByAlloc = (HWI_IRQ_PARAM_S *)LOS_MemAlloc(m_aucSysMem0, sizeof(HWI_IRQ_PARAM_S));
    if (paramByAlloc != NULL) {
        (VOID)memcpy_s(paramByAlloc, sizeof(HWI_IRQ_PARAM_S), irqParam, sizeof(HWI_IRQ_PARAM_S));
    }

    return (HWI_ARG_T)paramByAlloc;
}
#ifndef LOSCFG_SHARED_IRQ
STATIC UINT32 OsHwiDel(HwiHandleInfo *hwiForm, const HWI_IRQ_PARAM_S *irqParam, UINT32 irqId)
{
    UINT32 intSave;

    (VOID)irqParam;
    HWI_LOCK(intSave);
    hwiForm->hook = NULL;
    if (hwiForm->registerInfo) {
        (VOID)LOS_MemFree(m_aucSysMem0, (VOID *)hwiForm->registerInfo);
    }
    hwiForm->registerInfo = 0;
    hwiForm->respCount = 0;
    if (g_hwiOps->disableIrq == NULL) {
        HWI_UNLOCK(intSave);
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    g_hwiOps->disableIrq(irqId);

    HWI_UNLOCK(intSave);
    return LOS_OK;
}

STATIC UINT32 OsHwiCreate(HwiHandleInfo *hwiForm, HWI_MODE_T hwiMode, HWI_PROC_FUNC hwiHandler,
                          const HWI_IRQ_PARAM_S *irqParam)
{
    UINT32 intSave;

    if (hwiMode & IRQF_SHARED) {
        return LOS_ERRNO_HWI_SHARED_ERROR;
    }
    HWI_LOCK(intSave);
    if (hwiForm->hook == NULL) {
        hwiForm->hook = hwiHandler;

        if (irqParam != NULL) {
            hwiForm->registerInfo = OsHwiCpIrqParam(irqParam);
            if (hwiForm->registerInfo == (HWI_ARG_T)NULL) {
                HWI_UNLOCK(intSave);
                return LOS_ERRNO_HWI_NO_MEMORY;
            }
        }
    } else {
        HWI_UNLOCK(intSave);
        return LOS_ERRNO_HWI_ALREADY_CREATED;
    }
    HWI_UNLOCK(intSave);
    return LOS_OK;
}
#else /* LOSCFG_SHARED_IRQ */
STATIC INLINE UINT32 OsFreeHwiNode(HwiHandleInfo *head, HwiHandleInfo *hwiForm, UINT32 irqId)
{
    UINT32 ret = LOS_OK;

    if (hwiForm->registerInfo != (HWI_ARG_T)NULL) {
        (VOID)LOS_MemFree(m_aucSysMem0, (VOID *)hwiForm->registerInfo);
    }

    (VOID)LOS_MemFree(m_aucSysMem0, hwiForm);

    if (head->next == NULL) {
        head->shareMode = 0;
        head->respCount = 0;
        if (g_hwiOps->disableIrq == NULL) {
            ret = LOS_ERRNO_HWI_PROC_FUNC_NULL;
            return ret;
        }
        g_hwiOps->disableIrq(irqId);
    }

    return ret;
}

STATIC UINT32 OsHwiDel(HwiHandleInfo *head, const HWI_IRQ_PARAM_S *irqParam, UINT32 irqId)
{
    HwiHandleInfo *hwiFormPrev = NULL;
    HwiHandleInfo *hwiForm = NULL;
    UINT32 intSave;
    UINT32 ret;

    HWI_LOCK(intSave);

    if ((head->shareMode & IRQF_SHARED) && ((irqParam == NULL) || (irqParam->pDevId == NULL))) {
        HWI_UNLOCK(intSave);
        return LOS_ERRNO_HWI_SHARED_ERROR;
    }

    /* Non-shared interrupt. */
    if (!(head->shareMode & IRQF_SHARED)) {
        if (head->next == NULL) {
            HWI_UNLOCK(intSave);
            return LOS_ERRNO_HWI_HWINUM_UNCREATE;
        }

        hwiForm = head->next;
        head->next = NULL;
        ret = OsFreeHwiNode(head, hwiForm, irqId);
        HWI_UNLOCK(intSave);
        return ret;
    }

    /* Shared interrupt. */
    hwiFormPrev = head;
    hwiForm = head->next;
    while (hwiForm != NULL) {
        if (((HWI_IRQ_PARAM_S *)(hwiForm->registerInfo))->pDevId == irqParam->pDevId) {
            break;
        }
        hwiFormPrev = hwiForm;
        hwiForm = hwiForm->next;
    }

    if (hwiForm == NULL) {
        HWI_UNLOCK(intSave);
        return LOS_ERRNO_HWI_HWINUM_UNCREATE;
    }

    hwiFormPrev->next = hwiForm->next;
    ret = OsFreeHwiNode(head, hwiForm, irqId);
    HWI_UNLOCK(intSave);
    return ret;
}

STATIC UINT32 OsHwiCreate(HwiHandleInfo *head, HWI_MODE_T hwiMode, HWI_PROC_FUNC hwiHandler,
                          const HWI_IRQ_PARAM_S *irqParam)
{
    UINT32 intSave;
    HwiHandleInfo *hwiFormNode = NULL;
    HWI_IRQ_PARAM_S *hwiParam = NULL;
    HWI_MODE_T modeResult = hwiMode & IRQF_SHARED;
    HwiHandleInfo *hwiForm = NULL;

    if (modeResult && ((irqParam == NULL) || (irqParam->pDevId == NULL))) {
        return LOS_ERRNO_HWI_SHARED_ERROR;
    }

    HWI_LOCK(intSave);

    if ((head->next != NULL) && ((modeResult == 0) || (!(head->shareMode & IRQF_SHARED)))) {
        HWI_UNLOCK(intSave);
        return LOS_ERRNO_HWI_SHARED_ERROR;
    }

    hwiForm = head;
    while (hwiForm->next != NULL) {
        hwiForm = hwiForm->next;
        hwiParam = (HWI_IRQ_PARAM_S *)(hwiForm->registerInfo);
        if (hwiParam->pDevId == irqParam->pDevId) {
            HWI_UNLOCK(intSave);
            return LOS_ERRNO_HWI_ALREADY_CREATED;
        }
    }

    hwiFormNode = (HwiHandleInfo *)LOS_MemAlloc(m_aucSysMem0, sizeof(HwiHandleInfo));
    if (hwiFormNode == NULL) {
        HWI_UNLOCK(intSave);
        return LOS_ERRNO_HWI_NO_MEMORY;
    }
    hwiForm->respCount = 0;

    if (irqParam != NULL) {
        hwiFormNode->registerInfo = OsHwiCpIrqParam(irqParam);
        if (hwiFormNode->registerInfo == (HWI_ARG_T)NULL) {
            HWI_UNLOCK(intSave);
            (VOID) LOS_MemFree(m_aucSysMem0, hwiFormNode);
            return LOS_ERRNO_HWI_NO_MEMORY;
        }
    } else {
        hwiFormNode->registerInfo = 0;
    }

    hwiFormNode->hook = hwiHandler;
    hwiFormNode->next = (struct tagHwiHandleForm *)NULL;
    hwiForm->next = hwiFormNode;

    head->shareMode = modeResult;

    HWI_UNLOCK(intSave);
    return LOS_OK;
}
#endif

size_t IntActive()
{
    size_t intCount;
    UINT32 intSave = LOS_IntLock();

    intCount = g_intCount[ArchCurrCpuid()];
    LOS_IntRestore(intSave);
    return intCount;
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiCreate(HWI_HANDLE_T hwiNum,
                                           HWI_PRIOR_T hwiPrio,
                                           HWI_MODE_T hwiMode,
                                           HWI_PROC_FUNC hwiHandler,
                                           HWI_IRQ_PARAM_S *irqParam)
{
    UINT32 ret;
    HwiHandleInfo *hwiForm = NULL;

    if (hwiHandler == NULL) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }

    if ((g_hwiOps == NULL) || (g_hwiOps->getHandleForm == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }

    hwiForm = g_hwiOps->getHandleForm(hwiNum);
    if (hwiForm == NULL) {
        return LOS_ERRNO_HWI_NUM_INVALID;
    }
    LOS_TRACE(HWI_CREATE, hwiNum, hwiPrio, hwiMode, (UINTPTR)hwiHandler);

    ret = OsHwiCreate(hwiForm, hwiMode, hwiHandler, irqParam);
    LOS_TRACE(HWI_CREATE_SHARE, hwiNum, (UINTPTR)(irqParam != NULL ? irqParam->pDevId : NULL), ret);

    /* priority will be changed if setIrqPriority implemented,
     * but interrupt preemption only allowed when LOSCFG_ARCH_INTERRUPT_PREEMPTION enable */
    if ((ret == LOS_OK) && (g_hwiOps->setIrqPriority != NULL)) {
        ret = g_hwiOps->setIrqPriority(hwiNum, hwiPrio);
        if (ret != LOS_OK) {
            (VOID)OsHwiDel(hwiForm, irqParam, hwiNum);
        }
    }
    return ret;
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiDelete(HWI_HANDLE_T hwiNum, HWI_IRQ_PARAM_S *irqParam)
{
    UINT32 ret;
    HwiHandleInfo *hwiForm = NULL;

    if ((g_hwiOps == NULL) || (g_hwiOps->getHandleForm == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }

    hwiForm = g_hwiOps->getHandleForm(hwiNum);
    if (hwiForm == NULL) {
        return LOS_ERRNO_HWI_NUM_INVALID;
    }
    LOS_TRACE(HWI_DELETE, hwiNum);

    ret = OsHwiDel(hwiForm, irqParam, hwiNum);
    LOS_TRACE(HWI_DELETE_SHARE, hwiNum, (UINTPTR)(irqParam != NULL ? irqParam->pDevId : NULL), ret);

    return ret;
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiTrigger(HWI_HANDLE_T hwiNum)
{
    if ((g_hwiOps == NULL) || (g_hwiOps->triggerIrq == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    LOS_TRACE(HWI_TRIGGER, hwiNum);

    return g_hwiOps->triggerIrq(hwiNum);
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiEnable(HWI_HANDLE_T hwiNum)
{
    if ((g_hwiOps == NULL) || (g_hwiOps->enableIrq == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    LOS_TRACE(HWI_ENABLE, hwiNum);

    return g_hwiOps->enableIrq(hwiNum);
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiDisable(HWI_HANDLE_T hwiNum)
{
    if ((g_hwiOps == NULL) || (g_hwiOps->disableIrq == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    LOS_TRACE(HWI_DISABLE, hwiNum);

    return g_hwiOps->disableIrq(hwiNum);
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiClear(HWI_HANDLE_T hwiNum)
{
    if ((g_hwiOps == NULL) || (g_hwiOps->clearIrq == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    LOS_TRACE(HWI_CLEAR, hwiNum);

    return g_hwiOps->clearIrq(hwiNum);
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiSetPriority(HWI_HANDLE_T hwiNum, HWI_PRIOR_T priority)
{
    if ((g_hwiOps == NULL) || (g_hwiOps->setIrqPriority == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    LOS_TRACE(HWI_SETPRI, hwiNum, priority);

    return g_hwiOps->setIrqPriority(hwiNum, priority);
}
#ifdef LOSCFG_KERNEL_SMP
LITE_OS_SEC_TEXT UINT32 LOS_HwiSetAffinity(HWI_HANDLE_T hwiNum, UINT32 cpuMask)
{
    if ((g_hwiOps == NULL) || (g_hwiOps->setIrqCpuAffinity == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    LOS_TRACE(HWI_SETAFFINITY, hwiNum, cpuMask);

    return g_hwiOps->setIrqCpuAffinity(hwiNum, cpuMask);
}

LITE_OS_SEC_TEXT UINT32 LOS_HwiSendIpi(HWI_HANDLE_T hwiNum, UINT32 cpuMask)
{
    if ((g_hwiOps == NULL) || (g_hwiOps->sendIpi == NULL)) {
        return LOS_ERRNO_HWI_PROC_FUNC_NULL;
    }
    LOS_TRACE(HWI_SENDIPI, hwiNum, cpuMask);

    return g_hwiOps->sendIpi(cpuMask, hwiNum);
}
#endif

#ifdef LOSCFG_KERNEL_LOWPOWER
LITE_OS_SEC_TEXT_MINOR VOID LOS_IntWakeupHookReg(WAKEUPFROMINTHOOK hook)
{
    g_intWakeupHook = hook;
}
#endif

/* Initialization of the hardware interrupt */
LITE_OS_SEC_TEXT_INIT VOID OsHwiInit(VOID)
{
    ArchIrqInit();
    return;
}

#ifdef __cplusplus
#if __cplusplus
}
#endif /* __cplusplus */
#endif /* __cplusplus */
