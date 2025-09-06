// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "SwiftWebRTC",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "SwiftWebRTC",
            targets: ["SwiftWebRTC"]),
    ],
    targets: [
        .target(
            name: "SwiftWebRTC",
            dependencies: [],
            path: "Sources/SwiftWebRTC",
            linkerSettings: [
                .linkedLibrary("rust_ffi"),
                .unsafeFlags(["-L", "../../rust-ffi/target/aarch64-apple-ios-sim/release"], .when(platforms: [.iOS])),
                .unsafeFlags(["-L", "../../rust-ffi/target/release"], .when(platforms: [.macOS]))
            ]
        ),
        .testTarget(
            name: "SwiftWebRTCTests",
            dependencies: ["SwiftWebRTC"],
            path: "Tests/SwiftWebRTCTests"),
    ]
)