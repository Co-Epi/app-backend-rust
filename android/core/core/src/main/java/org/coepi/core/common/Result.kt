package org.coepi.common

import org.coepi.common.Result.Failure
import org.coepi.common.Result.Success

sealed class Result<out T, out E> {
    data class Success<out T>(val success: T) : Result<T, Nothing>()
    data class Failure<out E>(val error: E) : Result<Nothing, E>()
}

fun <T, E> Result<T, E>.fallback(value: T): T = when (this) {
    is Success -> this.success
    is Failure -> value
}

fun <T, U, E> Result<T, E>.map(f: (T) -> U): Result<U, E> = when (this) {
    is Success -> Success(f(success))
    is Failure -> this
}

fun <T, U, E> Result<T, E>.flatMap(f: (T) -> Result<U, E>): Result<U, E> = when (this) {
    is Success -> f(success)
    is Failure -> this
}

fun <T, E> Result<T, E>.doIfSuccess(f: (T) -> Unit): Result<T, E> {
    when (this) {
        is Success -> f(success)
        is Failure -> {}
    }
    return this
}

fun <T, E> Result<T, E>.doIfError(f: (E) -> Unit): Result<T, E> {
    when (this) {
        is Success -> {}
        is Failure -> f(error)
    }
    return this
}

fun <T, E> Result<T, E>.successOrNull(): T? =
    when (this) {
        is Success -> success
        is Failure -> null
    }

fun <T, E> Result<T, E>.failureOrNull(): E? =
    when (this) {
        is Success -> null
        is Failure -> error
    }

fun <T, E> Result<T, E>.isSuccess(): Boolean =
    this is Success

fun <T, E> Result<T, E>.isFailure(): Boolean =
    this is Failure

fun <T: Any, E: Any> List<Result<T, E>>.group(): Pair<List<T>, List<E>> =
   Pair(mapNotNull { it.successOrNull() }, mapNotNull { it.failureOrNull() } )
