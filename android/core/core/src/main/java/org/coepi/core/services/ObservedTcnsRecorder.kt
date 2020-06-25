package org.coepi.core.services

import org.coepi.core.jni.JniApi
import org.coepi.core.jni.asResult
import org.coepi.core.domain.model.Tcn
import org.coepi.core.domain.common.Result

interface ObservedTcnsRecorder {
    fun recordTcn(tcn: Tcn): Result<Unit, Throwable>
}

class ObservedTcnsRecorderImpl(private val api: JniApi) :
    ObservedTcnsRecorder {
    override fun recordTcn(tcn: Tcn): Result<Unit, Throwable> =
        api.recordTcn(tcn.toHex()).asResult()
}
