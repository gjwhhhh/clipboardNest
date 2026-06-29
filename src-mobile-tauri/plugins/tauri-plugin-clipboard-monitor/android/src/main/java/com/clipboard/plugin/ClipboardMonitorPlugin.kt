package com.clipboard.plugin

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.graphics.Bitmap
import android.graphics.ImageDecoder
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import android.util.Base64
import androidx.core.content.FileProvider
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.ByteArrayOutputStream
import java.io.File

@TauriPlugin
class ClipboardMonitorPlugin(private val activity: Activity) : Plugin(activity) {
    private val clipboardManager: ClipboardManager by lazy {
        activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    }
    private var listener: ClipboardManager.OnPrimaryClipChangedListener? = null
    private var lastClipText: String? = null

    @Command
    fun startMonitoring(invoke: Invoke) {
        if (listener == null) {
            listener = ClipboardManager.OnPrimaryClipChangedListener {
                handleClipboardChange()
            }
            clipboardManager.addPrimaryClipChangedListener(listener)
        }
        invoke.resolve()
    }

    @Command
    fun stopMonitoring(invoke: Invoke) {
        listener?.let { clipboardManager.removePrimaryClipChangedListener(it) }
        listener = null
        invoke.resolve()
    }

    @Command
    fun getText(invoke: Invoke) {
        try {
            val result = JSObject()
            result.put("text", readText())
            invoke.resolve(result)
        } catch (ex: Exception) {
            invoke.reject(ex.message, ex)
        }
    }

    @Command
    fun setText(invoke: Invoke) {
        try {
            val text = invoke.parseArgs(String::class.java)
            val clipData = ClipData.newPlainText("clipboard", text)
            clipboardManager.setPrimaryClip(clipData)
            lastClipText = text
            invoke.resolve()
        } catch (ex: Exception) {
            invoke.reject(ex.message, ex)
        }
    }

    @Command
    fun getImage(invoke: Invoke) {
        try {
            val result = JSObject()
            val imageBytes = readImage()
            result.put("data_url", imageBytes?.let { bytes ->
                "data:image/png;base64," + Base64.encodeToString(bytes, Base64.NO_WRAP)
            })
            invoke.resolve(result)
        } catch (ex: Exception) {
            invoke.reject(ex.message, ex)
        }
    }

    @Command
    fun setImage(invoke: Invoke) {
        try {
            val dataUrl = invoke.parseArgs(String::class.java)
            val bytes = decodeDataUrl(dataUrl)
            val imagesDir = File(activity.cacheDir, "clipboard_images").apply { mkdirs() }
            val imageFile = File(imagesDir, "clipboard.png")
            imageFile.writeBytes(bytes)

            val uri = FileProvider.getUriForFile(
                activity,
                "${activity.packageName}.fileprovider",
                imageFile
            )
            val clipData = ClipData.newUri(activity.contentResolver, "clipboard image", uri)
            clipboardManager.setPrimaryClip(clipData)
            invoke.resolve()
        } catch (ex: Exception) {
            invoke.reject(ex.message, ex)
        }
    }

    private fun handleClipboardChange() {
        val text = readText()
        if (!text.isNullOrBlank() && text != lastClipText) {
            lastClipText = text
            val payload = JSObject()
            payload.put("text", text)
            payload.put("content_type", "text")
            trigger("clipboard-updated", payload)
        }
    }

    private fun readText(): String? {
        val clipData = clipboardManager.primaryClip ?: return null
        if (clipData.itemCount == 0) return null
        return clipData.getItemAt(0).text?.toString()
    }

    private fun readImage(): ByteArray? {
        val clipData = clipboardManager.primaryClip ?: return null
        if (clipData.itemCount == 0) return null

        val uri: Uri = clipData.getItemAt(0).uri ?: return null
        return try {
            val bitmap = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                val source = ImageDecoder.createSource(activity.contentResolver, uri)
                ImageDecoder.decodeBitmap(source)
            } else {
                @Suppress("DEPRECATION")
                MediaStore.Images.Media.getBitmap(activity.contentResolver, uri)
            }
            val outputStream = ByteArrayOutputStream()
            bitmap.compress(Bitmap.CompressFormat.PNG, 100, outputStream)
            outputStream.toByteArray()
        } catch (_: Exception) {
            null
        }
    }

    private fun decodeDataUrl(dataUrl: String): ByteArray {
        val encoded = dataUrl.substringAfter(",", dataUrl)
        return Base64.decode(encoded, Base64.DEFAULT)
    }
}
