package com.kzeng.zkyt

import android.os.Bundle
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import org.json.JSONArray
import org.json.JSONObject
import java.io.BufferedReader
import java.io.InputStreamReader
import java.net.HttpURLConnection
import java.net.URL
import kotlin.concurrent.thread

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    WindowCompat.setDecorFitsSystemWindows(window, true)
    super.onCreate(savedInstanceState)
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)

    webView.settings.mediaPlaybackRequiresUserGesture = false
    webView.addJavascriptInterface(AndroidHttpBridge(this, webView), "ZkytAndroidHttp")

    ViewCompat.setOnApplyWindowInsetsListener(webView) { view, insets ->
      val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
      view.setPadding(0, systemBars.top, 0, systemBars.bottom)
      insets
    }
  }
}

class AndroidHttpBridge(
  private val activity: MainActivity,
  private val webView: WebView,
) {
  @JavascriptInterface
  fun request(payloadJson: String): String {
    return performRequest(payloadJson)
  }

  @JavascriptInterface
  fun requestAsync(payloadJson: String, callbackId: String) {
    thread(start = true) {
      val response = performRequest(payloadJson)

      activity.runOnUiThread {
        webView.evaluateJavascript(
          "window.__zkytAndroidHttpResolve(${JSONObject.quote(callbackId)}, ${JSONObject.quote(response)})",
          null
        )
      }
    }
  }

  private fun performRequest(payloadJson: String): String {
    return try {
      val payload = JSONObject(payloadJson)
      val url = URL(payload.getString("url"))

      if (url.protocol != "https") {
        throw IllegalArgumentException("only https URLs are allowed")
      }

      val connection = (url.openConnection() as HttpURLConnection).apply {
        requestMethod = payload.optString("method", "GET")
        connectTimeout = 12000
        readTimeout = 12000
        instanceFollowRedirects = true
        setRequestProperty("User-Agent", "ZKYT Tauri Android")
      }

      val headers = payload.optJSONObject("headers")
      if (headers != null) {
        val keys = headers.keys()
        while (keys.hasNext()) {
          val name = keys.next()
          val lowerName = name.lowercase()
          if (lowerName in setOf("host", "connection", "content-length", "cookie")) {
            continue
          }

          val value = headers.optString(name, "")
          if (value.isNotEmpty()) {
            connection.setRequestProperty(name, value)
          }
        }
      }

      if (!payload.isNull("body")) {
        val bodyBytes = payload.getString("body").toByteArray(Charsets.UTF_8)
        connection.doOutput = true
        connection.setFixedLengthStreamingMode(bodyBytes.size)
        connection.outputStream.use { output ->
          output.write(bodyBytes)
        }
      }

      val status = connection.responseCode
      val stream = if (status in 200..399) connection.inputStream else connection.errorStream
      val body = stream?.use { input ->
        BufferedReader(InputStreamReader(input, Charsets.UTF_8)).use { reader ->
          reader.readText()
        }
      } ?: ""

      val responseHeaders = JSONArray()
      connection.headerFields.forEach { (name, values) ->
        if (name != null) {
          values.forEach { value ->
            responseHeaders.put(JSONArray().put(name).put(value))
          }
        }
      }

      JSONObject()
        .put("status", status)
        .put("statusText", connection.responseMessage ?: "")
        .put("headers", responseHeaders)
        .put("body", body)
        .toString()
    } catch (error: Exception) {
      JSONObject()
        .put("error", error.toString())
        .toString()
    }
  }
}
