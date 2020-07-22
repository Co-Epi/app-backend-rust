package org.coepi.core.domain.model

import android.os.Parcelable
import kotlinx.android.parcel.Parcelize

@Parcelize
data class Alert(
    var id: String,
    val reportTime: UnixTime,
    val earliestSymptomTime: UserInput<UnixTime>,
    val feverSeverity: FeverSeverity,
    val coughSeverity: CoughSeverity,
    val breathlessness: Boolean,
    val muscleAches: Boolean,
    val lossSmellOrTaste: Boolean,
    val diarrhea: Boolean,
    val runnyNose: Boolean,
    val other: Boolean,
    val noSymptoms: Boolean, // https://github.com/Co-Epi/app-ios/issues/268#issuecomment-645583717
    var contactStart: UnixTime,
    var contactEnd: UnixTime,
    var minDistance: Float,
    var avgDistance: Float
) : Parcelable

enum class FeverSeverity {
    NONE, MILD, SERIOUS
}

enum class CoughSeverity {
    NONE, EXISTING, WET, DRY
}

fun FeverSeverity.toInt(): Int = when (this) {
    FeverSeverity.NONE -> 0
    FeverSeverity.MILD -> 1
    FeverSeverity.SERIOUS -> 2
}

fun toFeverSeverity(int: Int): FeverSeverity = when (int) {
    0 -> FeverSeverity.NONE
    1 -> FeverSeverity.MILD
    2 -> FeverSeverity.SERIOUS
    else -> error("Invalid value: $int")
}

fun CoughSeverity.toInt(): Int = when (this) {
    CoughSeverity.NONE -> 0
    CoughSeverity.EXISTING -> 1
    CoughSeverity.WET -> 2
    CoughSeverity.DRY -> 3
}

fun toCoughSeverity(int: Int): CoughSeverity = when (int) {
    0 -> CoughSeverity.NONE
    1 -> CoughSeverity.EXISTING
    2 -> CoughSeverity.WET
    3 -> CoughSeverity.DRY
    else -> error("Invalid value: $int")
}
