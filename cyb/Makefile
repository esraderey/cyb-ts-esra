.PHONY: dev build run clean check test

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

# Full dev workflow
# Terminal 1: cd .. && deno task start     (cyb-ts HTTPS :3001)
# Terminal 2: make dev                     (Bevy shell → Browser mode)
# Terminal 3: cd cyb-ui && dx serve        (Optional: Dioxus UI dev)
