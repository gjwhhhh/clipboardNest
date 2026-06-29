// swift-tools-version:5.3

import PackageDescription

let package = Package(
  name: "tauri-plugin-clipboard-monitor",
  platforms: [
    .macOS(.v10_13),
    .iOS(.v13),
  ],
  products: [
    .library(
      name: "tauri-plugin-clipboard-monitor",
      type: .static,
      targets: ["tauri-plugin-clipboard-monitor"])
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-clipboard-monitor",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources")
  ]
)
