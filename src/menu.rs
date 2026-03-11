use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meal {
    pub opcoes: Vec<String>,
    pub guarnicao: String,
    pub guarnicao_pvt: String,
    pub acompanhamento: String,
    pub salada: String,
    pub sobremesa: String,
    pub outros: Vec<String>,
    pub calories: u32,
    pub meal_type: String, // "A" ou "J"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DayMenu {
    pub date: String,
    pub weekday: u32, // 1=Segunda, ..., 7=Domingo
    pub almoco: Option<Meal>,
    pub jantar: Option<Meal>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MenuData {
    pub restaurant_name: String,
    pub days: Vec<DayMenu>,
}

impl MenuData {
    pub fn empty() -> Self {
        Self {
            restaurant_name: "Restaurante USP".to_string(),
            days: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let menu: Self = serde_json::from_str(&data)?;
        Ok(menu)
    }

    pub fn get_day(&self, weekday: usize) -> Option<DayMenu> {
        self.days.iter().find(|d| d.weekday as usize == weekday).cloned()
    }
}
