package org.coepi.core.domain.model

import android.os.Parcelable
import kotlinx.android.parcel.Parcelize
import org.coepi.core.domain.model.UserInput.None
import java.io.Serializable

enum class SymptomId {
    COUGH, BREATHLESSNESS, FEVER, MUSCLE_ACHES, LOSS_SMELL_OR_TASTE, DIARRHEA, RUNNY_NOSE, OTHER,
    NONE
}

@Parcelize
data class SymptomInputs(
    val ids: Set<SymptomId> = emptySet(),
    val cough: Cough = Cough(),
    val breathlessness: Breathlessness = Breathlessness(),
    val fever: Fever = Fever(),
    val earliestSymptom: EarliestSymptom = EarliestSymptom()
) : Parcelable {

    @Parcelize
    data class Cough(
        val type: UserInput<Type> = None,
        val days: UserInput<Days> = None,
        val status: UserInput<Status> = None

    ) : Parcelable {
        @Parcelize
        enum class Type : Parcelable { WET, DRY }

        @Parcelize
        enum class Status : Parcelable {
            BETTER_AND_WORSE_THROUGH_DAY, WORSE_WHEN_OUTSIDE, SAME_OR_STEADILY_WORSE
        }

        data class Days(val value: Int) : Serializable
    }

    @Parcelize
    data class Breathlessness(
        val cause: UserInput<Cause> = None
    ) : Parcelable {

        @Parcelize
        enum class Cause : Parcelable {
            LEAVING_HOUSE_OR_DRESSING, WALKING_YARDS_OR_MINS_ON_GROUND, GROUND_OWN_PACE,
            HURRY_OR_HILL, EXERCISE
        }
    }

    @Parcelize
    data class Fever(
        val days: UserInput<Days> = None,
        val takenTemperatureToday: UserInput<Boolean> = None,
        val temperatureSpot: UserInput<TemperatureSpot> = None,
        val highestTemperature: UserInput<Temperature> = None
    ) : Parcelable, Serializable {
        data class Days(val value: Int) : Serializable

        sealed class TemperatureSpot : Serializable {
            object Mouth : TemperatureSpot()
            object Ear : TemperatureSpot()
            object Armpit : TemperatureSpot()
            data class Other(val description: String) : TemperatureSpot()
        }
    }

    @Parcelize
    data class EarliestSymptom(
        val time: UserInput<UnixTime> = None
    ) : Parcelable
}

// Ideally the type parameter would have been Parcelable, but we want to allow primitives too.
sealed class UserInput<out T : Serializable> : Parcelable {
    @Parcelize
    object None : UserInput<Nothing>(), Parcelable

    @Parcelize
    data class Some<T : Serializable>(val value: T) : UserInput<T>(), Parcelable

    fun <U : Serializable> map(f: (T) -> U): UserInput<U> = when (this) {
        is None -> this
        is Some -> Some(f(value))
    }

    fun <U : Serializable> flatMap(f: (T) -> UserInput<U>): UserInput<U> = when (this) {
        is None -> this
        is Some -> f(value)
    }
}
