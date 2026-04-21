//! Modular CV TUI Application
//! A terminal-based CV/resume viewer built with Rust and Ratatui

use std::{
    fs,
    sync::mpsc::{self},
    thread,
    time::Duration,
};

use config::Config;
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
use serde::Deserialize;

// =============================================================================
// CONFIGURATION
// =============================================================================

#[derive(Debug, Deserialize)]
struct LayoutConfig {
    name_height: u16,
    middle_height: u16,
    projects_height: u16,
    image_width: u16,
    right_width: u16,
    image_height: u16,
    skills_height: u16,
    about_height: u16,
    education_height: u16,
    contacts_height: u16,
    skills_gap: u16,
    contacts_gap: u16,
}

#[derive(Debug, Deserialize)]
struct ColorsConfig {
    border_r: u8,
    border_g: u8,
    border_b: u8,
    border_sub_r: u8,
    border_sub_g: u8,
    border_sub_b: u8,
    title_r: u8,
    title_g: u8,
    title_b: u8,
    title_sub_r: u8,
    title_sub_g: u8,
    title_sub_b: u8,
    text_r: u8,
    text_g: u8,
    text_b: u8,
    text_sub_r: u8,
    text_sub_g: u8,
    text_sub_b: u8,
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
    image_bg_r: u8,
    image_bg_g: u8,
    image_bg_b: u8,
}

#[derive(Debug, Deserialize)]
struct GradientConfig {
    top_r: u8,
    top_g: u8,
    top_b: u8,
    bottom_r: u8,
    bottom_g: u8,
    bottom_b: u8,
}

#[derive(Debug, Deserialize)]
struct ProjectsConfig {
    project_1_height: u16,
    project_2_height: u16,
    project_3_height: u16,
    project_4_height: u16,
    project_5_height: u16,
    project_6_height: u16,
    project_7_height: u16,
    project_8_height: u16,
    project_9_height: u16,
    project_10_height: u16,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    layout: LayoutConfig,
    colors: ColorsConfig,
    gradient: GradientConfig,
    projects: ProjectsConfig,
}

impl AppConfig {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?;
        Ok(AppConfig {
            layout: config.get("layout")?,
            colors: config.get("colors")?,
            gradient: config.get("gradient")?,
            projects: config.get("projects")?,
        })
    }
}

// Color getters
fn get_color(cfg: &ColorsConfig) -> (u8, u8, u8) {
    (cfg.text_r, cfg.text_g, cfg.text_b)
}
fn get_bg_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.bg_r, cfg.bg_g, cfg.bg_b)
}
fn get_image_bg_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.image_bg_r, cfg.image_bg_g, cfg.image_bg_b)
}
fn get_border_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.border_r, cfg.border_g, cfg.border_b)
}
fn get_border_sub_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.border_sub_r, cfg.border_sub_g, cfg.border_sub_b)
}
fn get_title_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.title_r, cfg.title_g, cfg.title_b)
}
fn get_title_sub_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.title_sub_r, cfg.title_sub_g, cfg.title_sub_b)
}
fn get_text_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.text_r, cfg.text_g, cfg.text_b)
}
fn get_text_sub_color(cfg: &ColorsConfig) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(cfg.text_sub_r, cfg.text_sub_g, cfg.text_sub_b)
}
fn get_gradient_top(cfg: &GradientConfig) -> (u8, u8, u8) {
    (cfg.top_r, cfg.top_g, cfg.top_b)
}
fn get_gradient_bottom(cfg: &GradientConfig) -> (u8, u8, u8) {
    (cfg.bottom_r, cfg.bottom_g, cfg.bottom_b)
}

fn lerp_color(top: (u8, u8, u8), bottom: (u8, u8, u8), t: f64) -> ratatui::style::Color {
    let r = (top.0 as f64 + (bottom.0 as f64 - top.0 as f64) * t) as u8;
    let g = (top.1 as f64 + (bottom.1 as f64 - top.1 as f64) * t) as u8;
    let b = (top.2 as f64 + (bottom.2 as f64 - top.2 as f64) * t) as u8;
    ratatui::style::Color::Rgb(r, g, b)
}

// =============================================================================
// APP STRUCTURE
// =============================================================================

struct App {
    image_state: ThreadProtocol,
    last_known_size: Rect,
    name: String,
    about_text: String,
    education_text: String,
    skills_text: String,
    projects_text: Vec<String>,
    contacts_text: String,
}

enum AppEvent {
    KeyEvent(KeyEvent),
    ImageReady(Result<ratatui_image::thread::ResizeResponse, Errors>),
}

