use serde::{Deserialize, Serialize};
use ratatui::style::Color;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppTheme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub highlight: String,
    pub error: String,
    pub label_opcoes: String,
    pub label_guarnicao: String,
    pub label_pvt: String,
    pub label_acompanhamento: String,
    pub label_salada: String,
    pub label_sobremesa: String,
    pub label_outros: String,
}

impl AppTheme {
    pub fn presets() -> Vec<Self> {
        vec![Self::dark()]
    }

    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            background: "#000000".to_string(),
            foreground: "#FFFFFF".to_string(),
            primary: "#00FFFF".to_string(),
            secondary: "#444444".to_string(),
            highlight: "#FF00FF".to_string(),
            error: "#FF0000".to_string(),
            label_opcoes: "#FFFF00".to_string(),
            label_guarnicao: "#FF5555".to_string(),
            label_pvt: "#55FF55".to_string(),
            label_acompanhamento: "#5555FF".to_string(),
            label_salada: "#00FF00".to_string(),
            label_sobremesa: "#FF55FF".to_string(),
            label_outros: "#AAAAAA".to_string(),
        }
    }

    pub fn background(&self) -> Color { parse_color(&self.background) }
    pub fn foreground(&self) -> Color { parse_color(&self.foreground) }
    pub fn primary(&self) -> Color { parse_color(&self.primary) }
    pub fn secondary(&self) -> Color { parse_color(&self.secondary) }
    pub fn highlight(&self) -> Color { parse_color(&self.highlight) }
    #[allow(dead_code)] pub fn error(&self) -> Color { parse_color(&self.error) }
    pub fn label_opcoes(&self) -> Color { parse_color(&self.label_opcoes) }
    pub fn label_guarnicao(&self) -> Color { parse_color(&self.label_guarnicao) }
    pub fn label_pvt(&self) -> Color { parse_color(&self.label_pvt) }
    pub fn label_acompanhamento(&self) -> Color { parse_color(&self.label_acompanhamento) }
    pub fn label_salada(&self) -> Color { parse_color(&self.label_salada) }
    pub fn label_sobremesa(&self) -> Color { parse_color(&self.label_sobremesa) }
    pub fn label_outros(&self) -> Color { parse_color(&self.label_outros) }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub theme_name: String,
    pub daily_check: bool,
    pub unit_code: u32,
}

impl Config {
    pub fn load(path: &Path) -> Self {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&data) {
                return config;
            }
        }
        Config { 
            theme_name: "Dark".to_string(),
            daily_check: false,
            unit_code: 13,
        }
    }

    pub fn save(&self, path: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}

fn parse_color(color: &str) -> Color {
    if color.starts_with('#') {
        let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);
        return Color::Rgb(r, g, b);
    }

    match color.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => Color::Reset,
    }
}
