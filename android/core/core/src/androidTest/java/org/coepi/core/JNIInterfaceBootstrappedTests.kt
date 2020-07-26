package org.coepi.core

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.ExperimentalCoroutinesApi
import org.coepi.core.jni.JniApi
import org.coepi.core.jni.JniLogCallback
import org.coepi.core.jni.JniVoidResult
import org.coepi.core.services.CoreLogger
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Tests JNI functions that require that bootstrap was called in Rust.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@ExperimentalCoroutinesApi
@RunWith(AndroidJUnit4::class)
class JNIInterfaceBootstrappedTests {

    companion object {
        // Fixes Os { code: 11, kind: WouldBlock, message: "Try again" } error in bootstrap
        // Apparently something there (likely the Persy initalizer, which creates a DB before checking
        // whether the static OnceCell was set already) doesn't like being called in quick succession/parallel
        // TODO investigate
        // Not critical since the apps call bootstrap only once, at launch.
        private var bootstrapped = false
    }

    private lateinit var instrumentationContext: Context

    @Before
    fun setup() {
        if (bootstrapped) return
        bootstrapped = true
        bootstrap()
    }

    private fun bootstrap() {
        instrumentationContext = androidx.test.core.app.ApplicationProvider.getApplicationContext()

        val dbPath = instrumentationContext.getDatabasePath("remove")
            // we need to pass the db directory (without file name)
            .absolutePath.removeSuffix("/remove")

        val n = JniApi()
        val result = n.bootstrapCore(dbPath, "debug", true,
            JniLogCallback(object : CoreLogger {
                override fun log(level: Int, message: String) {
                    println("[CORE] level: $level, message: $message")
                }
            })
        )
        // Double check
        assertEquals(JniVoidResult(1, ""), result)
    }

    // Manual testing
    // Ideally we would support for a flag in bootstrap to use testing mode / backend mock
//    @Test
//    fun fetchNewReports() {
//        val value = Api().fetchNewReports()
//        assertEquals(JniAlertsArrayResult(1, "", emptyArray()), value)
//    }

    @Test
    fun recordTcn() {
        val value = JniApi().recordTcn("2485a64b57addcaea3ed1b538d07dbce", 34.03f)
        assertEquals(JniVoidResult(1, ""), value)
    }

    @Test
    fun generateTcn() {
        val value = JniApi().generateTcn()
        assertEquals(value.length, 32)
    }

    @Test
    fun setSymptomIds() {
        // NOTE: JSON format
        val value =
            JniApi().setSymptomIds("""["breathlessness", "muscle_aches", "runny_nose"]""")
        assertEquals(JniVoidResult(1, ""), value)
    }

    @Test
    fun setInvalidSymptomIdReturnsError() {
        // NOTE: JSON format
        val value = JniApi().setSymptomIds("""["not_supported", "muscle_aches", "runny_nose"]""")
        assertEquals(3, value.status)
    }

    @Test
    fun setInvalidSymptomIdsJsonReturnsError() {
        val value = JniApi().setSymptomIds("sdjfhskdf")
        assertEquals(3, value.status)
    }

    @Test
    fun setCoughTypeNone() {
        val result = JniApi().setCoughType("none")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughTypeWet() {
        val result = JniApi().setCoughType("wet")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughTypeDry() {
        val result = JniApi().setCoughType("dry")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidCoughTypeReturnsError() {
        val result = JniApi().setCoughType("invalid")
        assertEquals(3, result.status)
    }

    @Test
    fun setCoughDaysIsSet() {
        val result = JniApi().setCoughDays(1, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughDaysIsNotSet() {
        // Note: days is ignored
        val result = JniApi().setCoughDays(0, 123)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughStatus() {
        val result = JniApi().setCoughStatus("better_and_worse")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidCoughStatusReturnsError() {
        val result = JniApi().setCoughStatus("invalid")
        assertEquals(3, result.status)
    }

    @Test
    fun setBreathlessnessCause() {
        val result = JniApi().setBreathlessnessCause("leaving_house_or_dressing")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidBreathlessnessCauseReturnsError() {
        val result = JniApi().setCoughStatus("invalid")
        assertEquals(3, result.status)
    }

    @Test
    fun setFeverDaysIsSet() {
        val result = JniApi().setFeverDays(1, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverDaysNone() {
        // Note: days is ignored
        val result = JniApi().setFeverDays(0, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverTakenTemperatureToday() {
        val result = JniApi()
            .setFeverTakenTemperatureToday(1, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverTakenTemperatureTodayNone() {
        // Note: days is ignored
        val result = JniApi()
            .setFeverTakenTemperatureToday(0, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverTakenTemperatureSpot() {
        val result = JniApi().setFeverTakenTemperatureSpot("armpit")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidFeverTakenTemperatureSpot() {
        val result = JniApi().setFeverTakenTemperatureSpot("invalid")
        assertEquals(3, result.status)
    }

    @Test
    fun setHigherFeverTemperatureTaken() {
        val result = JniApi()
            .setFeverHighestTemperatureTaken(1, 100f)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setHigherFeverTemperatureTakenNone() {
        // Note: temp is ignored
        val result = JniApi()
            .setFeverHighestTemperatureTaken(0, 100f)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setEarliestSymptomStartedDaysAgo() {
        val result = JniApi()
            .setEarliestSymptomStartedDaysAgo(1, 10)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setEarliestSymptomStartedDaysAgoNone() {
        // Note: days is ignored
        val result = JniApi()
            .setEarliestSymptomStartedDaysAgo(0, 10)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun clearSymptoms() {
        val result = JniApi().clearSymptoms()
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun submitSymptoms() {
        val result = JniApi().submitSymptoms()
        assertEquals(JniVoidResult(1, ""), result)
    }

    // TODO more detailed tests, e.g. for each supported enum string (probably it makes sense to add
    // TODO constants in the app)
}