// =============================================================================
// MAIN
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let mut terminal = ratatui::init();

    let picker = Picker::from_query_stdio()?;
    let dyn_img = image::ImageReader::open("./assets/image.jpg")?.decode()?;

    type ImageResizeRequest = ratatui_image::thread::ResizeRequest;
    let (tx_worker, rec_worker) = mpsc::channel::<ImageResizeRequest>();
    let (tx_main, rec_main) = mpsc::channel();

    let tx_main_render = tx_main.clone();
    thread::spawn(move || loop {
        if let Ok(request) = rec_worker.recv() {
            tx_main_render
                .send(AppEvent::ImageReady(request.resize_encode()))
                .unwrap();
        }
    });

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

    let name = fs::read_to_string("./assets/NAME.md")?;
    let about_text = fs::read_to_string("./assets/ABOUT.md")?;
    let education_text = fs::read_to_string("./assets/EDUCATION.md")?;
    let skills_text = fs::read_to_string("./assets/SKILLS.md")?;
    let contacts_text = fs::read_to_string("./assets/CONTACTS.md")?;

    let mut projects_text: Vec<String> = Vec::new();
    for i in 1..=10 {
        let path = format!("./assets/PROJECT_{}.md", i);
        if let Ok(content) = fs::read_to_string(&path) {
            if !content.trim().is_empty() {
                projects_text.push(content);
            }
        }
    }

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

    loop {
        terminal.draw(|f| ui(f, &mut app, &config))?;
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
// UI
// =============================================================================

fn ui(f: &mut Frame<'_>, app: &mut App, config: &AppConfig) {
    let area = f.area();
    let cfg = &config.colors;

    let bg_color = get_bg_color(cfg);
    let image_bg_color = get_image_bg_color(cfg);
    let border_color = get_border_color(cfg);
    let title_color = get_title_color(cfg);
    let gradient_top = get_gradient_top(&config.gradient);
    let gradient_bottom = get_gradient_bottom(&config.gradient);

    // Main layout
    let name_h = if config.layout.name_height > 0 {
        Constraint::Length(config.layout.name_height)
    } else {
        Constraint::Length(7)
    };
    let middle_h = if config.layout.middle_height > 0 {
        Constraint::Length(config.layout.middle_height)
    } else {
        Constraint::Fill(1)
    };
    let projects_h = if config.layout.projects_height > 0 {
        Constraint::Length(config.layout.projects_height)
    } else {
        Constraint::Fill(1)
    };

    let main_vertical = Layout::vertical([name_h, middle_h, projects_h]);
    let [name_area, middle_area, projects_area] = area.layout(&main_vertical);

    render_name_block(
        f,
        name_area,
        &app.name,
        gradient_top,
        gradient_bottom,
        bg_color,
        border_color,
        title_color,
    );

    // Middle horizontal
    let left_w = if config.layout.image_width > 0 {
        Constraint::Length(config.layout.image_width)
    } else {
        Constraint::Length(25)
    };
    let right_w = if config.layout.right_width > 0 {
        Constraint::Length(config.layout.right_width)
    } else {
        Constraint::Fill(1)
    };
    let middle_horizontal = Layout::horizontal([left_w, right_w]);
    let [left_middle, right_middle] = middle_area.layout(&middle_horizontal);

    // Right vertical (about, education, contacts)
    let about_h = if config.layout.about_height > 0 {
        Constraint::Length(config.layout.about_height)
    } else {
        Constraint::Fill(1)
    };
    let edu_h = if config.layout.education_height > 0 {
        Constraint::Length(config.layout.education_height)
    } else {
        Constraint::Fill(1)
    };
    let contacts_h = Constraint::Length(config.layout.contacts_height.max(1));
    let right_vertical = Layout::vertical([about_h, edu_h, contacts_h]);
    let [about_area, education_area, contacts_area] = right_middle.layout(&right_vertical);

    // Left vertical (image, skills)
    let img_h = Constraint::Length(config.layout.image_height.max(1));
    let skills_h = if config.layout.skills_height > 0 {
        Constraint::Length(config.layout.skills_height)
    } else {
        Constraint::Fill(1)
    };
    let left_vertical = Layout::vertical([img_h, skills_h]);
    let [image_area, skills_area] = left_middle.layout(&left_vertical);

    render_image_block(
        f,
        image_area,
        app,
        bg_color,
        border_color,
        title_color,
        image_bg_color,
    );
    render_skills_block(
        f,
        skills_area,
        &app.skills_text,
        config.layout.skills_gap,
        cfg,
        bg_color,
        border_color,
        title_color,
    );
    render_about_block(
        f,
        about_area,
        &app.about_text,
        cfg,
        bg_color,
        border_color,
        title_color,
    );
    render_education_block(
        f,
        education_area,
        &app.education_text,
        cfg,
        bg_color,
        border_color,
        title_color,
    );
    render_contacts_block(
        f,
        contacts_area,
        &app.contacts_text,
        config.layout.contacts_gap,
        cfg,
        bg_color,
        border_color,
        title_color,
    );
    render_projects_block(
        f,
        projects_area,
        &app.projects_text,
        &config.projects,
        cfg,
        bg_color,
        border_color,
        title_color,
    );
}

// =============================================================================
// BLOCK RENDERERS
// =============================================================================

fn render_name_block(
    f: &mut Frame<'_>,
    area: Rect,
    name: &str,
    gradient_top: (u8, u8, u8),
    gradient_bottom: (u8, u8, u8),
    bg_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) {
    let name_block = create_block(" NAME ", border_color, title_color).bg(bg_color);
    let inner = name_block.inner(area);
    f.render_widget(&name_block, area);

    let ascii_lines: Vec<&str> = name.lines().collect();
    let ascii_count = ascii_lines.len();

    for (i, line) in ascii_lines.iter().enumerate() {
        if i >= inner.height as usize {
            break;
        }
        let t = if ascii_count > 1 {
            i as f64 / (ascii_count - 1) as f64
        } else {
            0.0
        };
        let gradient_color = lerp_color(gradient_top, gradient_bottom, t);
        let paragraph = Paragraph::new(*line).style(Style::new().fg(gradient_color));
        f.render_widget(
            paragraph,
            Rect::new(inner.x, inner.y + i as u16, inner.width, 1),
        );
    }
}

fn render_image_block(
    f: &mut Frame<'_>,
    area: Rect,
    app: &mut App,
    bg_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
    image_bg_color: ratatui::style::Color,
) {
    let image_block = create_block(" MY FACE ", border_color, title_color).bg(image_bg_color);
    let inner = image_block.inner(area);
    f.render_widget(&image_block, area);

    let size_for = app.image_state.size_for(Resize::Fit(None), inner);
    if let Some(size) = size_for {
        app.last_known_size = size;
        f.render_widget(Clear, inner);
        f.render_widget(Block::new().bg(image_bg_color), inner);
        f.render_stateful_widget(StatefulImage::new(), inner, &mut app.image_state);
    }
}

fn render_skills_block(
    f: &mut Frame<'_>,
    area: Rect,
    skills: &str,
    skills_gap: u16,
    cfg: &ColorsConfig,
    bg_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) {
    let skills_block = create_block(" SKILLS ", border_color, title_color).bg(bg_color);
    let inner = skills_block.inner(area);
    f.render_widget(&skills_block, area);

    let text_color = get_text_color(cfg);
    let title_sub_color = get_title_sub_color(cfg);

    let skills_list: Vec<&str> = skills.lines().filter(|l| !l.trim().is_empty()).collect();
    let mut y_pos = inner.y;

    for skill in skills_list {
        if y_pos >= inner.y + inner.height {
            break;
        }

        // Parse: "Main text (subtext)" or just "Main text"
        let (main_text, sub_text) = if let Some((main, sub)) = skill.trim().split_once('(') {
            (main.trim(), Some(sub.trim().trim_end_matches(')')))
        } else {
            (skill.trim(), None)
        };

        // Render main line
        let line_rect = Rect::new(inner.x, y_pos, inner.width, 1);
        let bullet = Span::styled("> ", Style::new().fg(title_sub_color));
        let skill_text = Span::styled(main_text, Style::new().fg(text_color));
        f.render_widget(
            Paragraph::new(Line::default().spans(vec![bullet, skill_text])),
            line_rect,
        );
        y_pos += 1;

        // Render sub-text on next line
        if let Some(sub) = sub_text {
            if y_pos < inner.y + inner.height {
                let sub_rect = Rect::new(inner.x, y_pos, inner.width, 1);
                let offset = Span::raw("  ");
                let sub_span = Span::styled(sub, Style::new().fg(title_sub_color));
                f.render_widget(
                    Paragraph::new(Line::default().spans(vec![offset, sub_span])),
                    sub_rect,
                );
                y_pos += 1;
            }
        }
        y_pos += skills_gap;
    }
}

fn render_about_block(
    f: &mut Frame<'_>,
    area: Rect,
    text: &str,
    cfg: &ColorsConfig,
    bg_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) {
    let block = create_block(" ABOUT ", border_color, title_color).bg(bg_color);
    let inner = block.inner(area);
    f.render_widget(&block, area);
    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(get_text_color(cfg)))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(paragraph, inner);
}

