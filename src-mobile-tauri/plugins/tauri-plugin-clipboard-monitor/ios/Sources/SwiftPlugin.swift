import Foundation
import SwiftRs
import Tauri
import UIKit

class ClipboardMonitorPlugin: Plugin {
    private var monitor: Any?
    private var lastChangeCount: Int = 0
    private var lastText: String?

    @objc public func startMonitoring(_ invoke: Invoke) throws {
        let pasteboard = UIPasteboard.general
        lastChangeCount = pasteboard.changeCount

        if monitor == nil {
            monitor = NotificationCenter.default.addObserver(
                forName: UIPasteboard.changedNotification,
                object: nil,
                queue: .main
            ) { [weak self] _ in
                self?.handlePasteboardChange()
            }
        }

        invoke.resolve()
    }

    @objc public func stopMonitoring(_ invoke: Invoke) throws {
        if let monitor = monitor {
            NotificationCenter.default.removeObserver(monitor)
            self.monitor = nil
        }

        invoke.resolve()
    }

    @objc public func getText(_ invoke: Invoke) throws {
        let result: JsonObject = ["text": UIPasteboard.general.string as Any?]
        invoke.resolve(result)
    }

    @objc public func setText(_ invoke: Invoke) throws {
        let text = try invoke.parseArgs(String.self)
        UIPasteboard.general.string = text
        lastText = text
        invoke.resolve()
    }

    @objc public func getImage(_ invoke: Invoke) throws {
        guard let image = UIPasteboard.general.image,
              let data = image.pngData() else {
            let result: JsonObject = ["data_url": nil]
            invoke.resolve(result)
            return
        }

        let result: JsonObject = [
            "data_url": "data:image/png;base64,\(data.base64EncodedString())"
        ]
        invoke.resolve(result)
    }

    @objc public func setImage(_ invoke: Invoke) throws {
        let dataUrl = try invoke.parseArgs(String.self)
        let encoded = dataUrl.split(separator: ",", maxSplits: 1).last.map(String.init) ?? dataUrl
        guard let data = Data(base64Encoded: encoded),
              let image = UIImage(data: data) else {
            invoke.reject("图片内容无效")
            return
        }

        UIPasteboard.general.image = image
        invoke.resolve()
    }

    @objc public func clear(_ invoke: Invoke) throws {
        UIPasteboard.general.string = ""
        invoke.resolve()
    }

    private func handlePasteboardChange() {
        let pasteboard = UIPasteboard.general
        guard pasteboard.changeCount != lastChangeCount else { return }

        lastChangeCount = pasteboard.changeCount

        if let text = pasteboard.string, !text.isEmpty, text != lastText {
            lastText = text
            let payload: JSObject = [
                "text": text,
                "content_type": "text"
            ]
            trigger("clipboard-updated", data: payload)
        }
    }
}

@_cdecl("init_plugin_clipboard_monitor")
func initPlugin() -> Plugin {
    return ClipboardMonitorPlugin()
}
