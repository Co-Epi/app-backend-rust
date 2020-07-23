package org.coepi.core.domain.model

import android.os.Parcelable
import kotlinx.android.parcel.Parcelize
import java.io.Serializable

sealed class LengthMeasurement : Parcelable, Serializable {
    @Parcelize
    data class Meters(val value: Float) : LengthMeasurement() {
        override fun toFeet(): Feet = Feet(value * 3.28084f)
    }

    @Parcelize
    data class Feet(val value: Float) : LengthMeasurement() {
        override fun toMeters(): Meters = Meters(value * 0.3048f)
    }

    open fun toFeet(): Feet = when (this) {
        is Feet -> this
        is Meters -> toFeet()
    }

    open fun toMeters(): Meters = when (this) {
        is Feet -> toMeters()
        is Meters -> this
    }
}