fn render_education_block(
    f: &mut Frame<'_>,
    area: Rect,
    text: &str,
    cfg: &ColorsConfig,
    bg_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) {
    let block = create_block(" EDUCATION ", border_color, title_color).bg(bg_color);
    let inner = block.inner(area);
    f.render_widget(&block, area);
    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(get_text_color(cfg)))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(paragraph, inner);
}

fn render_contacts_block(
    f: &mut Frame<'_>,
    area: Rect,
    text: &str,
    contacts_gap: u16,
    cfg: &ColorsConfig,
    bg_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) {
    let block = create_block(" CONTACTS ", border_color, title_color).bg(bg_color);
    let inner = block.inner(area);
    f.render_widget(&block, area);

    let text_color = get_text_color(cfg);
    let title_sub_color = get_title_sub_color(cfg);
    let line_height: u16 = 1;

    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    for (i, line) in lines.iter().enumerate() {
        let y_pos = inner.y + (i as u16 * (line_height + contacts_gap));
        if y_pos >= inner.y + inner.height {
            break;
        }

        let rect = Rect::new(inner.x, y_pos, inner.width, line_height);
        if let Some((label, value)) = line.split_once(':') {
            let label_span = Span::styled(
                format!("{}: ", label),
                Style::new()
                    .fg(title_sub_color)
                    .add_modifier(Modifier::BOLD),
            );
            let value_span = Span::styled(value.trim(), Style::new().fg(text_color));
            f.render_widget(
                Paragraph::new(Line::default().spans(vec![label_span, value_span])),
                rect,
            );
        } else {
            f.render_widget(
                Paragraph::new(*line).style(Style::new().fg(text_color)),
                rect,
            );
        }
    }
}

