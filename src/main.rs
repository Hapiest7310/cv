//! Modular CV TUI Application
//! A terminal-based CV/resume viewer built with Rust and Ratatui
//!
//! Features:
//! - ASCII art name with gradient coloring
//! - Image rendering via Kitty protocol with chafa
//! - Block-based responsive layout
//! - Customizable content via Markdown files

use std::{
    fs,
    sync::mpsc::{self},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use ratatui_image::{
    errors::Errors, picker::Picker, thread::ThreadProtocol, Resize, StatefulImage,
};

// =============================================================================
// COLOR THEME CONFIGURATION
// =============================================================================

/// Border color - used for main block borders (orange)
const BORDER_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(241, 83, 41);

/// Sub border color - used for nested/subsection borders (teal)
const BORDER_COLOR_SUB: ratatui::style::Color = ratatui::style::Color::Rgb(59, 129, 125);

/// Title color - used for main block titles (light orange)
const TITLE_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(243, 164, 101);

/// Sub title color - used for sub-block titles and labels (light teal)
const TITLE_COLOR_SUB: ratatui::style::Color = ratatui::style::Color::Rgb(151, 181, 178);

/// Primary text color - main content text (white)
const TEXT_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(255, 255, 255);

/// Secondary text color - bullets and accents (light teal)
const TEXT_COLOR_SUB: ratatui::style::Color = ratatui::style::Color::Rgb(151, 181, 178);

/// Background color - main background (black)
const BG_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(0, 0, 0);

/// Image background - slightly lighter for image display area
const IMAGE_BG_COLOR: ratatui::style::Color = ratatui::style::Color::Rgb(20, 20, 20);

// Gradient colors for ASCII art name (top to bottom)
const GRADIENT_TOP: (u8, u8, u8) = (59, 129, 125); // #3b817d
const GRADIENT_BOTTOM: (u8, u8, u8) = (151, 181, 178); // #97b5b2

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Linearly interpolates between two RGB colors
/// Used for creating gradient effects on ASCII art
fn lerp_color(top: (u8, u8, u8), bottom: (u8, u8, u8), t: f64) -> ratatui::style::Color {
    let r = (top.0 as f64 + (bottom.0 as f64 - top.0 as f64) * t) as u8;
    let g = (top.1 as f64 + (bottom.1 as f64 - top.1 as f64) * t) as u8;
    let b = (top.2 as f64 + (bottom.2 as f64 - top.2 as f64) * t) as u8;
    ratatui::style::Color::Rgb(r, g, b)
}

// =============================================================================
// APP STRUCTURE
// =============================================================================

/// Main application state holding all content for the CV
struct App {
    image_state: ThreadProtocol,
    last_known_size: Rect,
    name: String,
    about_text: String,
    education_text: String,
    skills_text: String,
    projects_text: String,
    contacts_text: String,
}

/// Event types for the application event loop
enum AppEvent {
    KeyEvent(KeyEvent),
    ImageReady(Result<ratatui_image::thread::ResizeResponse, Errors>),
}

// =============================================================================
// MAIN ENTRY POINT
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut terminal = ratatui::init();

    // Setup image picker and load portfolio image
    let picker = Picker::from_query_stdio()?;
    let dyn_img = image::ImageReader::open("./assets/image.jpg")?.decode()?;

    // Create channel for image resize requests (background thread communication)
    type ImageResizeRequest = ratatui_image::thread::ResizeRequest;
    let (tx_worker, rec_worker) = mpsc::channel::<ImageResizeRequest>();
    let (tx_main, rec_main) = mpsc::channel();

    // Spawn background thread for image resize/encoding (non-blocking)
    let tx_main_render = tx_main.clone();
    thread::spawn(move || loop {
        if let Ok(request) = rec_worker.recv() {
            tx_main_render
                .send(AppEvent::ImageReady(request.resize_encode()))
                .unwrap();
        }
    });

    // Spawn background thread for terminal event polling
    let tx_main_events = tx_main.clone();
    thread::spawn(move || -> Result<(), std::io::Error> {
        loop {
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    tx_main_events.send(AppEvent::KeyEvent(key)).unwrap();
                }
            }
        }
    });

    // Load all content from markdown files in assets folder
    let name = fs::read_to_string("./assets/NAME.md")?;
    let about_text = fs::read_to_string("./assets/ABOUT.md")?;
    let education_text = fs::read_to_string("./assets/EDUCATION.md")?;
    let skills_text = fs::read_to_string("./assets/SKILLS.md")?;
    let projects_text = fs::read_to_string("./assets/PROJECTS.md")?;
    let contacts_text = fs::read_to_string("./assets/CONTACTS.md")?;

    // Initialize app state
    let mut app = App {
        image_state: ThreadProtocol::new(tx_worker, Some(picker.new_resize_protocol(dyn_img))),
        last_known_size: Rect::default(),
        name,
        about_text,
        education_text,
        skills_text,
        projects_text,
        contacts_text,
    };

    // Main event loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Ok(ev) = rec_main.try_recv() {
            match ev {
                AppEvent::KeyEvent(key) => {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
                AppEvent::ImageReady(completed) => {
                    let _ = app.image_state.update_resized_protocol(completed?);
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

// =============================================================================
// UI RENDERING
// =============================================================================

/// Main UI rendering function - draws all CV blocks
fn ui(f: &mut Frame<'_>, app: &mut App) {
    let area = f.area();

    // =========================================================================
    // LAYOUT DEFINITION
    // =========================================================================
    // Main layout: vertical split into NAME, middle content, and PROJECTS
    let main_vertical = Layout::vertical([
        Constraint::Length(7), // NAME section (7 rows)
        Constraint::Fill(1),   // Middle section (image, skills, about, education, contacts)
        Constraint::Fill(1),   // PROJECTS section
    ]);

    let [name_area, middle_area, bottom_area] = area.layout(&main_vertical);

    // =========================================================================
    // NAME BLOCK - ASCII art with gradient coloring
    // =========================================================================
    render_name_block(f, name_area, &app.name);

    // =========================================================================
    // MIDDLE SECTION - Split into left (image+skills) and right (about+education+contacts)
    // =========================================================================
    let middle_horizontal = Layout::horizontal([
        Constraint::Length(25), // Left: image + skills (25 columns)
        Constraint::Fill(1),    // Right: about, education, contacts
    ]);

    let [left_middle, right_middle] = middle_area.layout(&middle_horizontal);

    // Right side: vertical split for ABOUT, EDUCATION, CONTACTS
    let right_vertical = Layout::vertical([
        Constraint::Fill(6),   // ABOUT
        Constraint::Length(4),   // EDUCATION
        Constraint::Length(9), // CONTACTS (fixed height)
    ]);

    let [about_area, education_area, contacts_area] = right_middle.layout(&right_vertical);

    // Left side: vertical split for IMAGE and SKILLS
    let left_vertical = Layout::vertical([
        Constraint::Length(12), // IMAGE (12 rows)
        Constraint::Fill(1),    // SKILLS (remaining)
    ]);

    let [image_area, skills_area] = left_middle.layout(&left_vertical);

    // =========================================================================
    // LEFT COLUMN - IMAGE and SKILLS
    // =========================================================================

    // Render portfolio image using Kitty protocol
    render_image_block(f, image_area, app);

    // Render skills list with bullet points
    render_skills_block(f, skills_area, &app.skills_text);

    // =========================================================================
    // RIGHT COLUMN - ABOUT, EDUCATION, CONTACTS
    // =========================================================================

    render_about_block(f, about_area, &app.about_text);
    render_education_block(f, education_area, &app.education_text);
    render_contacts_block(f, contacts_area, &app.contacts_text);

    // =========================================================================
    // PROJECTS SECTION
    // =========================================================================
    render_projects_block(f, bottom_area, &app.projects_text);
}

// =============================================================================
// BLOCK RENDERING FUNCTIONS
// =============================================================================

/// Renders the NAME block with gradient-colored ASCII art
fn render_name_block(f: &mut Frame<'_>, area: Rect, name: &str) {
    let name_block = create_block(" NAME ", BORDER_COLOR, TITLE_COLOR);
    let inner = name_block.inner(area);
    f.render_widget(&name_block, area);

    // Render each line with gradient color (top to bottom)
    let ascii_lines: Vec<&str> = name.lines().collect();
    let ascii_count = ascii_lines.len();

    for (i, line) in ascii_lines.iter().enumerate() {
        if i >= inner.height as usize {
            break;
        }

        // Calculate gradient position (0.0 = top, 1.0 = bottom)
        let t = if ascii_count > 1 {
            i as f64 / (ascii_count - 1) as f64
        } else {
            0.0
        };

        let gradient_color = lerp_color(GRADIENT_TOP, GRADIENT_BOTTOM, t);
        let paragraph = Paragraph::new(*line).style(Style::new().fg(gradient_color));

        f.render_widget(
            paragraph,
            Rect::new(inner.x, inner.y + i as u16, inner.width, 1),
        );
    }
}

/// Renders the portfolio image using Kitty protocol
fn render_image_block(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let image_block = create_block(" MY FACE ", BORDER_COLOR, TITLE_COLOR).bg(IMAGE_BG_COLOR);

    let inner = image_block.inner(area);
    f.render_widget(&image_block, area);

    let size_for = app.image_state.size_for(Resize::Fit(None), inner);
    if let Some(size) = size_for {
        app.last_known_size = size;
        f.render_widget(Clear, inner);
        f.render_widget(Block::new().bg(IMAGE_BG_COLOR), inner);
        f.render_stateful_widget(StatefulImage::new(), inner, &mut app.image_state);
    }
}

/// Renders the SKILLS block with bullet-point list
fn render_skills_block(f: &mut Frame<'_>, area: Rect, skills: &str) {
    let skills_block = create_block(" SKILLS ", BORDER_COLOR, TITLE_COLOR);
    let inner = skills_block.inner(area);
    f.render_widget(&skills_block, area);

    let skill_gap: u16 = 1;
    let line_height: u16 = 1;

    let skills_lines: Vec<&str> = skills
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    for (i, skill) in skills_lines.iter().enumerate() {
        let y_pos = inner.y + (i as u16 * (line_height + skill_gap));
        if y_pos >= inner.y + inner.height {
            break;
        }

        let line_rect = Rect::new(inner.x, y_pos, inner.width, line_height);
        let bullet = Span::styled("> ", Style::new().fg(TEXT_COLOR_SUB));
        let skill_text = Span::styled(skill.trim(), Style::new().fg(TEXT_COLOR));

        let paragraph = Paragraph::new(Line::default().spans(vec![bullet, skill_text]));
        f.render_widget(paragraph, line_rect);
    }
}

/// Renders the ABOUT block
fn render_about_block(f: &mut Frame<'_>, area: Rect, text: &str) {
    let about_block = create_block(" ABOUT ", BORDER_COLOR, TITLE_COLOR);
    let inner = about_block.inner(area);
    f.render_widget(&about_block, area);

    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(TEXT_COLOR))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(paragraph, inner);
}

/// Renders the EDUCATION block
fn render_education_block(f: &mut Frame<'_>, area: Rect, text: &str) {
    let education_block = create_block(" EDUCATION ", BORDER_COLOR, TITLE_COLOR);
    let inner = education_block.inner(area);
    f.render_widget(&education_block, area);

    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(TEXT_COLOR))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(paragraph, inner);
}

/// Renders the CONTACTS block with label:value formatting
/// Labels (before ":") are rendered in TITLE_COLOR_SUB + bold
fn render_contacts_block(f: &mut Frame<'_>, area: Rect, text: &str) {
    let contacts_block = create_block(" CONTACTS ", BORDER_COLOR, TITLE_COLOR);
    let inner = contacts_block.inner(area);
    f.render_widget(&contacts_block, area);

    let contacts_gap: u16 = 1;
    let line_height: u16 = 1;

    let contacts_lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    for (i, line) in contacts_lines.iter().enumerate() {
        let y_pos = inner.y + (i as u16 * (line_height + contacts_gap));
        if y_pos >= inner.y + inner.height {
            break;
        }

        let line_rect = Rect::new(inner.x, y_pos, inner.width, line_height);

        // Check if line contains "label: value" format
        if let Some((label, value)) = line.split_once(':') {
            let label_span = Span::styled(
                format!("{}: ", label),
                Style::new()
                    .fg(TITLE_COLOR_SUB)
                    .add_modifier(Modifier::BOLD),
            );
            let value_span = Span::styled(value.trim(), Style::new().fg(TEXT_COLOR));

            let paragraph = Paragraph::new(Line::default().spans(vec![label_span, value_span]));
            f.render_widget(paragraph, line_rect);
        } else {
            let paragraph = Paragraph::new(*line).style(Style::new().fg(TEXT_COLOR));
            f.render_widget(paragraph, line_rect);
        }
    }
}

/// Renders the PROJECTS block with sub-blocks for each project
fn render_projects_block(f: &mut Frame<'_>, area: Rect, text: &str) {
    let projects_block = create_block(" PROJECTS ", BORDER_COLOR, TITLE_COLOR);
    let inner = projects_block.inner(area);
    f.render_widget(&projects_block, area);

    // Split by empty lines to get individual projects
    let project_descriptions: Vec<&str> = text.trim().split("\n\n").collect();
    let project_count = project_descriptions.len().max(1);
    let project_height = inner.height as usize / project_count;

    // Create vertical layout for sub-blocks
    let project_layout = Layout::vertical(
        project_descriptions
            .iter()
            .map(|_| Constraint::Length(project_height as u16))
            .collect::<Vec<_>>(),
    );
    let project_areas = project_layout.split(inner);

    for (idx, project_text) in project_descriptions.iter().enumerate() {
        let lines: Vec<&str> = project_text.lines().collect();
        if lines.is_empty() {
            continue;
        }

        // First line is the project title, rest is content
        let title = lines.first().unwrap().trim();
        let content = lines.get(1..).map(|l| l.join("\n")).unwrap_or_default();

        let project_area = project_areas[idx];

        // Create sub-block with different border color
        let sub_block =
            create_sub_block(&format!(" {} ", title), BORDER_COLOR_SUB, TITLE_COLOR_SUB);
        let sub_inner = sub_block.inner(project_area);
        f.render_widget(&sub_block, project_area);

        let paragraph = Paragraph::new(content)
            .style(Style::new().fg(TEXT_COLOR))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(paragraph, sub_inner);
    }
}

// =============================================================================
// HELPER FUNCTIONS FOR BLOCK CREATION
// =============================================================================

/// Creates a standard block with given title, border color, and title color
fn create_block(
    title: &str,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(title.to_string())
        .bg(BG_COLOR)
        .fg(TEXT_COLOR)
        .border_style(Style::new().fg(border_color))
        .title_style(Style::new().fg(title_color).add_modifier(Modifier::BOLD))
}

/// Creates a sub-block for nested content (like project items)
fn create_sub_block(
    title: &str,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) -> Block<'static> {
    Block::default()
        .borders(Borders::TOP)
        .title(title.to_string())
        .bg(BG_COLOR)
        .fg(TEXT_COLOR)
        .border_style(Style::new().fg(border_color))
        .title_style(Style::new().fg(title_color).add_modifier(Modifier::BOLD))
}
