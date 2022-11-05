test_core:
	cargo check
	cargo test
	cargo fmt --all -- --check

test_python:
	source ./venv/bin/activate; \
	pip install -r ./python/requirements.txt; \
	cd ./python; \
	maturin build; \
	pytest;

test_swift:
	swift test

swift_clean:
	cd ./swift; \
	rm -rf NewYorkCalculateRust.xcframework; \
	rm libnew_york_calculate_macos.a; \
	rm libnew_york_calculate_iossimulator.a;

swift_build:
	cd ./swift; \
	cargo build --release --target aarch64-apple-ios; \
	cargo build --release --target x86_64-apple-darwin; \
	cargo build --release --target aarch64-apple-darwin; \
	cargo build --release --target x86_64-apple-ios; \
	cargo build --release --target aarch64-apple-ios-sim; \
	lipo -create target/x86_64-apple-darwin/release/libnew_york_calculate.a target/aarch64-apple-darwin/release/libnew_york_calculate.a -output libnew_york_calculate_macos.a; \
	lipo -create target/x86_64-apple-ios/release/libnew_york_calculate.a target/aarch64-apple-ios-sim/release/libnew_york_calculate.a -output libnew_york_calculate_iossimulator.a; \
	xcodebuild -create-xcframework -library ./libnew_york_calculate_macos.a -headers ./include/ -library ./libnew_york_calculate_iossimulator.a -headers ./include/ -library ./target/aarch64-apple-ios/release/libnew_york_calculate.a -headers ./include/ -output NewYorkCalculateRust.xcframework;

swift: swift_clean swift_build

test: test_core test_python