# Justfile for lockdir-rs

# Define variables
name := "lockdir"
target_dir := "target"
release_dir := target_dir / "release"
debug_dir := target_dir / "debug"
release_binary := release_dir / name
debug_binary := debug_dir / name
install_dir := "/usr/local/bin"

# Default recipe to run when just is called without arguments
default: help

# Display available commands and their descriptions
help:
    just -l

# Check code without building
check:
    cargo check

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the binary in debug mode
run *ARGS:
    cargo run -- {{ARGS}}

# Run the binary in release mode
run-release *ARGS:
    cargo run --release -- {{ARGS}}

# Clean the build artifacts
clean:
    cargo clean

# Run tests
test:
    cargo test

# Run clippy with pedantic lints
clippy:
    cargo clippy -- -W clippy::pedantic

# Install using cargo (user-level installation)
install:
    cargo install --path .

# Install the binary to system-wide location (requires sudo)
install-system: build-release
    @echo "Installing {{name}} to {{install_dir}}"
    @sudo cp {{release_binary}} {{install_dir}}
    @echo "Installation complete!"

# Uninstall the application from cargo bin directory
uninstall:
    @echo "Removing {{name}} from cargo bin directory"
    cargo uninstall {{name}}
    @echo "Uninstallation complete!"

# Uninstall the application from system-wide location (requires sudo)
uninstall-system:
    @echo "Removing {{name}} from {{install_dir}}"
    @sudo rm -f {{install_dir}}/{{name}}
    @echo "Uninstallation complete!"