fn render_projects_block(
    f: &mut Frame<'_>,
    area: Rect,
    projects: &[String],
    projects_cfg: &ProjectsConfig,
    cfg: &ColorsConfig,
    bg_color: ratatui::style::Color,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) {
    let block = create_block(" PROJECTS ", border_color, title_color).bg(bg_color);
    let inner = block.inner(area);
    f.render_widget(&block, area);

    let heights = [
        projects_cfg.project_1_height,
        projects_cfg.project_2_height,
        projects_cfg.project_3_height,
        projects_cfg.project_4_height,
        projects_cfg.project_5_height,
        projects_cfg.project_6_height,
        projects_cfg.project_7_height,
        projects_cfg.project_8_height,
        projects_cfg.project_9_height,
        projects_cfg.project_10_height,
    ];

    let active: Vec<(usize, &String)> = projects
        .iter()
        .enumerate()
        .filter(|(i, _)| heights[*i] > 0)
        .collect();
    if active.is_empty() {
        return;
    }

    let constraints: Vec<Constraint> = active
        .iter()
        .map(|(i, _)| Constraint::Length(heights[*i]))
        .collect();
    let layout = Layout::vertical(constraints);
    let areas = layout.split(inner);

    let border_sub = get_border_sub_color(cfg);
    let title_sub = get_title_sub_color(cfg);
    let text_color = get_text_color(cfg);

    for (area_idx, (_proj_idx, project_text)) in active.iter().enumerate() {
        let lines: Vec<&str> = project_text.lines().collect();
        if lines.is_empty() {
            continue;
        }

        let title = lines.first().unwrap().trim();
        let content = lines.get(1..).map(|l| l.join("\n")).unwrap_or_default();

        let sub_block =
            create_sub_block(&format!(" {} ", title), border_sub, title_sub).bg(bg_color);
        let sub_inner = sub_block.inner(areas[area_idx]);
        f.render_widget(&sub_block, areas[area_idx]);

        let paragraph = Paragraph::new(content)
            .style(Style::new().fg(text_color))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(paragraph, sub_inner);
    }
}

fn create_block(
    title: &str,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(title.to_string())
        .border_style(Style::new().fg(border_color))
        .title_style(Style::new().fg(title_color).add_modifier(Modifier::BOLD))
}

fn create_sub_block(
    title: &str,
    border_color: ratatui::style::Color,
    title_color: ratatui::style::Color,
) -> Block<'static> {
    Block::default()
        .borders(Borders::TOP)
        .title(title.to_string())
        .border_style(Style::new().fg(border_color))
        .title_style(Style::new().fg(title_color).add_modifier(Modifier::BOLD))
}
