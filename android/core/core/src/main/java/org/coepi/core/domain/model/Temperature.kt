package org.coepi.core.domain.model

import android.os.Parcelable
import kotlinx.android.parcel.Parcelize
import java.io.Serializable

sealed class Temperature : Parcelable, Serializable {
    @Parcelize
    data class Celsius(val value: Float) : Temperature() {
        override fun toFarenheit(): Fahrenheit =
            Fahrenheit((9 / 5.0 * value + 32).toFloat())
    }

    @Parcelize
    data class Fahrenheit(val value: Float) : Temperature() {
        override fun toCelsius(): Celsius =
            Celsius((5 / 9.0 * (value - 32)).toFloat())
    }

    open fun toCelsius(): Celsius = when (this) {
        is Celsius -> this
        is Fahrenheit -> toCelsius()
    }

    open fun toFarenheit(): Fahrenheit = when (this) {
        is Fahrenheit -> this
        is Celsius -> toFarenheit()
    }
}
