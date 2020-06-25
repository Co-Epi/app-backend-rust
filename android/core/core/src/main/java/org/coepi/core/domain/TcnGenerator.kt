package org.coepi.core.domain

import org.coepi.core.domain.model.Tcn
import org.coepi.api.Api
import org.coepi.core.extensions.hexToByteArray

interface TcnGenerator {
    fun generateTcn(): Tcn
}

class TcnGeneratorImpl(private val nativeApi: Api) : TcnGenerator {
    override fun generateTcn(): Tcn =
        Tcn(nativeApi.generateTcn().hexToByteArray())
}
