package org.coepi.core

import org.junit.Test
import org.junit.Assert.assertEquals
import org.coepi.core.domain.model.Temperature.Celsius
import org.coepi.core.domain.model.Temperature.Fahrenheit

class TemperatureUnitTest {
    @Test
    fun celsiusToFahrenheit_isCorrect() {
        assertEquals(
            Fahrenheit(32.0.toFloat()),
            Celsius(0.0.toFloat()).toFarenheit()
        )
    }

    @Test
    fun fahrenheitToCelsius_isCorrect() {
        assertEquals(
            Celsius(0.0.toFloat()),
            Fahrenheit(32.0.toFloat()).toCelsius()
        )
    }
}
