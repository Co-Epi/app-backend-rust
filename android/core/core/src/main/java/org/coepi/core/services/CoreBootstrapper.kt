package org.coepi.core.services

import android.content.Context
import org.coepi.core.jni.JniApi
import org.coepi.core.jni.JniLogCallback

interface CoreBootstrapper {
    fun bootstrap(applicationContext: Context, logger: CoreLogger)
}

interface CoreLogger {
    fun log(level: Int, message: String)
}

class CoreBootstrapperImpl(private val api: JniApi) : CoreBootstrapper {

    override fun bootstrap(applicationContext: Context, logger: CoreLogger) {
        // getDatabasePath requires a db name, but we use need the directory
        // (to initialize multiple databases), so adding and removing a suffix.
        val dbPath = applicationContext.getDatabasePath("remove")
            .absolutePath.removeSuffix("/remove")

        val result = api.bootstrapCore(
            dbPath, "debug", true,
            JniLogCallback(logger)
        )
        if (result.status != 1) {
            error("Couldn't bootstrap core: status: ${result.status}, message: ${result.message}")
        }
    }
}
