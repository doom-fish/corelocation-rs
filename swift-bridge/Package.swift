// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "CoreLocationBridge",
    platforms: [
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "CoreLocationBridge",
            type: .static,
            targets: ["CoreLocationBridge"]
        )
    ],
    targets: [
        .target(
            name: "CoreLocationBridge",
            path: "Sources/CoreLocationBridge",
            publicHeadersPath: "include"
        )
    ]
)
