package dev.yougotserved.thorui.demo

import android.os.Bundle
import android.util.Log

class CompanionActivity : SurfaceActivity() {
    override val surfaceRole = SurfaceRole.COMPANION

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val expected = intent.getIntExtra(DisplayLauncher.EXPECTED_DISPLAY_ID, -1)
        val actual = DisplayLauncher.activityDisplayId(this)
        Log.i("ThorUIDisplay", "Companion expected=$expected actual=$actual")
    }
}
