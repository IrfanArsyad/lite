# lite

A terminal text editor for people who want `Ctrl+S` to save and `Ctrl+F` to find.

There are no modes to learn. Open a file, start typing, and the shortcuts you need most are listed along the bottom of the screen. The keys follow Sublime Text, so if you have used it, multi-cursor editing with `Ctrl+D` works the same way here.

<p align="center">
  <img src="docs/screenshot.svg" alt="lite editing a Rust file: four cursors selected with Ctrl+D, syntax highlighting, and the shortcut bar at the bottom" width="806">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT license">
  <img src="https://img.shields.io/badge/platform-Linux-green.svg" alt="Linux">
  <img src="https://img.shields.io/badge/written%20in-Rust-orange.svg" alt="Written in Rust">
</p>

<p align="center">
  <b>English</b> · <a href="README.id.md">Bahasa Indonesia</a>
</p>

---

## Contents

- [What you get](#what-you-get)
- [Install](#install)
- [Usage](#usage)
- [Keyboard shortcuts](#keyboard-shortcuts)
- [Configuration](#configuration)
- [Shell completion](#shell-completion)
- [Known limitations](#known-limitations)
- [Troubleshooting](#troubleshooting)
- [Uninstall](#uninstall)
- [Development](#development)

## What you get

- Save, open, find, replace, undo and quit use the keys you know from desktop editors. The bar at the bottom lists them, and it reads the real key bindings, so it never shows a shortcut that does nothing.
- `Ctrl+D` selects the word under the cursor and then adds each next match as another cursor; `Alt+Shift+↑/↓` adds a cursor on the line above or below. Typing, deleting and pasting happen at every cursor at once.
- Each file opens in its own tab. You can split the window side by side or top and bottom and show a different file in each half.
- Rust, Python, JavaScript, TypeScript, Go, C, C++, JSON, Bash, HTML, CSS and Markdown get syntax highlighting through tree-sitter. Other files open as plain text.
- Saving writes to a temporary file first and then swaps it in, so a crash never leaves a half-written file behind. Permissions and symlinks are kept. Quitting or closing a tab with unsaved changes asks you to press the key a second time.
- One `Ctrl+Z` undoes the word you just typed, not a single letter.
- Accented letters, CJK characters, emoji and tabs move, select and line up correctly.

## Install

lite runs on Linux. Building it needs `git`, a C compiler (for the tree-sitter grammars) and Rust. The quick install script sets up Rust for you when it is missing.

### Quick install

```bash
curl -fsSL https://raw.githubusercontent.com/IrfanArsyad/lite/main/scripts/install.sh | bash
```

The script uses a prebuilt binary when a release exists for your platform and builds from source otherwise. It copies `lite` to `/usr/local/bin`, installs the man page and shell completions, and asks for `sudo` to do so.

### From source

```bash
git clone https://github.com/IrfanArsyad/lite.git
cd lite
make install
```

Without `make`:

```bash
cargo build --release
sudo cp target/release/lite /usr/local/bin/
```

### With Cargo

```bash
cargo install --git https://github.com/IrfanArsyad/lite.git lite-term
```

This puts the `lite` binary in `~/.cargo/bin`.

### As a .deb package

```bash
git clone https://github.com/IrfanArsyad/lite.git
cd lite
make deb                                  # installs cargo-deb if needed
sudo apt install ./target/debian/lite_*.deb
```

Check that it worked:

```bash
lite --version
```

## Usage

```bash
lite                  # start with an empty buffer
lite notes.md         # open a file; a file that does not exist yet is created on first save
lite src/*.rs         # open several files, one tab each
lite --update         # update to the latest release
lite --help           # options and the location of your config file
```

Inside the editor, press `Ctrl+S` to save and `Ctrl+Q` to quit. Saving an untitled buffer asks for a file name.

## Keyboard shortcuts

Where a row lists two keys joined by "or", either one works.

`Ctrl+Shift` plus a letter (such as `Ctrl+Shift+S`), `Ctrl+Tab`, `Ctrl+Enter` and `Ctrl+/` only work in terminals that report them as separate keys through the kitty keyboard protocol. lite switches the protocol on by itself in kitty, foot, Ghostty, Alacritty 0.13 or newer, and WezTerm with `enable_kitty_keyboard = true`. Most other terminals, and anything running inside tmux, send `Ctrl+Shift+S` as plain `Ctrl+S`; there, use the alternatives in the tables, such as `Ctrl+Y` for redo. `Ctrl+Shift` with an arrow key works everywhere.

### Files

| Shortcut | Action |
|----------|--------|
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save as |
| `Ctrl+O` | Open file |
| `Ctrl+W` | Close tab |
| `Ctrl+Q` | Quit |

### Editing

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` or `Ctrl+Shift+Z` | Redo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copy / cut / paste (the whole line when nothing is selected) |
| `Ctrl+Shift+K` | Delete line |
| `Ctrl+Shift+D` | Duplicate line |
| `Ctrl+Shift+↑` / `Ctrl+Shift+↓` | Move line up / down |
| `Tab` / `Shift+Tab` | Indent / unindent (selected lines, or insert an indent at the cursor) |
| `Ctrl+/` | Toggle comment, using the comment style of the file's language |
| `Ctrl+Enter` / `Ctrl+Shift+Enter` | New line below / above |

### Selection and multiple cursors

| Shortcut | Action |
|----------|--------|
| `Ctrl+A` | Select all |
| `Ctrl+L` | Select line |
| `Ctrl+D` | Select word, then add the next match |
| `Ctrl+Shift+A` | Select every match |
| `Ctrl+Shift+L` | Split a selection into one cursor per line |
| `Alt+Shift+↑` / `Alt+Shift+↓` | Add a cursor above / below |
| `Esc` | Back to a single cursor |

### Moving around

| Shortcut | Action |
|----------|--------|
| `Ctrl+←` / `Ctrl+→` | Previous / next word |
| `Home` / `End` | Start / end of line |
| `Ctrl+Home` / `Ctrl+End` | Start / end of file |
| `PageUp` / `PageDown` | One screen up / down |
| `Ctrl+G` | Go to line (`42` or `42:7` for line and column) |

### Search

| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Find |
| `F3` / `Shift+F3` | Next / previous match |
| `Ctrl+E` | Search for the selected text |
| `Ctrl+H` | Replace every match in the file |

### Tabs and splits

| Shortcut | Action |
|----------|--------|
| `Ctrl+PageDown` or `Ctrl+Tab` | Next tab |
| `Ctrl+PageUp` or `Ctrl+Shift+Tab` | Previous tab |
| `Alt+1` … `Alt+9` or `Ctrl+1` … `Ctrl+9` | Go to tab 1 to 9 |
| `Alt+\` or `Ctrl+\` | Split side by side |
| `Alt+-` or `Ctrl+Shift+\` | Split top and bottom |
| `F6` / `Shift+F6` | Move to the next / previous split |
| `Ctrl+Shift+W` | Close the current split |

Most terminals never pass `Ctrl+Tab`, `Ctrl+1` … `Ctrl+9` or `Ctrl+\` to the programs running in them. The `Alt`, `F6` and `Ctrl+PageUp`/`Ctrl+PageDown` shortcuts work in every terminal.

### Mouse

Click to place the cursor or to switch splits, and scroll with the wheel. Turn this off with `mouse = false` if you prefer your terminal's own text selection.

## Configuration

lite reads `~/.config/lite/config.toml` (run `lite --help` to see the exact path on your system). The file is optional, and so is every setting in it.

```toml
[editor]
tab_width = 4            # width of a tab, and of one indent level
indent_style = "spaces"  # "spaces" or "tabs"
mouse = true             # click to place the cursor, wheel to scroll
scrolloff = 5            # lines kept visible above and below the cursor
```

If the file has a mistake, lite still starts with the default settings and shows the error in the status line.

## Shell completion

Completions for bash, zsh and fish are installed by `make install` and by the quick install script. They complete file names and the `--help`, `--version` and `--update` options.

For zsh, the file goes to `/usr/share/zsh/vendor-completions` on Debian and Ubuntu and to `/usr/share/zsh/site-functions` elsewhere; zsh reads both by default. If you manage completions yourself (for example with oh-my-zsh), copy `completions/_lite` into any directory on your `fpath`, then run:

```zsh
rm -f ~/.zcompdump; compinit
```

## Known limitations

lite is at version 0.1.0. These are the gaps today:

- No selection with `Shift` + arrow keys yet. Use `Ctrl+L`, `Ctrl+D` or the mouse.
- Language server support (autocomplete, go to definition), Git markers, quick open and the command palette are planned. Their shortcuts show "not available yet" in the status line.
- Only UTF-8 files can be opened. Binary files are refused instead of being corrupted.
- In terminals without the kitty keyboard protocol, `Ctrl+Shift` plus a letter arrives as plain `Ctrl` (see [Keyboard shortcuts](#keyboard-shortcuts)), and `Ctrl+/` arrives as `Ctrl+7`, which switches to tab 7 instead of toggling a comment.
- Tested on Linux only.

## Troubleshooting

**`lite: command not found` after installing.** `/usr/local/bin` is not on your `PATH`. Add it to the startup file of your shell:

```bash
# zsh
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc

# bash
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc
```

If you installed with Cargo, the binary is in `~/.cargo/bin`; `source ~/.cargo/env` adds it for the current shell.

**`cargo: command not found` when building.** Install Rust with rustup, then open a new terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**`linker 'cc' not found` or a tree-sitter build error.** The grammars are written in C. On Debian and Ubuntu, `sudo apt install build-essential`; on Fedora, `sudo dnf install gcc`; on Arch, `sudo pacman -S base-devel`.

**A shortcut does nothing.** Your terminal, tmux or desktop may be catching the key before lite sees it. Try the alternative listed in [Tabs and splits](#tabs-and-splits), or remove the conflicting binding from the terminal's settings.

## Uninstall

| Installed with | Remove with |
|----------------|-------------|
| `make install` | `make uninstall` (from the cloned repository) |
| Quick install script | `curl -fsSL https://raw.githubusercontent.com/IrfanArsyad/lite/main/scripts/uninstall.sh \| bash` |
| Cargo | `cargo uninstall lite-term` |
| .deb package | `sudo apt remove lite` |

Your config file in `~/.config/lite` is left in place; delete it by hand if you no longer want it.

## Development

```bash
git clone https://github.com/IrfanArsyad/lite.git
cd lite
make build     # debug build
make run       # build and start lite
make test      # run the test suite
make lint      # clippy
make fmt       # rustfmt
```

The code is a Cargo workspace:

```
lite/
├── lite-core/     text primitives: rope helpers, selections, transactions, display width
├── lite-view/     editor state: documents, undo history, views, split layout, syntax trees
├── lite-ui/       ratatui widgets: editor view, tab line, status line, help bar, prompts
├── lite-term/     the lite binary: event loop, key handling, editing commands
├── lite-config/   settings, key bindings, theme
├── lite-lsp/      placeholder for language server support
└── lite-git/      placeholder for Git integration
```

### Contributing

Bug reports and pull requests are welcome.

1. Fork the repository and create a branch (`git checkout -b fix/short-description`).
2. Make your change and add a test for it.
3. Run `make test` and `make lint` before you push.
4. Open a pull request that explains what changed and why.

## License

MIT. See [LICENSE](LICENSE).
