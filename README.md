# Modular CV TUI

**Made to show employers how much you really love rust**

A beautiful, terminal-based CV/resume application built with Rust and Ratatui. Display your resume with ASCII art, images, and customizable content - all from simple Markdown files.

![CV TUI Demo](./assets/screenshot.png)

## Features

- 🎨 **Gradient ASCII Art** - Your name displayed with a beautiful vertical color gradient
- 🖼️ **Image Support** - Portfolio image rendered via Kitty protocol with chafa
- 📝 **Markdown-Based** - All content from simple `.md` files
- 🎯 **Block-Based Layout** - Clean, organized sections
- ✨ **Styled Text** - Bullet points, labels, and custom colors
- 🖥️ **Terminal-First** - Runs in any modern terminal

## Preview

```
┌─────────────────────────────────────────────────────┐
│ ██ ▄█▀ ██  ██ ██ ▄█████ ██  ██ █████▄ ▄████▄ ██████ │
│ ████   ██  ██ ██ ▀▀▀▄▄▄ ██████ ██▄▄██ ██▄▄██ ██▄▄   │
│ ██ ▀█▄ ▀████▀ ██ █████▀ ██  ██ ██▄▄█▀ ██  ██ ██▄▄▄▄  │
├─────────────────────┬───────────┬─────────────────────┤
│ ┌─────────────────┐ │ ABOUT     │ EDUCATION           │
│ │                 │ │ Second-   │ BSc Computer        │
│ │   [Your Image]  │ │ year      │ Game Development    │
│ │                 │ │ student...│                     │
│ └─────────────────┘ ├───────────┼─────────────────────┤
│ SKILLS             │            │ CONTACTS            │
│ > Rust             │            │ Phone: 601...       │
│ > Python           │            │ Email: test@...     │
│ > Game Dev         │            │ GitHub: ...         │
├─────────────────────┴───────────┴─────────────────────┤
│ PROJECTS                                           │
│ ┌─────────────────┐ ┌─────────────────────────────┐ │
│ │ Theme Config    │ │ Self-Hosted File Server     │ │
│ │ Editor          │ │ Tailscale + Copyparty       │ │
│ └─────────────────┘ └─────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Modular CV TUI                                  │ │
│ │ Rust + Ratatui                                  │ │
│ └─────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+
- A terminal with Kitty protocol support (Kitty, WezTerm, etc.)
- libchafa (for image rendering)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/cv.git
cd cv

# Build
cargo build --release

# Run
cargo run
```

### Controls

- `q` - Quit application

## Customization

### Content Files

Edit the Markdown files in `assets/`:

| File | Purpose | Format |
|------|---------|--------|
| `NAME.md` | Your ASCII art name | Plain text (3 lines) |
| `ABOUT.md` | About section | Plain text |
| `EDUCATION.md` | Education history | Plain text |
| `SKILLS.md` | Your skills | One skill per line |
| `PROJECTS.md` | Projects | Separate with empty lines |
| `CONTACTS.md` | Contact info | `Label: value` format |
| `image.jpg` | Your photo | JPEG/PNG image |

### Example: SKILLS.md
```
Rust
Python
JavaScript
Embedded Systems
Game Development
```

### Example: CONTACTS.md
```
Phone: +1234567890
Email: your@email.com
GitHub: https://github.com/yourusername
Location: City, Country
```

### Colors

Modify the color constants in `src/main.rs`:

```rust
const BORDER_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(241, 83, 41);
const TITLE_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(243, 164, 101);
const TEXT_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(255, 255, 255);
// ... etc
```

## Project Structure

```
cv/
├── src/
│   └── main.rs          # Application code
├── assets/
│   ├── NAME.md          # ASCII art name
│   ├── ABOUT.md         # About section
│   ├── EDUCATION.md     # Education
│   ├── SKILLS.md        # Skills list
│   ├── PROJECTS.md      # Projects
│   ├── CONTACTS.md      # Contact info
│   └── image.jpg        # Portfolio image
├── Cargo.toml
└── README.md
```

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [ratatui-image](https://github.com/ratatui-org/ratatui-image) - Image widgets
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal features
- [image](https://github.com/image-rs/image) - Image processing

## Terminal Compatibility

Works best with terminals supporting Kitty graphics protocol:
- ✅ Kitty
- ✅ WezTerm
- ✅ iTerm2 (via fallback)
- ⚠️ Other terminals (uses chafa fallback)

## License

MIT License - feel free to use and modify!

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

Made with ❤️ using Rust