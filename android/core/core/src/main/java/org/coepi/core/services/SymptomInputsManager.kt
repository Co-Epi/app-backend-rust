package org.coepi.core.services

import com.google.gson.Gson
import org.coepi.core.jni.JniApi
import org.coepi.core.jni.asResult
import org.coepi.core.domain.common.Result
import org.coepi.core.domain.model.SymptomId
import org.coepi.core.domain.model.SymptomId.BREATHLESSNESS
import org.coepi.core.domain.model.SymptomId.COUGH
import org.coepi.core.domain.model.SymptomId.DIARRHEA
import org.coepi.core.domain.model.SymptomId.FEVER
import org.coepi.core.domain.model.SymptomId.LOSS_SMELL_OR_TASTE
import org.coepi.core.domain.model.SymptomId.MUSCLE_ACHES
import org.coepi.core.domain.model.SymptomId.NONE
import org.coepi.core.domain.model.SymptomId.OTHER
import org.coepi.core.domain.model.SymptomId.RUNNY_NOSE
import org.coepi.core.domain.model.SymptomInputs.Breathlessness
import org.coepi.core.domain.model.SymptomInputs.Breathlessness.Cause.EXERCISE
import org.coepi.core.domain.model.SymptomInputs.Breathlessness.Cause.GROUND_OWN_PACE
import org.coepi.core.domain.model.SymptomInputs.Breathlessness.Cause.HURRY_OR_HILL
import org.coepi.core.domain.model.SymptomInputs.Breathlessness.Cause.LEAVING_HOUSE_OR_DRESSING
import org.coepi.core.domain.model.SymptomInputs.Breathlessness.Cause.WALKING_YARDS_OR_MINS_ON_GROUND
import org.coepi.core.domain.model.SymptomInputs.Cough
import org.coepi.core.domain.model.SymptomInputs.Cough.Status.BETTER_AND_WORSE_THROUGH_DAY
import org.coepi.core.domain.model.SymptomInputs.Cough.Status.SAME_OR_STEADILY_WORSE
import org.coepi.core.domain.model.SymptomInputs.Cough.Status.WORSE_WHEN_OUTSIDE
import org.coepi.core.domain.model.SymptomInputs.Cough.Type.DRY
import org.coepi.core.domain.model.SymptomInputs.Cough.Type.WET
import org.coepi.core.domain.model.SymptomInputs.Fever
import org.coepi.core.domain.model.SymptomInputs.Fever.TemperatureSpot.Armpit
import org.coepi.core.domain.model.SymptomInputs.Fever.TemperatureSpot.Ear
import org.coepi.core.domain.model.SymptomInputs.Fever.TemperatureSpot.Mouth
import org.coepi.core.domain.model.SymptomInputs.Fever.TemperatureSpot.Other
import org.coepi.core.domain.model.Temperature
import org.coepi.core.domain.model.UserInput
import java.io.Serializable

interface SymptomsInputManager {
    fun setSymptoms(inputs: Set<SymptomId>): Result<Unit, Throwable>
    fun setCoughType(input: UserInput<Cough.Type>): Result<Unit, Throwable>
    fun setCoughDays(input: UserInput<Cough.Days>): Result<Unit, Throwable>
    fun setCoughStatus(input: UserInput<Cough.Status>): Result<Unit, Throwable>
    fun setBreathlessnessCause(input: UserInput<Breathlessness.Cause>): Result<Unit, Throwable>
    fun setFeverDays(input: UserInput<Fever.Days>): Result<Unit, Throwable>
    fun setFeverTakenTemperatureToday(input: UserInput<Boolean>): Result<Unit, Throwable>
    fun setFeverTakenTemperatureSpot(input: UserInput<Fever.TemperatureSpot>): Result<Unit, Throwable>
    fun setFeverHighestTemperatureTaken(input: UserInput<Temperature>): Result<Unit, Throwable>
    fun setEarliestSymptomStartedDaysAgo(input: UserInput<Int>): Result<Unit, Throwable>

    fun submitSymptoms(): Result<Unit, Throwable>
    fun clearSymptoms(): Result<Unit, Throwable>
}

