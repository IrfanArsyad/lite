# lite

<p align="center">
  <b>A lightweight, fast, and modern terminal text editor</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/platform-Linux-green.svg" alt="Platform">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust">
</p>

<p align="center">
  <a href="README.md">English</a> •
  <a href="README.id.md">Bahasa Indonesia</a>
</p>

---

## Features

- **Sublime-style keybindings** (Ctrl+P, Ctrl+D, Ctrl+Shift+K, etc.)
- **Multi-cursor** editing
- **Split views** (horizontal & vertical)
- **Tabs** for multiple buffers
- **Syntax highlighting** (tree-sitter based)
- **LSP support** for autocompletion
- **Git integration**
- Lightweight and fast

---

## Installation

### Method 1: Quick Install (Recommended)

Run this single command in your terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/IrfanArsyad/lite/main/scripts/install.sh | bash
```

This script will automatically:
- Detect your system
- Install Rust if not present
- Build and install lite

---

### Method 2: Install from Source

#### Prerequisites

Install Rust first:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Build & Install

```bash
# Clone repository
git clone https://github.com/IrfanArsyad/lite.git
cd lite

# Build and install
make install
```

Or without Makefile:

```bash
cargo build --release
sudo cp target/release/lite /usr/local/bin/
```

---

### Method 3: Install via Cargo

```bash
cargo install --git https://github.com/IrfanArsyad/lite.git lite-term
```

---

### Method 4: Debian/Ubuntu (.deb package)

#### Download from Releases

```bash
# Download latest package
wget https://github.com/IrfanArsyad/lite/releases/latest/download/lite_0.1.0_amd64.deb

# Install
sudo dpkg -i lite_0.1.0_amd64.deb

# Or with apt (handles dependencies)
sudo apt install ./lite_0.1.0_amd64.deb
```

#### Build Package Yourself

```bash
# Install cargo-deb
cargo install cargo-deb

# Clone and build
git clone https://github.com/IrfanArsyad/lite.git
cd lite
make deb

# Install the generated package
sudo dpkg -i target/debian/lite_*.deb
```

---

### Method 5: Arch Linux (AUR)

```bash
# Using yay
yay -S lite-editor

# Or paru
paru -S lite-editor
```

---

### Method 6: Fedora/RHEL (.rpm)

```bash
# Build .rpm package
cargo install cargo-rpm
make rpm

# Install
sudo rpm -i target/rpm/lite-*.rpm
```

---

## Verify Installation

```bash
# Check version
lite --version

# Open a file
lite myfile.txt

# Show help
lite --help
```

---

## Uninstall

### If installed via make/manual:

```bash
make uninstall
```

Or run the script:

```bash
curl -fsSL https://raw.githubusercontent.com/irfan/lite/main/scripts/uninstall.sh | bash
```

### If installed via dpkg:

```bash
sudo dpkg -r lite
# or
sudo apt remove lite
```

### If installed via cargo:

```bash
cargo uninstall lite-term
```

---

## Usage

```bash
# Open new file
lite

# Open existing file
lite filename.txt

# Open multiple files
lite file1.rs file2.rs file3.rs
```

---

## Keybindings

### File Operations
| Shortcut | Action |
|----------|--------|
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+O` | Open File |
| `Ctrl+P` | Quick Open |
| `Ctrl+W` | Close Buffer |
| `Ctrl+Q` | Quit |

### Editing
| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+D` | Select Word / Next Occurrence |
| `Ctrl+Shift+D` | Duplicate Line |
| `Ctrl+Shift+K` | Delete Line |
| `Ctrl+/` | Toggle Comment |
| `Ctrl+Shift+↑/↓` | Move Line Up/Down |

### Multi-cursor
| Shortcut | Action |
|----------|--------|
| `Alt+Shift+↑` | Add Cursor Above |
| `Alt+Shift+↓` | Add Cursor Below |
| `Ctrl+Shift+L` | Split Selection into Lines |
| `Esc` | Single Cursor |

### Navigation
| Shortcut | Action |
|----------|--------|
| `Ctrl+G` | Go to Line |
| `Ctrl+Home` | Go to Start |
| `Ctrl+End` | Go to End |
| `Ctrl+←/→` | Move by Word |

### Search
| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Find |
| `Ctrl+H` | Replace |
| `F3` | Find Next |
| `Shift+F3` | Find Previous |

### Splits & Tabs
| Shortcut | Action |
|----------|--------|
| `Ctrl+\` | Split Vertical |
| `Ctrl+Shift+\` | Split Horizontal |
| `Ctrl+Tab` | Next Tab |
| `Ctrl+1-9` | Switch to Tab N |

---

## Configuration

Config file location: `~/.config/lite/config.toml`

```toml
[editor]
tab_width = 4
indent_style = "spaces"
line_numbers = true
mouse = true
scrolloff = 5
auto_save = false

[theme]
name = "default"
```

---

## Troubleshooting

### Error: "cargo: command not found"

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Error: "permission denied"

Use sudo to install to /usr/local/bin:
```bash
sudo make install
```

### Build errors

Update Rust to latest version:
```bash
rustup update
```

### lite not found after install

Make sure `/usr/local/bin` is in your PATH:
```bash
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## Building from Source (Development)

```bash
# Clone
git clone https://github.com/IrfanArsyad/lite.git
cd lite

# Debug build
make build

# Release build
make release

# Run tests
make test

# Run linter
make lint

# Format code
make fmt
```

---

## Project Structure

```
lite/
├── lite-core/      # Core text primitives
├── lite-view/      # Editor state & view
├── lite-ui/        # UI widgets (ratatui)
├── lite-term/      # Main application
├── lite-config/    # Configuration
├── lite-lsp/       # LSP client
└── lite-git/       # Git integration
```

---

## Contributing

1. Fork the repository
2. Create your branch (`git checkout -b feature/new-feature`)
3. Commit your changes (`git commit -m 'Add new feature'`)
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

---

## License

MIT License - see [LICENSE](LICENSE)

---

## Acknowledgments

Inspired by [nano](https://www.nano-editor.org/), [Sublime Text](https://www.sublimetext.com/), and [Helix](https://helix-editor.com/)

Built with [Rust](https://www.rust-lang.org/), [ratatui](https://ratatui.rs/), and [ropey](https://github.com/cessen/ropey)
