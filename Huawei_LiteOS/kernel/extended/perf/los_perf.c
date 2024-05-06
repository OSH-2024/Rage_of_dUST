/*----------------------------------------------------------------------------
 * Copyright (c) Huawei Technologies Co., Ltd. 2017-2020. All rights reserved.
 * Description: LiteOS Performance Monitor Module Implementation
 * Author: Huawei LiteOS Team
 * Create: 2017-01-01
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
 *---------------------------------------------------------------------------*/

#include "los_perf_pri.h"
#include "perf_pmu_pri.h"
#include "perf_output_pri.h"

#ifdef __cplusplus
#if __cplusplus
extern "C" {
#endif /* __cplusplus */
#endif /* __cplusplus */

#ifdef LOSCFG_KERNEL_PERF
STATIC Pmu *g_pmu = NULL;
STATIC PerfCB g_perfCb = {0};

LITE_OS_SEC_BSS SPIN_LOCK_INIT(g_perfSpin);
#define PERF_LOCK(state)       LOS_SpinLockSave(&g_perfSpin, &(state))
#define PERF_UNLOCK(state)     LOS_SpinUnlockRestore(&g_perfSpin, (state))

#define MIN(x, y)             ((x) < (y) ? (x) : (y))

STATIC INLINE UINT64 OsPerfGetCurrTime(VOID)
{
#ifdef LOSCFG_PERF_CALC_TIME_BY_TICK
    return LOS_TickCountGet();
#else
    return HalClockGetCycles();
#endif
}

STATIC UINT32 OsPmuInit(VOID)
{
#ifdef LOSCFG_PERF_HW_PMU
    if (OsHwPmuInit() != LOS_OK) {
        return LOS_ERRNO_PERF_HW_INIT_ERROR;
    }
#endif

#ifdef LOSCFG_PERF_TIMED_PMU
    if (OsTimedPmuInit() != LOS_OK) {
        return LOS_ERRNO_PERF_TIMED_INIT_ERROR;
    }
#endif

#ifdef LOSCFG_PERF_SW_PMU
    if (OsSwPmuInit() != LOS_OK) {
        return LOS_ERRNO_PERF_SW_INIT_ERROR;
    }
#endif
    return LOS_OK;
}

STATIC UINT32 OsPerfConfig(PerfEventConfig *eventsCfg)
{
    UINT32 i;
    UINT32 ret;

    g_pmu = OsPerfPmuGet(eventsCfg->type);
    if (g_pmu == NULL) {
        PRINT_ERR("perf config type error %u!\n", eventsCfg->type);
        return LOS_ERRNO_PERF_INVALID_PMU;
    }

    UINT32 eventNum = MIN(eventsCfg->eventsNr, PERF_MAX_EVENT);

    (VOID)memset_s(&g_pmu->events, sizeof(PerfEvent), 0, sizeof(PerfEvent));

    for (i = 0; i < eventNum; i++) {
        g_pmu->events.per[i].eventId = eventsCfg->events[i].eventId;
        g_pmu->events.per[i].period = eventsCfg->events[i].period;
    }
    g_pmu->events.nr = i;
    g_pmu->events.cntDivided = eventsCfg->predivided;
    g_pmu->type = eventsCfg->type;

    ret = g_pmu->config();
    if (ret != LOS_OK) {
        PRINT_ERR("perf config failed!\n");
        (VOID)memset_s(&g_pmu->events, sizeof(PerfEvent), 0, sizeof(PerfEvent));
        return LOS_ERRNO_PERF_PMU_CONFIG_ERROR;
    }
    return LOS_OK;
}

STATIC VOID OsPerfPrintCount(VOID)
{
    UINT32 index;
    UINT32 intSave;
    UINT32 cpuid = ArchCurrCpuid();

    PerfEvent *events = &g_pmu->events;
    UINT32 eventNum = events->nr;

    PERF_LOCK(intSave);
    for (index = 0; index < eventNum; index++) {
        Event *event = &(events->per[index]);

        /* filter out event counter with no event binded. */
        if (event->period == 0) {
            continue;
        }
        PRINT_EMG("[%s] eventType: 0x%x [core %u]: %llu\n", g_pmu->getName(event), event->eventId, cpuid,
            event->count[cpuid]);
    }
    PERF_UNLOCK(intSave);
}

STATIC INLINE VOID OsPerfPrintTs(VOID)
{
#ifdef LOSCFG_PERF_CALC_TIME_BY_TICK
    DOUBLE time = (g_perfCb.endTime - g_perfCb.startTime) * 1.0 / LOSCFG_BASE_CORE_TICK_PER_SECOND;
#else
    DOUBLE time = (g_perfCb.endTime - g_perfCb.startTime) * 1.0 / OS_SYS_CLOCK;
#endif
    PRINT_EMG("time used: %.6f(s)\r\n", time);
}

STATIC VOID OsPerfStart(VOID)
{
    UINT32 cpuid = ArchCurrCpuid();

    if (g_pmu == NULL) {
        PRINT_ERR("pmu not registered!\n");
        return;
    }

    if (g_perfCb.pmuStatusPerCpu[cpuid] != PERF_PMU_STARTED) {
        UINT32 ret = g_pmu->start();
        if (ret != LOS_OK) {
            PRINT_ERR("perf start on core:%u failed, ret = 0x%x\n", cpuid, ret);
            return;
        }

        g_perfCb.pmuStatusPerCpu[cpuid] = PERF_PMU_STARTED;
    } else {
        PRINT_ERR("percpu status err %d\n", g_perfCb.pmuStatusPerCpu[cpuid]);
    }
}

STATIC VOID OsPerfStop(VOID)
{
    UINT32 cpuid = ArchCurrCpuid();

    if (g_pmu == NULL) {
        PRINT_ERR("pmu not registered!\n");
        return;
    }

    if (g_perfCb.pmuStatusPerCpu[cpuid] != PERF_PMU_STOPED) {
        UINT32 ret = g_pmu->stop();
        if (ret != LOS_OK) {
            PRINT_ERR("perf stop on core:%u failed, ret = 0x%x\n", cpuid, ret);
            return;
        }

        if (!g_perfCb.needSample) {
            OsPerfPrintCount();
        }

        g_perfCb.pmuStatusPerCpu[cpuid] = PERF_PMU_STOPED;
    } else {
        PRINT_ERR("percpu status err %d\n", g_perfCb.pmuStatusPerCpu[cpuid]);
    }
}

STATIC UINT32 OsPerfBackTrace(UINTPTR *callChain, UINT32 maxDepth, PerfRegs *regs)
{
    UINT32 i;

    UINT32 count = ArchBackTraceGet(regs->fp, callChain, maxDepth);
    PRINT_DEBUG("backtrace depth = %u, fp = 0x%x\n", count, regs->fp);

    for (i = 0; i < count; i++) {
        PRINT_DEBUG("ip[%u]: 0x%x\n", i, callChain[i]);
    }
    return count;
}

STATIC UINT32 OsPerfCollectData(Event *event, PerfSampleData *data, PerfRegs *regs)
{
    UINT32 size = 0;
    UINT32 depth;
    UINT32 sampleType = g_perfCb.sampleType;
    CHAR *p = (CHAR *)data;

    if (sampleType & PERF_RECORD_CPU) {
        *(UINT32 *)(p + size) = ArchCurrCpuid();
        size += sizeof(data->cpuid);
    }

    if (sampleType & PERF_RECORD_TID) {
        *(UINT32 *)(p + size) = LOS_CurTaskIDGet();
        size += sizeof(data->taskId);
    }

    if (sampleType & PERF_RECORD_TYPE) {
        *(UINT32 *)(p + size) = event->eventId;
        size += sizeof(data->eventId);
    }

    if (sampleType & PERF_RECORD_PERIOD) {
        *(UINT32 *)(p + size) = event->period;
        size += sizeof(data->period);
    }

    if (sampleType & PERF_RECORD_TIMESTAMP) {
        *(UINT64 *)(p + size) = OsPerfGetCurrTime();
        size += sizeof(data->time);
    }

    if (sampleType & PERF_RECORD_IP) {
        *(UINTPTR *)(p + size) = regs->pc;
        size += sizeof(data->pc);
    }

    if (sampleType & PERF_RECORD_CALLCHAIN) {
        depth = OsPerfBackTrace((UINTPTR *)(p + size + sizeof(data->callChain.ipNr)), PERF_MAX_CALLCHAIN_DEPTH, regs);
        *(UINT32 *)(p + size) = depth;
        size += sizeof(data->callChain.ipNr) + depth * sizeof(data->callChain.ip[0]);
    }

    return size;
}

/*
 * return TRUE if the taskId in the taskId list, return FALSE otherwise;
 * return TRUE if user haven't specified any taskId(which is supposed
 * to instrument the whole system)
 */
STATIC BOOL OsPerfTaskFilter(UINT32 taskId)
{
    UINT32 i;

    if (!g_perfCb.taskIdsNr) {
        return TRUE;
    }

    for (i = 0; i < g_perfCb.taskIdsNr; i++) {
        if (g_perfCb.taskIds[i] == taskId) {
            return TRUE;
        }
    }
    return FALSE;
}

STATIC INLINE UINT32 OsPerfParamValid(VOID)
{
    UINT32 index;
    UINT32 res = 0;

    if (g_pmu == NULL) {
        return 0;
    }
    PerfEvent *events = &g_pmu->events;
    UINT32 eventNum = events->nr;

    for (index = 0; index < eventNum; index++) {
        res |= events->per[index].period;
    }
    return res;
}

STATIC UINT32 OsPerfHdrInit(UINT32 id)
{
    PerfDataHdr head = {
        .magic      = PERF_DATA_MAGIC_WORD,
        .sampleType = g_perfCb.sampleType,
        .sectionId  = id,
        .eventType  = g_pmu->type,
        .len        = sizeof(PerfDataHdr),
    };
    return OsPerfOutPutWrite((CHAR *)&head, head.len);
}

VOID OsPerfUpdateEventCount(Event *event, UINT32 value)
{
    if (event == NULL) {
        return;
    }
    event->count[ArchCurrCpuid()] += (value & 0xFFFFFFFF); /* event->count is UINT64 */
}

VOID OsPerfHandleOverFlow(Event *event, PerfRegs *regs)
{
    PerfSampleData data;
    UINT32 len;

    (VOID)memset_s(&data, sizeof(PerfSampleData), 0, sizeof(PerfSampleData));
    if ((g_perfCb.needSample) && OsPerfTaskFilter(LOS_CurTaskIDGet())) {
        len = OsPerfCollectData(event, &data, regs);
        OsPerfOutPutWrite((CHAR *)&data, len);
    }
}

UINT32 LOS_PerfInit(VOID *buf, UINT32 size)
{
    UINT32 ret;
    UINT32 intSave;

    PERF_LOCK(intSave);
    if (g_perfCb.status != PERF_UNINIT) {
        ret = LOS_ERRNO_PERF_STATUS_INVALID;
        goto PERF_INIT_ERROR;
    }

    ret = OsPmuInit();
    if (ret != LOS_OK) {
        goto PERF_INIT_ERROR;
    }

    ret = OsPerfOutPutInit(buf, size);
    if (ret != LOS_OK) {
        ret = LOS_ERRNO_PERF_BUF_ERROR;
        goto PERF_INIT_ERROR;
    }
    g_perfCb.status = PERF_STOPED;
PERF_INIT_ERROR:
    PERF_UNLOCK(intSave);
    return ret;
}

UINT32 LOS_PerfConfig(PerfConfigAttr *attr)
{
    UINT32 ret;
    UINT32 intSave;

    if (attr == NULL) {
        return LOS_ERRNO_PERF_CONFIG_NULL;
    }

    PERF_LOCK(intSave);
    if (g_perfCb.status != PERF_STOPED) {
        ret = LOS_ERRNO_PERF_STATUS_INVALID;
        PRINT_ERR("perf config status error : 0x%x\n", g_perfCb.status);
        goto PERF_CONFIG_ERROR;
    }

    g_pmu = NULL;

    g_perfCb.needSample = attr->needSample;
    g_perfCb.taskFilterEnable = attr->taskFilterEnable;
    g_perfCb.sampleType = attr->sampleType;

    if (attr->taskFilterEnable) {
        ret = memcpy_s(g_perfCb.taskIds, PERF_MAX_FILTER_TSKS * sizeof(UINT32), attr->taskIds,
                       g_perfCb.taskIdsNr * sizeof(UINT32));
        if (ret != EOK) {
            PRINT_ERR("In %s At line:%d execute memcpy_s error\n", __FUNCTION__, __LINE__);
            goto PERF_CONFIG_ERROR;
        }
        g_perfCb.taskIdsNr = MIN(attr->taskIdsNr, PERF_MAX_FILTER_TSKS);
    }
    ret = OsPerfConfig(&attr->eventsCfg);
PERF_CONFIG_ERROR:
    PERF_UNLOCK(intSave);
    return ret;
}

VOID LOS_PerfStart(UINT32 sectionId)
{
    UINT32 intSave;
    UINT32 ret;

    PERF_LOCK(intSave);
    if (g_perfCb.status != PERF_STOPED) {
        PRINT_ERR("perf start status error : 0x%x\n", g_perfCb.status);
        goto PERF_START_ERROR;
    }

    if (!OsPerfParamValid()) {
        PRINT_ERR("forgot call `LOS_Config(...)` before instrumenting?\n");
        goto PERF_START_ERROR;
    }

    if (g_perfCb.needSample) {
        ret = OsPerfHdrInit(sectionId); /* section header init */
        if (ret != LOS_OK) {
            PRINT_ERR("perf hdr init error 0x%x\n", ret);
            goto PERF_START_ERROR;
        }
    }

    SMP_CALL_PERF_FUNC(OsPerfStart); /* send to all cpu to start pmu */
    g_perfCb.status = PERF_STARTED;
    g_perfCb.startTime = OsPerfGetCurrTime();
PERF_START_ERROR:
    PERF_UNLOCK(intSave);
    return;
}

VOID LOS_PerfStop(VOID)
{
    UINT32 intSave;

    PERF_LOCK(intSave);
    if (g_perfCb.status != PERF_STARTED) {
        PRINT_ERR("perf stop status error : 0x%x\n", g_perfCb.status);
        goto PERF_STOP_ERROR;
    }

    SMP_CALL_PERF_FUNC(OsPerfStop); /* send to all cpu to stop pmu */

    OsPerfOutPutFlush();

    if (g_perfCb.needSample) {
        OsPerfOutPutInfo();
    }

    g_perfCb.status = PERF_STOPED;
    g_perfCb.endTime = OsPerfGetCurrTime();

    OsPerfPrintTs();
PERF_STOP_ERROR:
    PERF_UNLOCK(intSave);
    return;
}

UINT32 LOS_PerfDataRead(CHAR *dest, UINT32 size)
{
    return OsPerfOutPutRead(dest, size);
}

VOID LOS_PerfNotifyHookReg(const PERF_BUF_NOTIFY_HOOK func)
{
    UINT32 intSave;

    PERF_LOCK(intSave);
    OsPerfNotifyHookReg(func);
    PERF_UNLOCK(intSave);
}

VOID LOS_PerfFlushHookReg(const PERF_BUF_FLUSH_HOOK func)
{
    UINT32 intSave;

    PERF_LOCK(intSave);
    OsPerfFlushHookReg(func);
    PERF_UNLOCK(intSave);
}

VOID OsPerfSetIrqRegs(UINTPTR pc, UINTPTR fp)
{
    LosTaskCB *runTask = (LosTaskCB *)ArchCurrTaskGet();
    runTask->pc = pc;
    runTask->fp = fp;
}

#endif /* LOSCFG_KERNEL_PERF == YES */

#ifdef __cplusplus
#if __cplusplus
}
#endif /* __cplusplus */
#endif /* __cplusplus */
