/* ----------------------------------------------------------------------------
 * Copyright (c) Huawei Technologies Co., Ltd. 2020-2020. All rights reserved.
 * Description: LiteOS Perf Pmu Manager Module Implementation
 * Author: Huawei LiteOS Team
 * Create: 2020-07-29
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

#include "perf_pmu_pri.h"

#ifdef __cplusplus
#if __cplusplus
extern "C" {
#endif /* __cplusplus */
#endif /* __cplusplus */

STATIC Pmu *g_pmuMgr[PERF_EVENT_TYPE_MAX] = {NULL};

UINT32 OsPerfPmuRegister(Pmu *pmu)
{
    UINT32 type;

    if ((pmu == NULL) || (pmu->type >= PERF_EVENT_TYPE_MAX)) {
        return LOS_NOK;
    }

    type = pmu->type;
    if (g_pmuMgr[type] == NULL) {
        g_pmuMgr[type] = pmu;
        return LOS_OK;
    }
    return LOS_NOK;
}

Pmu *OsPerfPmuGet(UINT32 type)
{
    if (type >= PERF_EVENT_TYPE_MAX) {
        return NULL;
    }

    if (type == PERF_EVENT_TYPE_RAW) { /* process hardware raw events with hard pmu */
        type = PERF_EVENT_TYPE_HW;
    }
    return g_pmuMgr[type];
}

VOID OsPerfPmuRm(UINT32 type)
{
    if (type >= PERF_EVENT_TYPE_MAX) {
        return;
    }
    g_pmuMgr[type] = NULL;
}

#ifdef __cplusplus
#if __cplusplus
}
#endif /* __cplusplus */
#endif /* __cplusplus */
