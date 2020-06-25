package org.coepi.core.domain

import org.coepi.core.domain.model.Tcn
import org.coepi.api.Api
import org.coepi.api.asResult
import org.coepi.core.domain.common.Result

interface ObservedTcnsRecorder {
    fun recordTcn(tcn: Tcn): Result<Unit, Throwable>
}

class ObservedTcnsRecorderImpl(private val nativeApi: Api) : ObservedTcnsRecorder {
    override fun recordTcn(tcn: Tcn): Result<Unit, Throwable> =
        nativeApi.recordTcn(tcn.toHex()).asResult()
}
