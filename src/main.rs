use std::{path::PathBuf, io, time::{Duration, Instant}, fs};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use chrono::{Datelike, Local};

mod menu;
mod theme;
mod fetcher;

use menu::{MenuData, Meal, DayMenu};
use theme::{AppTheme, Config};

struct App {
    menu_data: MenuData,
    all_themes: Vec<AppTheme>,
    current_theme_idx: usize,
    selected_day_idx: usize,
    days: Vec<String>,
    tick: u64,
    base_path: PathBuf,
    show_help: bool,
    config: Config,
}

fn set_terminal_background(hex: &str) {
    use std::io::{stdout, Write};
    print!("\x1b]11;{}\x07", hex);
    let _ = stdout().flush();
}

fn reset_terminal_background() {
    use std::io::{stdout, Write};
    print!("\x1b]111\x07"); // Reset terminal background to default
    let _ = stdout().flush();
}

impl App {
    fn new(menu_data: MenuData, base_path: PathBuf, config: Config) -> App {
        let days = vec![
            "Segunda".to_string(),
            "Terça".to_string(),
            "Quarta".to_string(),
            "Quinta".to_string(),
            "Sexta".to_string(),
            "Sábado".to_string(),
            "Domingo".to_string(),
        ];
        
        let now = Local::now();
        let current_weekday = now.weekday().number_from_monday() as usize - 1;
        let selected_day_idx = if current_weekday < 7 { current_weekday } else { 0 };

        let mut all_themes = Vec::new();
        let mut seen_names = std::collections::HashSet::new();
        let themes_dir = base_path.join("themes");
        if let Ok(entries) = fs::read_dir(themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(data) = fs::read_to_string(&path) {
                        if let Ok(theme) = serde_json::from_str::<AppTheme>(&data) {
                            let name_norm = theme.name.trim().to_lowercase();
                            if !seen_names.contains(&name_norm) {
                                seen_names.insert(name_norm);
                                all_themes.push(theme);
                            }
                        }
                    }
                }
            }
        }

        if all_themes.is_empty() {
            all_themes = AppTheme::presets();
        }

        all_themes.sort_by(|a, b| a.name.trim().to_lowercase().cmp(&b.name.trim().to_lowercase()));
        all_themes.dedup_by(|a, b| a.name.trim().to_lowercase() == b.name.trim().to_lowercase());

        let current_theme_idx = all_themes.iter()
            .position(|t| {
                let name_norm = t.name.trim().to_lowercase();
                let config_name_norm = config.theme_name.trim().to_lowercase();
                name_norm == config_name_norm
            })
            .unwrap_or(0);

        let mut app = App {
            menu_data,
            all_themes,
            current_theme_idx,
            selected_day_idx,
            days,
            tick: 0,
            base_path,
            show_help: false,
            config,
        };
        
        set_terminal_background(&app.theme().background);
        app
    }

    fn theme(&self) -> &AppTheme {
        &self.all_themes[self.current_theme_idx]
    }

    fn next_theme(&mut self) {
        self.current_theme_idx = (self.current_theme_idx + 1) % self.all_themes.len();
        self.config.theme_name = self.theme().name.clone();
        self.config.save(&self.base_path.join("config.json"));
        set_terminal_background(&self.theme().background);
        self.tick = 0;
    }

    fn next_day(&mut self) {
        self.selected_day_idx = (self.selected_day_idx + 1) % self.days.len();
        self.tick = 0;
    }

    fn prev_day(&mut self) {
        if self.selected_day_idx > 0 {
            self.selected_day_idx -= 1;
        } else {
            self.selected_day_idx = self.days.len() - 1;
        }
        self.tick = 0;
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn on_tick(&mut self) {
        self.tick += 1;
    }
}

