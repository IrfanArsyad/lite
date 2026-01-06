# Makefile for lite editor

.PHONY: all build release test clean install uninstall deb rpm help

# Variables
CARGO := cargo
INSTALL_DIR := /usr/local/bin
MAN_DIR := /usr/local/share/man/man1
COMPLETIONS_DIR_BASH := /etc/bash_completion.d
COMPLETIONS_DIR_FISH := /usr/share/fish/vendor_completions.d
COMPLETIONS_DIR_ZSH := /usr/share/zsh/vendor-completions
BINARY := lite
TARGET := target/release/$(BINARY)

# Default target
all: release

# Build debug version
build:
	$(CARGO) build

# Build release version
release:
	$(CARGO) build --release

# Run tests
test:
	$(CARGO) test --workspace

# Run clippy
lint:
	$(CARGO) clippy --workspace -- -D warnings

# Format code
fmt:
	$(CARGO) fmt --all

# Check formatting
fmt-check:
	$(CARGO) fmt --all -- --check

# Clean build artifacts
clean:
	$(CARGO) clean

# Install locally
install: release
	@echo "Installing lite to $(INSTALL_DIR)..."
	sudo install -Dm755 $(TARGET) $(INSTALL_DIR)/$(BINARY)
	@echo "Installing man page..."
	sudo install -Dm644 man/lite.1 $(MAN_DIR)/lite.1
	@echo "Installing bash completions..."
	-sudo install -Dm644 completions/lite.bash $(COMPLETIONS_DIR_BASH)/lite 2>/dev/null || true
	@echo "Installing fish completions..."
	-sudo install -Dm644 completions/lite.fish $(COMPLETIONS_DIR_FISH)/lite.fish 2>/dev/null || true
	@echo "Installing zsh completions..."
	-sudo install -Dm644 completions/_lite $(COMPLETIONS_DIR_ZSH)/_lite 2>/dev/null || true
	@echo "Installation complete!"

# Uninstall
uninstall:
	@echo "Uninstalling lite..."
	sudo rm -f $(INSTALL_DIR)/$(BINARY)
	sudo rm -f $(MAN_DIR)/lite.1
	sudo rm -f $(COMPLETIONS_DIR_BASH)/lite
	sudo rm -f $(COMPLETIONS_DIR_FISH)/lite.fish
	sudo rm -f $(COMPLETIONS_DIR_ZSH)/_lite
	@echo "Uninstallation complete!"

# Build .deb package
deb: release
	@echo "Building .deb package..."
	@command -v cargo-deb >/dev/null 2>&1 || { echo "Installing cargo-deb..."; $(CARGO) install cargo-deb; }
	cd lite-term && $(CARGO) deb
	@echo "Package created in target/debian/"

# Build .rpm package (requires cargo-rpm)
rpm: release
	@echo "Building .rpm package..."
	@command -v cargo-rpm >/dev/null 2>&1 || { echo "Installing cargo-rpm..."; $(CARGO) install cargo-rpm; }
	cd lite-term && $(CARGO) rpm build
	@echo "Package created in target/rpm/"

# Build AppImage (requires appimagetool)
appimage: release
	@echo "Building AppImage..."
	@mkdir -p AppDir/usr/bin
	@cp $(TARGET) AppDir/usr/bin/
	@cp scripts/lite.desktop AppDir/
	@cp scripts/lite.png AppDir/
	@appimagetool AppDir lite-x86_64.AppImage
	@rm -rf AppDir
	@echo "AppImage created!"

# Create tarball release
tarball: release
	@echo "Creating release tarball..."
	@mkdir -p dist
	@tar czf dist/lite-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz \
		-C target/release $(BINARY) \
		-C ../.. README.md LICENSE man/lite.1
	@echo "Tarball created in dist/"

# Run the editor
run: build
	$(CARGO) run

# Run with release optimizations
run-release: release
	./$(TARGET)

# Generate documentation
docs:
	$(CARGO) doc --workspace --no-deps --open

# Check for security vulnerabilities
audit:
	$(CARGO) audit

# Update dependencies
update:
	$(CARGO) update

# Show help
help:
	@echo "lite editor - Makefile targets"
	@echo ""
	@echo "Build:"
	@echo "  make build      - Build debug version"
	@echo "  make release    - Build release version"
	@echo "  make clean      - Clean build artifacts"
	@echo ""
	@echo "Test:"
	@echo "  make test       - Run all tests"
	@echo "  make lint       - Run clippy"
	@echo "  make fmt        - Format code"
	@echo "  make fmt-check  - Check formatting"
	@echo ""
	@echo "Install:"
	@echo "  make install    - Install to system"
	@echo "  make uninstall  - Remove from system"
	@echo ""
	@echo "Package:"
	@echo "  make deb        - Build .deb package"
	@echo "  make rpm        - Build .rpm package"
	@echo "  make tarball    - Create release tarball"
	@echo ""
	@echo "Run:"
	@echo "  make run        - Run debug build"
	@echo "  make run-release - Run release build"
	@echo ""
	@echo "Other:"
	@echo "  make docs       - Generate documentation"
	@echo "  make audit      - Security audit"
	@echo "  make update     - Update dependencies"
