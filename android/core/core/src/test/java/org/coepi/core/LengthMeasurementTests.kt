package org.coepi.core

import org.coepi.core.domain.model.LengthMeasurement.Feet
import org.coepi.core.domain.model.LengthMeasurement.Meters
import org.junit.Test
import org.junit.Assert.assertEquals

class LengthMeasurementTests {
    @Test
    fun metersToFeetPositiveIsCorrect() {
        assertEquals(
            Feet(40.682415f),
            Meters(12.4f).toFeet()
        )
    }

    @Test
    fun metersToFeetNegativeIsCorrect() {
        assertEquals(
            Feet(-1.9685041f),
            Meters(-0.6f).toFeet()
        )
    }

    @Test
    fun metersToFeet0IsCorrect() {
        assertEquals(
            Feet(0f),
            Meters(0f).toFeet()
        )
    }

    @Test
    fun feetToMetersPositiveIsCorrect() {
        assertEquals(
            Meters(3.77952f),
            Feet(12.4f).toMeters()
        )
    }

    @Test
    fun feetToMetersNegativeIsCorrect() {
        assertEquals(
            Meters(-3761.96352f),
            Feet(-12342.4f).toMeters()
        )
    }

    @Test
    fun feetToMeters0IsCorrect() {
        assertEquals(
            Meters(0f),
            Feet(0f).toMeters()
        )
    }
}
