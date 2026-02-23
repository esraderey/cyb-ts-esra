.PHONY: dev build run clean check test dmg nu-build

# Development: run cyb-shell (Browser mode loads https://localhost:3001)
# Start cyb-ts dev server first: cd .. && deno task start
dev:
	cargo run -p cyb-shell

# Build all workspace members (debug)
build:
	cargo build -p cyb-shell
	cargo build -p cyb-services

# Build release
release:
	cargo build --release -p cyb-shell
	cargo build --release -p cyb-services

# Run release binary
run:
	cargo run --release -p cyb-shell

# Check all (fast, no codegen)
check:
	cargo check --workspace

# Run tests
test:
	cargo test -p cyb-services

# Clean build artifacts
clean:
	cargo clean

# Build nushell binary (separate workspace)
nu-build:
	cargo build --release --manifest-path vendor/nushell/Cargo.toml

# Build macOS .dmg
dmg: release nu-build
	rm -rf target/release/cyb.app
	mkdir -p target/release/cyb.app/Contents/MacOS
	mkdir -p target/release/cyb.app/Contents/Resources
	cp target/release/cyb-shell target/release/cyb.app/Contents/MacOS/cyb-shell
	cp vendor/nushell/target/release/nu target/release/cyb.app/Contents/MacOS/nu
	mkdir -p target/release/cyb.app/Contents/Resources/nushell
	cp cyb-shell/assets/nu-config/env.nu target/release/cyb.app/Contents/Resources/nushell/
	cp cyb-shell/assets/nu-config/config.nu target/release/cyb.app/Contents/Resources/nushell/
	@if [ -d "cyb-shell/web-dist" ]; then \
		cp -r cyb-shell/web-dist target/release/cyb.app/Contents/Resources/web-dist; \
	fi
	/usr/libexec/PlistBuddy -c "Add :CFBundleName string cyb" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :CFBundleDisplayName string cyb" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :CFBundleExecutable string cyb-shell" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :CFBundleIdentifier string ai.cyb.app" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :CFBundleVersion string 0.1.0" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string 0.1.0" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :CFBundlePackageType string APPL" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :LSMinimumSystemVersion string 13.0" target/release/cyb.app/Contents/Info.plist
	/usr/libexec/PlistBuddy -c "Add :NSHighResolutionCapable bool true" target/release/cyb.app/Contents/Info.plist
	rm -f target/release/cyb.dmg
	hdiutil create -volname "cyb" -srcfolder target/release/cyb.app -ov -format UDZO target/release/cyb.dmg

# Full dev workflow
# Terminal 1: cd .. && deno task start     (cyb-ts HTTPS :3001)
# Terminal 2: make dev                     (Bevy shell → Browser mode)
# Terminal 3: cd cyb-ui && dx serve        (Optional: Dioxus UI dev)
