package org.coepi.core

import org.coepi.core.domain.model.Length
import org.coepi.core.domain.model.LengthtUnit
import org.coepi.core.domain.model.LengthtUnit.FEET
import org.coepi.core.domain.model.LengthtUnit.METERS
import org.junit.Test
import org.junit.Assert.assertEquals

class LengthMeasurementTests {
    @Test
    fun metersToFeetPositiveIsCorrect() {
        assertEquals(
            Length(40.682415f, FEET),
            Length(12.4f, METERS).convert(FEET)
        )
    }

    @Test
    fun metersToFeetNegativeIsCorrect() {
        assertEquals(
            Length(-1.9685041f, FEET),
            Length(-0.6f, METERS).convert(FEET)
        )
    }

    @Test
    fun metersToFeet0IsCorrect() {
        assertEquals(
            Length(0f, FEET),
            Length(0f, METERS).convert(FEET)
        )
    }

    @Test
    fun feetToMetersPositiveIsCorrect() {
        assertEquals(
            Length(3.77952f, METERS),
            Length(12.4f, FEET).convert(METERS)
        )
    }

    @Test
    fun feetToMetersNegativeIsCorrect() {
        assertEquals(
            Length(-3761.96352f, METERS),
            Length(-12342.4f, FEET).convert(METERS)
        )
    }

    @Test
    fun feetToMeters0IsCorrect() {
        assertEquals(
            Length(0f, METERS),
            Length(0f, FEET).convert(METERS)
        )
    }
}
