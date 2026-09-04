package dev.yougotserved.thorui.demo

data class DisplayProfile(
    val id: Int,
    val width: Int,
    val height: Int,
    val launchAllowed: Boolean,
)

object DisplayPolicy {
    fun chooseCompanion(currentDisplayId: Int, displays: List<DisplayProfile>): DisplayProfile? =
        displays.asSequence()
            .filter { it.id != currentDisplayId && it.launchAllowed }
            .minWithOrNull(compareBy<DisplayProfile> { it.width }.thenBy { it.height })
}
