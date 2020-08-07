package org.coepi.core.domain.model

import android.os.Parcelable
import kotlinx.android.parcel.Parcelize
import org.coepi.core.domain.model.LengthtUnit.FEET
import org.coepi.core.domain.model.LengthtUnit.METERS

enum class LengthtUnit {
    METERS, FEET
}

@Parcelize
data class Length(val value: Float, val unit: LengthtUnit) : Parcelable {

    fun convert(unit: LengthtUnit): Length =
        when {
            this.unit == METERS && unit == METERS -> this
            this.unit == METERS && unit == FEET ->
                Length(value * 3.28084f, unit)
            this.unit == FEET && unit == METERS ->
                Length(value * 0.3048f, unit)
            this.unit == FEET && unit == FEET -> this
            else -> error("Not handled: $this -> $unit")
        }
}
