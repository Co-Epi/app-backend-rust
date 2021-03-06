package org.coepi.core.services

import org.coepi.core.domain.common.Result
import org.coepi.core.domain.model.Tcn
import org.coepi.core.jni.JniApi
import org.coepi.core.jni.asResult

interface ObservedTcnsRecorder {
    // Meters
    fun recordTcn(tcn: Tcn, distance: Float): Result<Unit, Throwable>
}

class ObservedTcnsRecorderImpl(private val api: JniApi) :
    ObservedTcnsRecorder {
    override fun recordTcn(tcn: Tcn, distance: Float): Result<Unit, Throwable> =
        api.recordTcn(tcn.toHex(), distance).asResult()
}
