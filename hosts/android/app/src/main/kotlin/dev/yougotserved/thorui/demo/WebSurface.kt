package dev.yougotserved.thorui.demo

import android.annotation.SuppressLint
import android.app.Activity
import android.graphics.Color
import android.net.Uri
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.RenderProcessGoneDetail
import android.webkit.WebSettings
import android.webkit.WebView
import android.view.ViewGroup
import androidx.webkit.WebViewAssetLoader
import androidx.webkit.WebViewClientCompat

class WebSurface private constructor(val view: WebView) {
    fun destroy() {
        view.stopLoading()
        view.loadUrl("about:blank")
        view.removeAllViews()
        view.destroy()
    }

    companion object {
        private const val ORIGIN = "https://appassets.androidplatform.net"

        fun create(activity: Activity, role: SurfaceRole): WebSurface {
            val loader = WebViewAssetLoader.Builder()
                .addPathHandler("/", WebViewAssetLoader.AssetsPathHandler(activity))
                .build()
            val webView = WebView(activity)
            configure(webView)
            webView.webViewClient = AssetClient(loader) { activity.recreate() }
            webView.setBackgroundColor(Color.rgb(5, 8, 20))
            webView.loadUrl("$ORIGIN/index.html?surface=${role.queryValue}&host=android&session=demo")
            return WebSurface(webView)
        }

        @SuppressLint("SetJavaScriptEnabled")
        private fun configure(webView: WebView) {
            webView.settings.javaScriptEnabled = true
            webView.settings.domStorageEnabled = true
            webView.settings.allowFileAccess = false
            webView.settings.allowContentAccess = false
            webView.settings.cacheMode = WebSettings.LOAD_DEFAULT
            webView.settings.mediaPlaybackRequiresUserGesture = false
            WebView.setWebContentsDebuggingEnabled(BuildConfig.DEBUG)
        }
    }

    @SuppressLint("MissingOnRenderProcessGone")
    private class AssetClient(
        private val loader: WebViewAssetLoader,
        private val recover: () -> Unit,
    ) : WebViewClientCompat() {
        override fun shouldInterceptRequest(
            view: WebView,
            request: WebResourceRequest,
        ): WebResourceResponse? = loader.shouldInterceptRequest(request.url)

        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean =
            request.url.host != Uri.parse(ORIGIN).host

        override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean {
            (view.parent as? ViewGroup)?.removeView(view)
            view.destroy()
            recover()
            return true
        }
    }
}
