// swift-tools-version: 5.7
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "NewYorkCalculate",
    platforms: [
        .iOS(.v13),
        .macOS(.v11)
    ],
    products: [
        .library(
            name: "NewYorkCalculate",
            targets: ["NewYorkCalculate"]
        ),
    ],
    dependencies: [    ],
    targets: [
        .target(
            name: "NewYorkCalculate",
            dependencies: ["NewYorkCalculateRust"],
            path: "swift/Sources"
        ),
//        .binaryTarget(
//            name: "NewYorkCalculateRustRemote",
//            url: "https://github.com/vivalaakam/new_york_calculate/releases/download/v0.0.56/bundle.zip",
//            checksum: "6aea2787461faaad12d1ccf9eb9dc12f48c600d76e979570a56abdf13d80fe0e"),
        .binaryTarget(
            name: "NewYorkCalculateRust",
            path: "swift/NewYorkCalculateRust.xcframework"),
        .testTarget(
            name: "NewYorkCalculateTests",
            dependencies: ["NewYorkCalculate"],
            path: "swift/Tests")
    ]
)
