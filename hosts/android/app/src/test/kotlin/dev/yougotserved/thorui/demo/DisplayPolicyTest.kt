package dev.yougotserved.thorui.demo

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DisplayPolicyTest {
    @Test
    fun choosesTheSmallestAllowedNonCurrentDisplay() {
        val displays = listOf(
            DisplayProfile(0, 1920, 1080, true),
            DisplayProfile(2, 1600, 900, false),
            DisplayProfile(3, 1240, 1080, true),
        )

        assertEquals(3, DisplayPolicy.chooseCompanion(0, displays)?.id)
    }

    @Test
    fun returnsNullWithoutAnEligibleCompanion() {
        val displays = listOf(DisplayProfile(0, 1920, 1080, true))

        assertNull(DisplayPolicy.chooseCompanion(0, displays))
    }
}
