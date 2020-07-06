package org.coepi.core

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.ExperimentalCoroutinesApi
import org.coepi.core.jni.BenchmarksIntClass
import org.coepi.core.jni.JniApi
import org.junit.Ignore
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */

// NOTE: Ideally this should use benchmark runner
// https://developer.android.com/studio/profile/build-benchmarks-without-gradle
// The documentation is incomplete, though, and it was consuming too much time
// For now "manual" benchmarks

@Ignore("Benchmarks")
@ExperimentalCoroutinesApi
@RunWith(AndroidJUnit4::class)
class JniBenchmarks {

    private val jniApi = JniApi()
    private val nonJniApi = NonJniApi()

    @Test
    fun benchmarkNoopWithJni() {
        // 70ms, 97ms, 80ms, 85ms
        benchmark("benchmarkNoop") {
            for (i in 0..1000000) {
                jniApi.noopForBenchmarks()
            }
        }
    }

    @Test
    fun benchmarkNoopWithoutJni() {
        // 22ms, 22ms, 21ms, 22ms
        benchmark("benchmarkNoopWithoutJni") {
            for (i in 0..1000000) {
                nonJniApi.noopForBenchmarks()
            }
        }
    }

    @Test
    fun benchmarkSendReceiveIntWithJni() {
        // 72ms, 79ms, 89ms, 90ms
        benchmark("benchmarkSendReceiveIntWithJni") {
            for (i in 0..1000000) {
                jniApi.sendReceiveIntForBenchmarks(1)
            }
        }
    }

    @Test
    fun benchmarkSendReceiveIntWithoutJni() {
        // 22ms, 10ms, 23ms, 23ms
        benchmark("benchmarkSendReceiveIntWithoutJni") {
            for (i in 0..1000000) {
                nonJniApi.sendReceiveIntForBenchmarks(1)
            }
        }
    }

    @Test
    fun benchmarkSendReceiveStringWithJni() {
        // 4596ms, 4343ms, 4561ms, 4360ms
        benchmark("benchmarkSendReceiveStringWithJni") {
            for (i in 0..1000000) {
                jniApi.sendCreateStringForBenchmarks("hello")
            }
        }
    }

    @Test
    fun benchmarkSendReceiveStringDontUseInputWithJni() {
        // 2415ms, 2439ms, 2415ms, 2455ms
        benchmark("sendCreateStringDontUseInputForBenchmarks") {
            for (i in 0..1000000) {
                jniApi.sendCreateStringDontUseInputForBenchmarks("hello")
            }
        }
    }

    @Test
    fun benchmarkSendReceiveStringWithoutJni() {
        // 12ms, 31ms, 27ms, 27ms
        benchmark("benchmarkSendReceiveStringWithoutJni") {
            for (i in 0..1000000) {
                nonJniApi.sendCreateStringForBenchmarks("hello")
            }
        }
    }

    @Test
    fun benchmarkSendClassWithJni() {
        // 8639ms, 8690ms, 8476ms, 8694ms
        benchmark("benchmarkClassStructWithJni") {
            for (i in 0..1000000) {
                jniApi.sendClassForBenchmarks(BenchmarksIntClass(1))
            }
        }
    }

    @Test
    fun benchmarkSendClassWithoutJni() {
        // 39ms, 31ms, 32ms, 41ms
        benchmark("benchmarkClassStructWithoutJni") {
            for (i in 0..1000000) {
                nonJniApi.sendClassForBenchmarks(BenchmarksIntClass(1))
            }
        }
    }

    @Test
    fun benchmarkReturnClassWithJni() {
        // 37929ms, 39027ms, 38815ms, 38711ms
        benchmark("benchmarkReturnClassWithJni") {
            for (i in 0..1000000) {
                jniApi.returnClassForBenchmarks()
            }
        }
    }

    @Test
    fun benchmarkReturnClassWithoutJni() {
        // 16ms, 40ms, 41ms, 30ms
        benchmark("benchmarkReturnClassWithoutJni") {
            for (i in 0..1000000) {
                nonJniApi.returnClassForBenchmarks()
            }
        }
    }

    private fun benchmark(label: String, f: () -> Unit) {
        val tsLong = System.currentTimeMillis()
        f()
        val ttLong = System.currentTimeMillis() - tsLong
        println("$label took: ${ttLong}ms")
    }
}

private class NonJniApi {
    fun noopForBenchmarks() {}

    fun sendReceiveIntForBenchmarks(i: Int): Int = 1

    fun sendCreateStringForBenchmarks(string: String): String = "Return string"

    fun sendClassForBenchmarks(c: BenchmarksIntClass) {}

    fun returnClassForBenchmarks() = BenchmarksIntClass(1)
}
