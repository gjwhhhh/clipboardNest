import Foundation
import UIKit

@objc public class ClipboardMonitorPlugin: NSObject {
    private var monitor: Any?
    private var lastChangeCount: Int = 0
    private var onClipboardChanged: ((String) -> Void)?

    /// 设置剪切板变化回调
    @objc public func setCallback(_ callback: @escaping (String) -> Void) {
        self.onClipboardChanged = callback
    }

    /// 开始监听剪切板
    @objc public func startMonitoring() {
        let pasteboard = UIPasteboard.general
        lastChangeCount = pasteboard.changeCount

        monitor = NotificationCenter.default.addObserver(
            forName: UIPasteboard.changedNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.handlePasteboardChange()
        }

        print("[ClipboardMonitor] 开始监听剪切板")
    }

    /// 停止监听剪切板
    @objc public func stopMonitoring() {
        if let monitor = monitor {
            NotificationCenter.default.removeObserver(monitor)
            self.monitor = nil
        }

        print("[ClipboardMonitor] 停止监听剪切板")
    }

    /// 处理剪切板变化
    private func handlePasteboardChange() {
        let pasteboard = UIPasteboard.general
        guard pasteboard.changeCount != lastChangeCount else { return }

        lastChangeCount = pasteboard.changeCount

        // 获取文本内容
        if let text = pasteboard.string, !text.isEmpty {
            onClipboardChanged?(text)
        }
    }

    /// 获取剪切板文本
    @objc public func getText() -> String? {
        return UIPasteboard.general.string
    }

    /// 设置剪切板文本
    @objc public func setText(_ text: String) {
        UIPasteboard.general.string = text
    }

    /// 获取剪切板图片
    @objc public func getImage() -> Data? {
        guard let image = UIPasteboard.general.image else { return nil }
        return image.pngData()
    }

    /// 设置剪切板图片
    @objc public func setImage(_ imageData: Data) {
        guard let image = UIImage(data: imageData) else { return }
        UIPasteboard.general.image = image
    }

    /// 检查剪切板是否有内容
    @objc public func hasContent() -> Bool {
        let pasteboard = UIPasteboard.general
        return pasteboard.hasStrings || pasteboard.hasImages
    }

    /// 清空剪切板
    @objc public func clear() {
        UIPasteboard.general.string = ""
    }
}
