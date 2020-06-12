package org.coepi.android.api

class NativeApi {

    init
    {
        System.loadLibrary("coepi_core")
    }

    external fun testSendReceiveString(pattern: String): String

    external fun bootstrapCore(db_path: String): String

    // external fun callCallback(void (*callback)(Int, Boolean, String)): Int

    external fun clearSymptoms(): String

    external fun fetchNewReports(): String

    external fun generateTcn(): String

    // external fun passAndReturnStruct(const FFIParameterStruct *par): FFIReturnStruct

    // external fun passStruct(const FFIParameterStruct *par): Int

    external fun postReport(c_report: String): String

    external fun recordTcn(c_tcn: String): String

    // external fun registerCallback(void (*callback)(Int, Boolean, String)): Int

    // external fun registerLogCallback(void (*log_callback)(CoreLogMessage)): Int

    // external fun returnStruct(): FFIReturnStruct

    external fun setBreathlessnessCause(c_cause: String): String

    external fun setCoughDays(c_is_set: UByte, c_days: UInt): String

    external fun setCoughStatus(c_status: String): String

    external fun setCoughType(c_cough_type: String): String

    external fun setEarliestSymptomStartedDaysAgo(c_is_set:UByte, c_days: UInt): String

    external fun setFeverDays(c_is_set:UByte, c_days: UInt): String

    external fun setFeverHighestTemperatureTaken(c_is_set: UByte, c_temp: Float): String

    external fun setFeverTakenTemperatureSpot(c_cause: String): String

    external fun setFeverTakenTemperatureToday(c_is_set: UByte, c_taken: UByte): String

    external fun setSymptomIds(c_ids: String): String

    external fun submitSymptoms(): String

    external fun triggerCallback(my_str: String): Int

    external fun triggerLoggingMacros(): Int

}