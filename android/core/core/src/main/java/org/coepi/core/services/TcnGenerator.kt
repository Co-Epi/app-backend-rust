package org.coepi.core.services

import org.coepi.core.jni.JniApi
import org.coepi.core.domain.model.Tcn
import org.coepi.core.extensions.hexToByteArray

interface TcnGenerator {
    fun generateTcn(): Tcn
}

class TcnGeneratorImpl(private val api: JniApi) : TcnGenerator {
    override fun generateTcn(): Tcn =
        Tcn(api.generateTcn().hexToByteArray())
}
