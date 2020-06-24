package org.coepi.api

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.ExperimentalCoroutinesApi
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@ExperimentalCoroutinesApi
@RunWith(AndroidJUnit4::class)
class JNIInterfaceTests {

    private lateinit var instrumentationContext: Context

    @Before
    fun setup() {
        instrumentationContext = androidx.test.core.app.ApplicationProvider.getApplicationContext()
    }

    @Test
    fun testBootstrap() {
        val dbPath = instrumentationContext.getDatabasePath("remove")
            // we need to pass the db directory (without file name)
            .absolutePath.removeSuffix("/remove")

        val n = Api()
        val result = n.bootstrapCore(dbPath, "debug", true, JniLogCallback())

        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun testFetchAReport() {
        val n = Api()
        val value = n.testReturnAnAlert()
        assertEquals(
            JniOneAlertResult(
                1, "", JniAlert(
                    "123", JniPublicReport(
                        reportTime = 234324,
                        earliestSymptomTime = 1590356601,
                        feverSeverity = 1,
                        coughSeverity = 3,
                        breathlessness = true,
                        muscleAches = true,
                        lossSmellOrTaste = false,
                        diarrhea = false,
                        runnyNose = true,
                        other = false,
                        noSymptoms = true
                    ), 1592567315
                )
            ),
            value
        )
    }

    @Test
    fun testFetchNewReports() {
        val n = Api()
        val value = n.testReturnMultipleAlerts()
        assertEquals(
            JniAlertsArrayResult(
                1, "", arrayOf(
                    JniAlert(
                        "123", JniPublicReport(
                            reportTime = 131321,
                            earliestSymptomTime = 1590356601,
                            feverSeverity = 1,
                            coughSeverity = 3,
                            breathlessness = true,
                            muscleAches = true,
                            lossSmellOrTaste = false,
                            diarrhea = false,
                            runnyNose = true,
                            other = false,
                            noSymptoms = true
                        ), 1592567315
                    ),
                    JniAlert(
                        "343356", JniPublicReport(
                            reportTime = 32516899200,
                            earliestSymptomTime = 1590356601,
                            feverSeverity = 1,
                            coughSeverity = 3,
                            breathlessness = true,
                            muscleAches = true,
                            lossSmellOrTaste = false,
                            diarrhea = false,
                            runnyNose = true,
                            other = false,
                            noSymptoms = true
                        ), 1592567315
                    )
                )
            ),
            value
        )
    }
}
