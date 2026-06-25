package com.clipboard.plugin

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.graphics.Bitmap
import android.graphics.ImageDecoder
import android.os.Build
import android.provider.MediaStore
import java.io.ByteArrayOutputStream

class ClipboardMonitorPlugin(private val context: Context) {
    private var clipboardManager: ClipboardManager? = null
    private var listener: ClipboardManager.OnPrimaryClipChangedListener? = null
    private var onClipboardChanged: ((String) -> Unit)? = null
    private var lastClipText: String? = null

    /// 设置剪切板变化回调
    fun setCallback(callback: (String) -> Unit) {
        this.onClipboardChanged = callback
    }

    /// 开始监听剪切板
    fun startMonitoring() {
        clipboardManager = context.getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager

        clipboardManager?.let { manager ->
            listener = ClipboardManager.OnPrimaryClipChangedListener {
                handleClipboardChange()
            }
            manager.addPrimaryClipChangedListener(listener)
        }

        println("[ClipboardMonitor] 开始监听剪切板")
    }

    /// 停止监听剪切板
    fun stopMonitoring() {
        listener?.let { listener ->
            clipboardManager?.removePrimaryClipChangedListener(listener)
        }
        listener = null

        println("[ClipboardMonitor] 停止监听剪切板")
    }

    /// 处理剪切板变化
    private fun handleClipboardChange() {
        val clipData = clipboardManager?.primaryClip ?: return

        if (clipData.itemCount > 0) {
            val item = clipData.getItemAt(0)
            val text = item.text?.toString()

            if (!text.isNullOrEmpty() && text != lastClipText) {
                lastClipText = text
                onClipboardChanged?.invoke(text)
            }
        }
    }

    /// 获取剪切板文本
    fun getText(): String? {
        val clipData = clipboardManager?.primaryClip ?: return null
        return if (clipData.itemCount > 0) {
            clipData.getItemAt(0).text?.toString()
        } else {
            null
        }
    }

    /// 设置剪切板文本
    fun setText(text: String) {
        val clipData = ClipData.newPlainText("clipboard", text)
        clipboardManager?.setPrimaryClip(clipData)
    }

    /// 获取剪切板图片
    fun getImage(): ByteArray? {
        val clipData = clipboardManager?.primaryClip ?: return null
        if (clipData.itemCount == 0) return null

        val item = clipData.getItemAt(0)
        val uri = item.uri ?: return null

        return try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                val source = ImageDecoder.createSource(context.contentResolver, uri)
                val bitmap = ImageDecoder.decodeBitmap(source)
                val outputStream = ByteArrayOutputStream()
                bitmap.compress(Bitmap.CompressFormat.PNG, 100, outputStream)
                outputStream.toByteArray()
            } else {
                @Suppress("DEPRECATION")
                val bitmap = MediaStore.Images.Media.getBitmap(context.contentResolver, uri)
                val outputStream = ByteArrayOutputStream()
                bitmap.compress(Bitmap.CompressFormat.PNG, 100, outputStream)
                outputStream.toByteArray()
            }
        } catch (e: Exception) {
            println("[ClipboardMonitor] 获取图片失败: ${e.message}")
            null
        }
    }

    /// 设置剪切板图片
    fun setImage(imageData: ByteArray) {
        // Android 设置图片需要通过 ContentProvider，这里简化处理
        println("[ClipboardMonitor] 设置图片功能暂未实现")
    }

    /// 检查剪切板是否有内容
    fun hasContent(): Boolean {
        val clipData = clipboardManager?.primaryClip ?: return false
        return clipData.itemCount > 0
    }

    /// 清空剪切板
    fun clear() {
        val clipData = ClipData.newPlainText("", "")
        clipboardManager?.setPrimaryClip(clipData)
    }
}
