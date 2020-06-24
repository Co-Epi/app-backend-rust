package org.coepi.api

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.ExperimentalCoroutinesApi
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

    private lateinit var instrumentationContext: Context

    @Before
    fun setup() {
        instrumentationContext = androidx.test.core.app.ApplicationProvider.getApplicationContext()

        val dbPath = instrumentationContext.getDatabasePath("remove")
            // we need to pass the db directory (without file name)
            .absolutePath.removeSuffix("/remove")

        val n = Api()
        val result = n.bootstrapCore(dbPath, "debug", true, JniLogCallback())
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
        val value = Api().recordTcn("2485a64b57addcaea3ed1b538d07dbce")
        assertEquals(JniVoidResult(1, ""), value)
    }

    @Test
    fun generateTcn() {
        val value = Api().generateTcn()
        assertEquals(value.length, 32)
    }

    @Test
    fun setSymptomIds() {
        // NOTE: JSON format
        val value =
            Api().setSymptomIds("""["breathlessness", "muscle_aches", "runny_nose"]""")
        assertEquals(JniVoidResult(1, ""), value)
    }

    @Test
    fun setInvalidSymptomIdReturnsError() {
        // NOTE: JSON format
        val value = Api().setSymptomIds("""["not_supported", "muscle_aches", "runny_nose"]""")
        // TODO https://github.com/Co-Epi/app-backend-rust/issues/79 shouldn't return 1
        assertEquals(JniVoidResult(1, ""), value)
    }

    @Test
    fun setInvalidSymptomIdsJsonReturnsError() {
        val value = Api().setSymptomIds("sdjfhskdf")
        // TODO https://github.com/Co-Epi/app-backend-rust/issues/79 shouldn't return 1
        assertEquals(JniVoidResult(1, ""), value)
    }

    @Test
    fun setCoughTypeNone() {
        val result = Api().setCoughType("none")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughTypeWet() {
        val result = Api().setCoughType("wet")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughTypeDry() {
        val result = Api().setCoughType("dry")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidCoughTypeReturnsError() {
        val result = Api().setCoughType("invalid")
        // TODO https://github.com/Co-Epi/app-backend-rust/issues/79 shouldn't return 1
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughDaysIsSet() {
        val result = Api().setCoughDays(1, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughDaysIsNotSet() {
        // Note: days is ignored
        val result = Api().setCoughDays(0, 123)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setCoughStatus() {
        val result = Api().setCoughStatus("better_and_worse")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidCoughStatusReturnsError() {
        val result = Api().setCoughStatus("invalid")
        // TODO https://github.com/Co-Epi/app-backend-rust/issues/79 shouldn't return 1
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setBreathlessnessCause() {
        val result = Api().setCoughStatus("leaving_house_or_dressing")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidBreathlessnessCauseReturnsError() {
        val result = Api().setCoughStatus("invalid")
        // TODO https://github.com/Co-Epi/app-backend-rust/issues/79 shouldn't return 1
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverDaysIsSet() {
        val result = Api().setFeverDays(1, 3)
        // TODO https://github.com/Co-Epi/app-backend-rust/issues/79 shouldn't return 1
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverDaysNone() {
        // Note: days is ignored
        val result = Api().setFeverDays(0, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverTakenTemperatureToday() {
        val result = Api().setFeverTakenTemperatureToday(1, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverTakenTemperatureTodayNone() {
        // Note: days is ignored
        val result = Api().setFeverTakenTemperatureToday(0, 3)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setFeverTakenTemperatureSpot() {
        val result = Api().setFeverTakenTemperatureSpot("armpit")
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setInvalidFeverTakenTemperatureSpot() {
        val result = Api().setFeverTakenTemperatureSpot("invalid")
        // TODO https://github.com/Co-Epi/app-backend-rust/issues/79 shouldn't return 1
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setHigherFeverTemperatureTaken() {
        val result = Api().setFeverHighestTemperatureTaken(1, 100f)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setHigherFeverTemperatureTakenNone() {
        // Note: temp is ignored
        val result = Api().setFeverHighestTemperatureTaken(0, 100f)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setEarliestSymptomStartedDaysAgo() {
        val result = Api().setEarliestSymptomStartedDaysAgo(1, 10)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun setEarliestSymptomStartedDaysAgoNone() {
        // Note: days is ignored
        val result = Api().setEarliestSymptomStartedDaysAgo(0, 10)
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun clearSymptoms() {
        val result = Api().clearSymptoms()
        assertEquals(JniVoidResult(1, ""), result)
    }

    @Test
    fun submitSymptoms() {
        val result = Api().submitSymptoms()
        assertEquals(JniVoidResult(1, ""), result)
    }

    // TODO more detailed tests, e.g. for each supported enum string (probably it makes sense to add
    // TODO constants in the app)
}