class SymptomInputsManagerImpl(private val api: JniApi, private val gson: Gson) :
    SymptomsInputManager {

    override fun setSymptoms(inputs: Set<SymptomId>): Result<Unit, Throwable> {
        val jniIdentifiers = inputs.map { it.toJniIdentifier() }
        return api.setSymptomIds(gson.toJson(jniIdentifiers)).asResult()
    }

    override fun setCoughType(input: UserInput<Cough.Type>): Result<Unit, Throwable> =
        api.setCoughType(
            when (input) {
                is UserInput.None -> "none"
                is UserInput.Some -> when (input.value) {
                    WET -> "wet"
                    DRY -> "dry"
                }
            }
        ).asResult()

    override fun setCoughDays(input: UserInput<Cough.Days>): Result<Unit, Throwable> =
        when (input) {
            is UserInput.None -> api.setCoughDays(0, -1)
            is UserInput.Some -> api.setCoughDays(1, input.value.value)
        }.asResult()

    override fun setCoughStatus(input: UserInput<Cough.Status>): Result<Unit, Throwable> =
        api.setCoughType(
            input.toJniStringInput {
                when (it) {
                    BETTER_AND_WORSE_THROUGH_DAY -> "better_and_worse"
                    SAME_OR_STEADILY_WORSE -> "same_steadily_worse"
                    WORSE_WHEN_OUTSIDE -> "worse_outside"
                }
            }
        ).asResult()

    // TODO tests: e.g. this was previously setting cough type, since JNI api isn't typed, compiler can't detect it,
    // TODO probably we need to expose a function in Rust that returns the current inputs.
    override fun setBreathlessnessCause(input: UserInput<Breathlessness.Cause>): Result<Unit, Throwable> =
        api.setBreathlessnessCause(
            input.toJniStringInput {
                when (it) {
                    EXERCISE -> "exercise"
                    LEAVING_HOUSE_OR_DRESSING -> "leaving_house_or_dressing"
                    WALKING_YARDS_OR_MINS_ON_GROUND -> "walking_yards_or_mins_on_ground"
                    GROUND_OWN_PACE -> "ground_own_pace"
                    HURRY_OR_HILL -> "hurry_or_hill"
                }
            }
        ).asResult()

    override fun setFeverDays(input: UserInput<Fever.Days>): Result<Unit, Throwable> =
        input.toJniIntInput { it.value }.let {
            api.setFeverDays(it.isSet, it.value).asResult()
        }

    override fun setFeverTakenTemperatureToday(input: UserInput<Boolean>): Result<Unit, Throwable> =
        input.toJniIntInput { if (it) 1 else 0 }.let {
            api.setFeverTakenTemperatureToday(it.isSet, it.value).asResult()
        }

    override fun setFeverTakenTemperatureSpot(input: UserInput<Fever.TemperatureSpot>): Result<Unit, Throwable> =
        api.setCoughType(
            input.toJniStringInput {
                when (it) {
                    is Armpit -> "armpit"
                    is Mouth -> "mouth"
                    is Ear -> "ear"
                    is Other -> "other"
                }
            }
        ).asResult()

    override fun setFeverHighestTemperatureTaken(input: UserInput<Temperature>): Result<Unit, Throwable> =
        input.toJniFloatInput { it.toFarenheit().value }.let {
            api.setFeverHighestTemperatureTaken(it.isSet, it.value).asResult()
        }

    override fun setEarliestSymptomStartedDaysAgo(input: UserInput<Int>): Result<Unit, Throwable> =
        input.toJniIntInput { it }.let {
            api.setEarliestSymptomStartedDaysAgo(it.isSet, it.value).asResult()
        }

    override fun submitSymptoms(): Result<Unit, Throwable> = api.submitSymptoms().asResult()

    override fun clearSymptoms(): Result<Unit, Throwable> = api.clearSymptoms().asResult()

    //endregion

    private fun <T : Serializable> UserInput<T>.toJniStringInput(f: (T) -> String): String =
        when (this) {
            is UserInput.None -> "none"
            is UserInput.Some -> f(value)
        }

    private fun <T : Serializable> UserInput<T>.toJniIntInput(f: (T) -> Int): JniIntInput =
        when (this) {
            is UserInput.Some -> JniIntInput(
                1,
                f(value)
            )
            is UserInput.None -> JniIntInput(
                0,
                -1
            )
        }

    private fun <T : Serializable> UserInput<T>.toJniFloatInput(f: (T) -> Float): JniFloatInput =
        when (this) {
            is UserInput.Some -> JniFloatInput(
                1,
                f(value)
            )
            is UserInput.None -> JniFloatInput(
                0,
                -1f
            )
        }

    private data class JniIntInput(val isSet: Int, val value: Int)
    private data class JniFloatInput(val isSet: Int, val value: Float)

    private fun Boolean.asInt(): Int = if (this) 1 else 0

    private fun SymptomId.toJniIdentifier(): String = when (this) {
        COUGH -> "cough"
        BREATHLESSNESS -> "breathlessness"
        FEVER -> "fever"
        MUSCLE_ACHES -> "muscle_aches"
        LOSS_SMELL_OR_TASTE -> "loss_smell_or_taste"
        DIARRHEA -> "diarrhea"
        RUNNY_NOSE -> "runny_nose"
        OTHER -> "other"
        NONE -> "none"
    }
}
