package org.coepi.core.services

import org.coepi.core.jni.JniAlert
import org.coepi.core.jni.JniAlertsArrayResult
import org.coepi.core.jni.JniApi
import org.coepi.core.domain.model.Alert
import org.coepi.core.domain.model.toCoughSeverity
import org.coepi.core.domain.model.toFeverSeverity
import org.coepi.core.domain.common.Result
import org.coepi.core.domain.common.Result.Success
import org.coepi.core.domain.common.Result.Failure
import org.coepi.core.domain.model.LengthMeasurement.Meters
import org.coepi.core.domain.model.UnixTime
import org.coepi.core.domain.model.UserInput.None
import org.coepi.core.domain.model.UserInput.Some
import org.coepi.core.jni.asResult

interface AlertsApi {
    fun fetchNewAlerts(): Result<List<Alert>, Throwable>
    fun deleteAlert(id: String): Result<Unit, Throwable>
}

class AlertsFetcherImpl(private val api: JniApi) :
    AlertsApi {

    override fun fetchNewAlerts(): Result<List<Alert>, Throwable> {
        val result = api.fetchNewReports()
        return when (result.status) {
            1 -> Success(result.obj.map { it.toAlert() })
            else -> Failure(Throwable(result.statusDescription()))
        }
    }

    override fun deleteAlert(id: String): Result<Unit, Throwable> =
        api.deleteAlert(id).asResult()

    private fun JniAlertsArrayResult.statusDescription(): String =
        statusDescription(status, message)

    private fun statusDescription(status: Int, message: String) =
        "Status: $status Message: $message"

    private fun JniAlert.toAlert() = Alert(
        id = id,
        contactStart = when {
            contactStart < 0 -> error("Invalid contact start: $contactStart")
            else -> UnixTime.fromValue(contactStart)
        },
        contactEnd = when {
            contactEnd < 0 -> error("Invalid contact end: $contactEnd")
            else -> UnixTime.fromValue(contactEnd)
        },
        minDistance = when {
            minDistance < 0 -> error("Invalid min distance: $minDistance")
            else -> Meters(minDistance)
        },
        avgDistance = when {
            avgDistance < 0 -> error("Invalid avg distance: $avgDistance")
            else -> Meters(avgDistance)
        },
        reportTime = when {
            symptoms.reportTime < 0 -> error("Invalid report time: ${symptoms.reportTime}")
            else -> UnixTime.fromValue(symptoms.reportTime)
        },
        earliestSymptomTime = when {
            symptoms.earliestSymptomTime == -1L ->
                None
            symptoms.earliestSymptomTime < -1L ->
                error("Invalid earliestSymptomTime: ${symptoms.earliestSymptomTime}")
            else ->
                Some(UnixTime.fromValue(symptoms.earliestSymptomTime))
        },
        feverSeverity = toFeverSeverity(symptoms.feverSeverity),
        coughSeverity = toCoughSeverity(symptoms.coughSeverity),
        breathlessness = symptoms.breathlessness,
        muscleAches = symptoms.muscleAches,
        lossSmellOrTaste = symptoms.lossSmellOrTaste,
        diarrhea = symptoms.diarrhea,
        runnyNose = symptoms.runnyNose,
        other = symptoms.other,
        noSymptoms = symptoms.noSymptoms,
        isRead = isRead
    )
}
