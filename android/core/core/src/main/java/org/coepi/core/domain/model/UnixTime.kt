package org.coepi.core.domain.model

import android.os.Parcelable
import kotlinx.android.parcel.Parcelize
import org.threeten.bp.Instant
import java.io.Serializable
import java.util.Date

@Parcelize
class UnixTime private constructor(val value: Long) : Parcelable, Serializable {

    init {
        if (value < 0) error("Invalid unix time: $value")
    }

    companion object {
        fun fromValue(value: Long): UnixTime = UnixTime(value)
        fun minTimestamp(): UnixTime = UnixTime(0)
        fun now(): UnixTime = fromDate(Date())
        fun fromDate(date: Date) = UnixTime(date.time / 1000)
        fun fromInstant(instant: Instant) = UnixTime(instant.epochSecond)
    }

    override fun equals(other: Any?): Boolean =
        other is UnixTime && value == other.value

    override fun hashCode(): Int = value.hashCode()

    override fun toString(): String =
        "$value, ${toDate()}"

    fun toDate() =
        Date(value * 1000)

}