fn load_all_themes(base_path: &PathBuf) -> Vec<AppTheme> {
    let mut all_themes = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    let themes_dir = base_path.join("themes");
    if let Ok(entries) = fs::read_dir(themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(theme) = serde_json::from_str::<AppTheme>(&data) {
                        let name_norm = theme.name.trim().to_lowercase();
                        if !seen_names.contains(&name_norm) {
                            seen_names.insert(name_norm);
                            all_themes.push(theme);
                        }
                    }
                }
            }
        }
    }

    if all_themes.is_empty() {
        all_themes = AppTheme::presets();
    }

    all_themes.sort_by(|a, b| a.name.trim().to_lowercase().cmp(&b.name.trim().to_lowercase()));
    all_themes.dedup_by(|a, b| a.name.trim().to_lowercase() == b.name.trim().to_lowercase());
    all_themes
}

fn get_current_theme(all_themes: &[AppTheme], theme_name: &str) -> AppTheme {
    all_themes.iter()
        .find(|t| t.name.trim().to_lowercase() == theme_name.trim().to_lowercase())
        .cloned()
        .unwrap_or_else(|| all_themes[0].clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    let mut base_path = std::env::current_exe()?
        .parent()
        .ok_or("Não foi possível encontrar o diretório do executável")?
        .to_path_buf();

    if base_path.ends_with("target/release") || base_path.ends_with("target/debug") {
        if let Some(parent) = base_path.parent().and_then(|p| p.parent()) {
            base_path = parent.to_path_buf();
        }
    }
    
    if args.len() > 1 && args[1] == "--fetch" {
        let config = Config::load(&base_path.join("config.json"));
        println!("Atualizando cardápio...");
        fetcher::update_cache(&base_path, config.unit_code).await?;
        println!("Cache atualizado!");
        return Ok(());
    }

    let config = Config::load(&base_path.join("config.json"));
    let menu_data = MenuData::load(&base_path.join("menu.json")).unwrap_or_else(|_| {
        MenuData::empty()
    });

    if config.daily_check {
        let all_themes = load_all_themes(&base_path);
        let theme = get_current_theme(&all_themes, &config.theme_name);
        print_daily_menu(&menu_data, &theme);
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(menu_data, base_path, config);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    reset_terminal_background();

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn colorize(text: &str, color: Color) -> String {
    let ansi = match color {
        Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
        Color::Reset => "\x1b[0m".to_string(),
        Color::Black => "\x1b[30m".to_string(),
        Color::Red => "\x1b[31m".to_string(),
        Color::Green => "\x1b[32m".to_string(),
        Color::Yellow => "\x1b[33m".to_string(),
        Color::Blue => "\x1b[34m".to_string(),
        Color::Magenta => "\x1b[35m".to_string(),
        Color::Cyan => "\x1b[36m".to_string(),
        Color::Gray => "\x1b[37m".to_string(),
        Color::DarkGray => "\x1b[90m".to_string(),
        Color::LightRed => "\x1b[91m".to_string(),
        Color::LightGreen => "\x1b[92m".to_string(),
        Color::LightYellow => "\x1b[93m".to_string(),
        Color::LightBlue => "\x1b[94m".to_string(),
        Color::LightMagenta => "\x1b[95m".to_string(),
        Color::LightCyan => "\x1b[96m".to_string(),
        Color::White => "\x1b[97m".to_string(),
        _ => "\x1b[0m".to_string(),
    };
    format!("{}{}\x1b[0m", ansi, text)
}

fn print_daily_menu(menu_data: &MenuData, theme: &AppTheme) {
    let now = Local::now();
    let current_weekday = now.weekday().number_from_monday() as usize - 1;
    let selected_day_idx = if current_weekday < 7 { current_weekday } else { 0 };

    let box_width: usize = 70;
    let border_color = theme.primary();

    // Restaurante Name Box
    let section_title = " Restaurante ";
    let section_title_len = section_title.chars().count();
    let dashes = box_width.saturating_sub(section_title_len + 2);
    let res_top = format!("┌{}{}{}┐", section_title, "─".repeat(dashes), "");
    println!("\n{}", colorize(&res_top, border_color));
    
    let inner_space = box_width.saturating_sub(6);
    let label = "Unidade selecionada: ";
    let label_len = label.chars().count();
    let val = &menu_data.restaurant_name;
    let val_len = val.chars().count();
    let padding_len = inner_space.saturating_sub(label_len + val_len);
    let padding = " ".repeat(padding_len);

    println!("{}  {}{}{}  {}", 
        colorize("│", border_color),
        colorize(label, theme.highlight()),
        val,
        padding,
        colorize("│", border_color)
    );
    println!("{}", colorize(&format!("└{}┘", "─".repeat(box_width - 2)), border_color));

    if let Some(day) = menu_data.days.get(selected_day_idx) {
        let weekday_names = vec!["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo"];
        
        // Agenda Box
        let agenda_title = " Agenda ";
        let agenda_title_len = agenda_title.chars().count();
        let dashes = box_width.saturating_sub(agenda_title_len + 2);
        let agenda_top = format!("┌{}{}{}┐", agenda_title, "─".repeat(dashes), "");
        println!("{}", colorize(&agenda_top, border_color));

        let inner_space = box_width.saturating_sub(6);
        let day_text = format!("{} ({})", weekday_names[selected_day_idx], day.date);
        let padding_len = inner_space.saturating_sub(day_text.chars().count());
        let padding = " ".repeat(padding_len);

        println!("{}  {}  {}", 
            colorize("│", border_color),
            day_text,
            format!("{}{}", padding, colorize("│", border_color))
        );
        println!("{}", colorize(&format!("└{}┘", "─".repeat(box_width - 2)), border_color));
        
        print_meal_boxed("ALMOÇO", day.almoco.as_ref(), theme);
        print_meal_boxed("JANTAR", day.jantar.as_ref(), theme);
    } else {
        println!("{}", colorize("Dados do cardápio não encontrados para hoje.", theme.error()));
    }
}

fn print_meal_boxed(title: &str, meal: Option<&Meal>, theme: &AppTheme) {
    let box_width: usize = 70; // Largura total fixa da caixa
    let border_color = theme.primary();
    
    // Top border: ┌── TITULO ──────────────────────────────────────────┐
    let prefix = format!("── {} ", title);
    let prefix_len = prefix.chars().count();
    let dashes_count = box_width.saturating_sub(prefix_len + 2); // -2 para cantos ┌ e ┐
    let top_border = format!("┌{}{}{}┐", prefix, "─".repeat(dashes_count), "");
    println!("{}", colorize(&top_border, border_color));

    let print_row = |label: &str, value: &str, label_color: Color| {
        if value.is_empty() || value == "[]" { return; }

        let label_text = if label.is_empty() { "".to_string() } else { format!("{}: ", label) };
        let label_len = label_text.chars().count();
        
        // Espaço interno total: box_width - 2 (bordas laterais) - 4 (margens internas)
        let inner_space = box_width.saturating_sub(6);
        let avail_for_val = inner_space.saturating_sub(label_len);
        
        let display_val = if value.chars().count() > avail_for_val {
            let mut v: String = value.chars().take(avail_for_val.saturating_sub(3)).collect();
            v.push_str("...");
            v
        } else {
            value.to_string()
        };
        
        let display_val_len = display_val.chars().count();
        let padding_len = inner_space.saturating_sub(label_len + display_val_len);
        let padding = " ".repeat(padding_len);
        
        println!("{}  {}{}{}  {}", 
            colorize("│", border_color),
            colorize(&label_text, label_color),
            display_val,
            padding,
            colorize("│", border_color)
        );
    };

    if let Some(m) = meal {
        print_row("Opções", &m.opcoes.join(", "), theme.label_opcoes());
        print_row("Guarnição", &m.guarnicao, theme.label_guarnicao());
        print_row("Guarnição PVT", &m.guarnicao_pvt, theme.label_pvt());
        print_row("Acompanhamento", &m.acompanhamento, theme.label_acompanhamento());
        print_row("Salada", &m.salada, theme.label_salada());
        print_row("Sobremesa", &m.sobremesa, theme.label_sobremesa());
        print_row("Outros", &m.outros.join(", "), theme.label_outros());
        print_row("Calorias", &format!("{} kcal", m.calories), theme.highlight());
    } else {
        print_row("", "Sem cardápio cadastrado", theme.secondary());
    }

    // Bottom border: └─────────────────────────────────────────────────┘
    let bottom_border = format!("└{}┘", "─".repeat(box_width - 2));
    println!("{}", colorize(&bottom_border, border_color));
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    
    loop {
        terminal.draw(|f| ui(f, app)).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => app.toggle_help(),
                    KeyCode::Tab => if !app.show_help { app.next_theme() },
                    KeyCode::Left => if !app.show_help { app.prev_day() },
                    KeyCode::Right => if !app.show_help { app.next_day() },
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn apply_marquee(value: &str, available_space: usize, tick: u64) -> String {
    let text_len = value.chars().count();
    if text_len > available_space && available_space > 0 {
        let diff = text_len - available_space;
        let pause_ticks = 15;
        let cycle_ticks = diff as u64 + (pause_ticks * 2);
        let current_cycle_tick = tick % cycle_ticks;
        
        let start_index = if current_cycle_tick < pause_ticks {
            0
        } else if current_cycle_tick < pause_ticks + diff as u64 {
            (current_cycle_tick - pause_ticks) as usize
        } else {
            diff
        };

        value.chars().skip(start_index).take(available_space).collect::<String>()
    } else {
        value.to_string()
    }
}

fn ui(f: &mut Frame, app: &App) {
    let theme = app.theme();
    
    let area = f.area();
    // Removido Clear redundante que causava flickering
    
    let main_block = Block::default().style(Style::default().bg(theme.background()));
    f.render_widget(main_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Restaurante
            Constraint::Length(3), // Agenda (Dias)
            Constraint::Min(0),    // Conteúdo
            Constraint::Length(3), // Temas
            Constraint::Length(1), // Rodapé
        ])
        .split(f.area());

    // Bloco de Restaurante
    let res_block = Block::default()
        .borders(Borders::ALL)
        .title(" Restaurante ")
        .border_style(Style::default().fg(theme.primary()));
    let res_text = Line::from(vec![
        Span::styled("Unidade selecionada: ", Style::default().fg(theme.highlight()).add_modifier(Modifier::BOLD)),
        Span::styled(&app.menu_data.restaurant_name, Style::default().fg(theme.foreground())),
    ]);
    f.render_widget(Paragraph::new(res_text).block(res_block), chunks[0]);

    let titles: Vec<Line> = app.days.iter().cloned().map(Line::from).collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Agenda "))
        .select(app.selected_day_idx)
        .divider(" | ")
        .style(Style::default().fg(theme.foreground()))
        .highlight_style(Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[1]);

    if app.show_help {
        render_help(f, chunks[2], theme);
    } else {
        let day_data = app.menu_data.get_day(app.selected_day_idx + 1);
        let meal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        render_meal(f, meal_chunks[0], "Almoço", day_data.as_ref().and_then(|d| d.almoco.as_ref()), theme, day_data.as_ref(), app.tick);
        render_meal(f, meal_chunks[1], "Jantar", day_data.as_ref().and_then(|d| d.jantar.as_ref()), theme, day_data.as_ref(), app.tick);
    }

    let theme_titles: Vec<Line> = app.all_themes.iter().map(|t| Line::from(t.name.clone())).collect();
    let theme_tabs = Tabs::new(theme_titles)
        .block(Block::default().borders(Borders::ALL).title(" Tema "))
        .select(app.current_theme_idx)
        .divider(" | ")
        .style(Style::default().fg(theme.secondary()))
        .highlight_style(Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD));
    f.render_widget(theme_tabs, chunks[3]);

    let footer_text = if app.show_help { "ESC: Voltar" } else { "ESC: Ajuda" };
    let footer = Paragraph::new(footer_text).style(Style::default().fg(theme.secondary()));
    f.render_widget(footer, chunks[4]);
}

fn render_help(f: &mut Frame, area: Rect, theme: &AppTheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Ajuda")
        .border_style(Style::default().fg(theme.primary()));

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("- Navegação", Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("        - ← / → : ", Style::default().fg(theme.highlight()).add_modifier(Modifier::BOLD)),
            Span::styled("Altera o dia da semana atual.", Style::default().fg(theme.foreground())),
        ]),
        Line::from(vec![
            Span::styled("        - Tab   : ", Style::default().fg(theme.highlight()).add_modifier(Modifier::BOLD)),
            Span::styled("Cicla entre os temas disponíveis.", Style::default().fg(theme.foreground())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("- Sistema", Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("        - Esc   : ", Style::default().fg(theme.highlight()).add_modifier(Modifier::BOLD)),
            Span::styled("Alterna entre cardápio e ajuda.", Style::default().fg(theme.foreground())),
        ]),
        Line::from(vec![
            Span::styled("        - q     : ", Style::default().fg(theme.highlight()).add_modifier(Modifier::BOLD)),
            Span::styled("Encerra a aplicação imediatamente.", Style::default().fg(theme.foreground())),
        ]),
        Line::from(""),
        Line::from(Span::styled("─".repeat(area.width as usize - 2), Style::default().fg(theme.secondary()))),
        Line::from(""),
        Line::from("    - O cardápio é atualizado a cada 6 horas."),
        Line::from("    - Textos longos usam letreiro animado."),
        Line::from("    - A escolha de tema é salva automaticamente."),
    ];

    let p = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().fg(theme.foreground()));
    f.render_widget(p, area);
}

fn render_meal(f: &mut Frame, area: Rect, title: &str, meal: Option<&Meal>, theme: &AppTheme, day: Option<&DayMenu>, tick: u64) {
    let mut title_with_date = title.to_string();
    if let Some(d) = day { title_with_date = format!("{} ({})", title, d.date); }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title_with_date)
        .border_style(Style::default().fg(theme.primary()))
        .style(Style::default().bg(theme.background()));

    if let Some(m) = meal {
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let mut lines = Vec::new();
        let max_width = inner_chunks[0].width as usize;

        let mut add_item = |label: &str, value: String, color: Color| {
            if !value.is_empty() && value != "[]" {
                let label_text = format!("{}: ", label);
                let label_len = label_text.chars().count();
                let display_value = apply_marquee(&value, max_width.saturating_sub(label_len), tick);
                lines.push(Line::from(vec![
                    Span::styled(label_text, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(display_value, Style::default().fg(theme.foreground())),
                ]));
                lines.push(Line::from(Span::styled("─".repeat(max_width), Style::default().fg(theme.secondary()))));
            }
        };

        add_item("Opções", m.opcoes.join(", "), theme.label_opcoes());
        add_item("Guarnição", m.guarnicao.clone(), theme.label_guarnicao());
        add_item("Guarnição PVT", m.guarnicao_pvt.clone(), theme.label_pvt());
        add_item("Acompanhamento", m.acompanhamento.clone(), theme.label_acompanhamento());
        add_item("Salada", m.salada.clone(), theme.label_salada());
        add_item("Sobremesa", m.sobremesa.clone(), theme.label_sobremesa());
        add_item("Outros", m.outros.join(", "), theme.label_outros());

        f.render_widget(block, area);
        f.render_widget(Paragraph::new(lines).style(Style::default().fg(theme.foreground())), inner_chunks[0]);
        let cal_line = Line::from(vec![
            Span::styled("Calorias: ", Style::default().fg(theme.highlight()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", m.calories), Style::default().fg(theme.foreground())),
        ]);
        f.render_widget(Paragraph::new(cal_line), inner_chunks[1]);
    } else {
        f.render_widget(block, area);
        let p = Paragraph::new("Sem cardápio cadastrado").style(Style::default().fg(theme.secondary()));
        let inner_chunks = Layout::default().margin(1).constraints([Constraint::Min(0)]).split(area);
        f.render_widget(p, inner_chunks[0]);
    }
}
