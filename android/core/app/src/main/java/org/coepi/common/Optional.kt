package org.coepi.android.common

import org.coepi.android.common.Optional.None
import org.coepi.android.common.Optional.Some

/**
 * To deal with RxJava's limitation of not allowing null.
 */
sealed class Optional<out T> {
    class Some<T>(val value: T) : Optional<T>()
    object None : Optional<Nothing>()

    companion object {
        fun <T> from(value: T?): Optional<T> =
            value?.let { Some(it) } ?: None
    }
}

fun <T>Optional<T>.toNullable(): T? = when (this) {
    is Some -> this.value
    is None -> null
}
