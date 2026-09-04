package dev.yougotserved.thorui.demo

import android.os.Bundle
import android.widget.Toast

class MainActivity : SurfaceActivity() {
    override val surfaceRole = SurfaceRole.MAIN
    private var launchAttempted = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        launchAttempted = savedInstanceState?.getBoolean(LAUNCH_ATTEMPTED) ?: false
    }

    override fun onResume() {
        super.onResume()
        if (launchAttempted) return
        launchAttempted = true
        if (!DisplayLauncher.launchCompanion(this)) {
            Toast.makeText(this, R.string.companion_unavailable, Toast.LENGTH_LONG).show()
        }
    }

    override fun onSaveInstanceState(outState: Bundle) {
        outState.putBoolean(LAUNCH_ATTEMPTED, launchAttempted)
        super.onSaveInstanceState(outState)
    }

    companion object {
        private const val LAUNCH_ATTEMPTED = "launch_attempted"
    }
}
