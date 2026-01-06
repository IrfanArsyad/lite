# lite

<p align="center">
  <b>Text editor terminal yang ringan, cepat, dan modern</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/lisensi-MIT-blue.svg" alt="Lisensi">
  <img src="https://img.shields.io/badge/platform-Linux-green.svg" alt="Platform">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust">
</p>

<p align="center">
  <a href="README.md">English</a> •
  <a href="README.id.md">Bahasa Indonesia</a>
</p>

---

## Fitur

- **Keybinding ala Sublime Text** (Ctrl+P, Ctrl+D, Ctrl+Shift+K, dll)
- **Multi-cursor** editing
- **Split view** (horizontal & vertikal)
- **Tab** untuk multiple buffer
- **Syntax highlighting** (berbasis tree-sitter)
- **LSP support** untuk autocomplete
- **Integrasi Git**
- Ringan dan cepat

---

## Instalasi

### Metode 1: Quick Install (Direkomendasikan)

Jalankan satu perintah ini di terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/irfan/lite/main/scripts/install.sh | bash
```

Script ini akan otomatis:
- Mendeteksi sistem Anda
- Menginstall Rust jika belum ada
- Build dan install lite

---

### Metode 2: Install dari Source

#### Prasyarat

Install Rust terlebih dahulu:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Build & Install

```bash
# Clone repository
git clone https://github.com/IrfanArsyad/lite.git
cd lite

# Build dan install
make install
```

Atau tanpa Makefile:

```bash
cargo build --release
sudo cp target/release/lite /usr/local/bin/
```

---

### Metode 3: Install via Cargo

```bash
cargo install --git https://github.com/IrfanArsyad/lite.git lite-term
```

---

### Metode 4: Package .deb (Debian/Ubuntu)

#### Download dari Releases

```bash
# Download package terbaru
wget https://github.com/IrfanArsyad/lite/releases/latest/download/lite_0.1.0_amd64.deb

# Install
sudo dpkg -i lite_0.1.0_amd64.deb

# Atau dengan apt (mengatasi dependency)
sudo apt install ./lite_0.1.0_amd64.deb
```

#### Build Package Sendiri

```bash
# Install cargo-deb
cargo install cargo-deb

# Clone dan build
git clone https://github.com/IrfanArsyad/lite.git
cd lite
make deb

# Install package yang dihasilkan
sudo dpkg -i target/debian/lite_*.deb
```

---

### Metode 5: Arch Linux (AUR)

```bash
# Menggunakan yay
yay -S lite-editor

# Atau paru
paru -S lite-editor
```

---

### Metode 6: Fedora/RHEL (.rpm)

```bash
# Build package .rpm
cargo install cargo-rpm
make rpm

# Install
sudo rpm -i target/rpm/lite-*.rpm
```

---

## Verifikasi Instalasi

```bash
# Cek versi
lite --version

# Buka file
lite myfile.txt

# Lihat bantuan
lite --help
```

---

## Uninstall

### Jika install via make/manual:

```bash
make uninstall
```

Atau jalankan script:

```bash
curl -fsSL https://raw.githubusercontent.com/irfan/lite/main/scripts/uninstall.sh | bash
```

### Jika install via dpkg:

```bash
sudo dpkg -r lite
# atau
sudo apt remove lite
```

### Jika install via cargo:

```bash
cargo uninstall lite-term
```

---

## Penggunaan

```bash
# Buka file baru
lite

# Buka file yang ada
lite namafile.txt

# Buka beberapa file
lite file1.rs file2.rs file3.rs
```

---

## Keybindings

### Operasi File
| Shortcut | Aksi |
|----------|------|
| `Ctrl+S` | Simpan |
| `Ctrl+Shift+S` | Simpan Sebagai |
| `Ctrl+O` | Buka File |
| `Ctrl+P` | Buka Cepat |
| `Ctrl+W` | Tutup Buffer |
| `Ctrl+Q` | Keluar |

### Editing
| Shortcut | Aksi |
|----------|------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+D` | Pilih Kata / Kemunculan Berikutnya |
| `Ctrl+Shift+D` | Duplikat Baris |
| `Ctrl+Shift+K` | Hapus Baris |
| `Ctrl+/` | Toggle Komentar |
| `Ctrl+Shift+↑/↓` | Pindah Baris Atas/Bawah |

### Multi-cursor
| Shortcut | Aksi |
|----------|------|
| `Alt+Shift+↑` | Tambah Cursor di Atas |
| `Alt+Shift+↓` | Tambah Cursor di Bawah |
| `Ctrl+Shift+L` | Pisah Seleksi ke Baris |
| `Esc` | Cursor Tunggal |

### Navigasi
| Shortcut | Aksi |
|----------|------|
| `Ctrl+G` | Pergi ke Baris |
| `Ctrl+Home` | Pergi ke Awal |
| `Ctrl+End` | Pergi ke Akhir |
| `Ctrl+←/→` | Pindah per Kata |

### Pencarian
| Shortcut | Aksi |
|----------|------|
| `Ctrl+F` | Cari |
| `Ctrl+H` | Ganti |
| `F3` | Cari Berikutnya |
| `Shift+F3` | Cari Sebelumnya |

### Split & Tab
| Shortcut | Aksi |
|----------|------|
| `Ctrl+\` | Split Vertikal |
| `Ctrl+Shift+\` | Split Horizontal |
| `Ctrl+Tab` | Tab Berikutnya |
| `Ctrl+1-9` | Pindah ke Tab N |

---

## Konfigurasi

Lokasi file konfigurasi: `~/.config/lite/config.toml`

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

Gunakan sudo untuk install ke /usr/local/bin:
```bash
sudo make install
```

### Error saat build

Update Rust ke versi terbaru:
```bash
rustup update
```

### lite tidak ditemukan setelah install

Pastikan `/usr/local/bin` ada di PATH:
```bash
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

---

## Build dari Source (Development)

```bash
# Clone
git clone https://github.com/IrfanArsyad/lite.git
cd lite

# Build debug
make build

# Build release
make release

# Jalankan test
make test

# Jalankan linter
make lint

# Format kode
make fmt
```

---

## Struktur Project

```
lite/
├── lite-core/      # Primitif teks inti
├── lite-view/      # State editor & view
├── lite-ui/        # Widget UI (ratatui)
├── lite-term/      # Aplikasi utama
├── lite-config/    # Konfigurasi
├── lite-lsp/       # LSP client
└── lite-git/       # Integrasi Git
```

---

## Kontribusi

1. Fork repository
2. Buat branch (`git checkout -b fitur/fitur-baru`)
3. Commit perubahan (`git commit -m 'Tambah fitur baru'`)
4. Push ke branch (`git push origin fitur/fitur-baru`)
5. Buat Pull Request

---

## Lisensi

MIT License - lihat [LICENSE](LICENSE)

---

## Penghargaan

Terinspirasi dari [nano](https://www.nano-editor.org/), [Sublime Text](https://www.sublimetext.com/), dan [Helix](https://helix-editor.com/)

Dibangun dengan [Rust](https://www.rust-lang.org/), [ratatui](https://ratatui.rs/), dan [ropey](https://github.com/cessen/ropey)
