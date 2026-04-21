# Modular CV TUI - Technical Documentation

## Overview

A modular, terminal-based CV/resume application built with Rust and Ratatui. Features include ASCII art name display with gradient coloring, image rendering via Kitty protocol with chafa, block-based responsive layout, and customizable content via Markdown files.

## Architecture

### Project Structure

```
/home/your/Desktop/cv/
├── src/
│   └── main.rs          # Main application code
├── assets/
│   ├── NAME.md          # ASCII art name
│   ├── ABOUT.md         # About section content
│   ├── EDUCATION.md     # Education section content
│   ├── SKILLS.md        # Skills list (one per line)
│   ├── PROJECTS.md      # Projects (separated by empty lines)
│   ├── CONTACTS.md      # Contacts (label: value format)
│   └── image.jpg        # Portfolio image
├── Cargo.toml
└── README.md
```

### Module Design

The codebase is organized into logical sections:

1. **Color Theme Configuration** - All colors defined as constants at the top
2. **Helper Functions** - `lerp_color()` for gradient interpolation
3. **App Structure** - `App` struct holding all content
4. **Main Entry Point** - Terminal init, image setup, event loop
5. **UI Rendering** - Main layout orchestration
6. **Block Rendering Functions** - Individual block renderers
7. **Helper Functions for Block Creation** - Reusable block builders

## Key Features

### 1. Gradient ASCII Art Name

The NAME block displays ASCII art with a vertical gradient effect:
- Top line: `#3b817d` (teal)
- Bottom line: `#97b5b2` (light teal)
- Linear interpolation between colors

**Implementation:** `lerp_color()` function calculates RGB values based on line position.

### 2. Image Rendering (Kitty Protocol)

The portfolio image is rendered using:
- `ratatui-image` crate for image handling
- `ThreadProtocol` for non-blocking resize/encode in background thread
- Kitty protocol for terminal image display
- chafa for image-to-ANSI conversion

**Thread Architecture:**
```
Main Thread                    Worker Thread
    │                              │
    ├───── send ResizeRequest ────►│
    │                              ├── resize + encode
    │                              │
    ◄─── ResizeResponse ───────────┤
    │                              │
    └── render ────────────────────┤
```

### 3. Block-Based Layout

The UI uses a hierarchical layout system:

```
┌─────────────────────────────────────────────────────┐
│ NAME (ASCII art with gradient)                      │
├─────────────────────┬───────────┬─────────────────────┤
│ MY FACE (image)     │ ABOUT     │ EDUCATION         │
│ 25×12              │           │                   │
├─────────────────────┼───────────┤ CONTACTS         │
│ SKILLS (bullets)    │           │ (label:value)    │
├─────────────────────┴───────────┴─────────────────────┤
│ PROJECTS (sub-blocks)                              │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐    │
│ │ Project 1   │ │ Project 2   │ │ Project 3   │    │
│ └─────────────┘ └─────────────┘ └─────────────┘    │
└────────────────────────────────────────────────────┘
```

### 4. Markdown-Based Content

All content is loaded from Markdown files in `assets/`:
- **NAME.md**: ASCII art (3 lines)
- **ABOUT.md**: Plain text
- **EDUCATION.md**: Plain text
- **SKILLS.md**: One skill per line → rendered as bullet list
- **PROJECTS.md**: Projects separated by empty lines, first line = title
- **CONTACTS.md**: `Label: value` format → label styled differently

### 5. Color System

| Constant | Purpose | Color |
|----------|---------|-------|
| `BORDER_COLOR` | Main block borders | Orange #f15329 |
| `BORDER_COLOR_SUB` | Sub-block borders | Teal #3b817d |
| `TITLE_COLOR` | Main titles | Light orange #f3a465 |
| `TITLE_COLOR_SUB` | Sub-titles, labels | Light teal #97b5b2 |
| `TEXT_COLOR` | Body text | White #ffffff |
| `TEXT_COLOR_SUB` | Bullets, accents | Light teal #97b5b2 |
| `BG_COLOR` | Background | Black #000000 |
| `IMAGE_BG_COLOR` | Image area | Dark gray #141414 |

### 6. Styled Text Rendering

**Skills Block:**
- Bullet `>` in `TEXT_COLOR_SUB`
- Skill text in `TEXT_COLOR`

**Contacts Block:**
- Label (before `:`) in `TITLE_COLOR_SUB` + bold
- Value in `TEXT_COLOR`

**Projects Block:**
- Each project in a bordered sub-block
- Project title in sub-block title
- Description in body text

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.30.0 | Terminal UI framework |
| `ratatui-image` | 10.0.6 | Image rendering in terminal |
| `crossterm` | 0.29.0 | Cross-platform terminal features |
| `image` | 0.25 | Image loading/processing |
| `color-eyre` | 0.6.5 | Error handling |

## Building and Running

### Prerequisites
- Rust 1.70+
- Terminal with Kitty protocol support (Kitty, WezTerm, etc.)
- libchafa for image-to-ANSI conversion

### Build
```bash
cd /home/your/Desktop/cv
cargo build --release
```

### Run
```bash
cargo run
```

### Controls
- `q` - Quit application

## Customization

### Adding New Content

1. **Edit markdown files** in `assets/`:
   - `NAME.md` - Your ASCII art name
   - `ABOUT.md` - Your bio
   - `EDUCATION.md` - Your education history
   - `SKILLS.md` - One skill per line
   - `PROJECTS.md` - Separate projects with empty lines
   - `CONTACTS.md` - Use `Label: value` format

2. **Replace image** - Add your photo as `assets/image.jpg`

### Modifying Colors

Edit the color constants at the top of `src/main.rs`:

```rust
const BORDER_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(241, 83, 41);
// ... etc
```

### Adding New Blocks

To add a new content block:

1. Add field to `App` struct
2. Load content in `main()` from markdown file
3. Create render function (e.g., `render_new_block()`)
4. Add layout constraint and call render function in `ui()`

## Event Loop Architecture

The application uses a multi-threaded event loop:

```
┌─────────────────────────────────────────────────────┐
│                  Main Event Loop                     │
├─────────────────────────────────────────────────────┤
│  1. terminal.draw() - Render UI                     │
│  2. recv() from channel - Get events                │
│  3. Match events:                                   │
│     - KeyEvent: Handle key presses                  │
│     - ImageReady: Update image state                │
│  4. Repeat                                          │
└─────────────────────────────────────────────────────┘
         ▲                  ▲                  ▲
         │                  │                  │
    ┌────┴────┐       ┌─────┴─────┐      ┌─────┴─────┐
    │Input    │       │Image      │      │Terminal   │
    │Thread   │       │Worker     │      │Poll       │
    └─────────┘       │Thread     │      │Thread     │
                      └───────────┘      └───────────┘
```

- **Input Thread**: Polls terminal for key events
- **Image Worker Thread**: Resizes and encodes images (non-blocking)
- **Terminal Poll Thread**: Handles terminal events

## Error Handling

- Uses `color-eyre` for beautiful error reports
- Image loading errors are caught and logged
- Channel communication uses `unwrap()` with clear error messages

## Performance Considerations

- Image processing happens in background thread
- Paragraph wrapping uses `trim: true` to preserve space
- Layout constraints use `Fill(1)` for responsive design
- ASCII art gradient calculated once per frame

## Testing

To test the application:
```bash
# Development build
cargo build

# Run in terminal
cargo run

# Check for warnings
cargo clippy

# Format code
cargo fmt
```

## License

MIT License - See LICENSE file for details