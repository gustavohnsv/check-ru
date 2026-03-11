use std::path::Path;
use reqwest::Client;
use std::fs;
use crate::menu::{MenuData, DayMenu, Meal};
use regex::Regex;

pub async fn update_cache(base_path: &Path, unit_code: u32) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .cookie_store(true)
        .build()?;
    
    // 1. Visitar a página principal para estabelecer a sessão (JSESSIONID)
    let home_url = format!("https://uspdigital.usp.br/rucard/Jsp/cardapioSAS.jsp?codrtn={}", unit_code);
    let _ = client.get(home_url).send().await?;

    // 2. Buscar o nome do restaurante via DWR (obterRestauranteUsp)
    let name_url = "https://uspdigital.usp.br/rucard/dwr/call/plaincall/CardapioControleDWR.obterRestauranteUsp.dwr";
    let name_body = format!(
"callCount=1
windowName=
c0-scriptName=CardapioControleDWR
c0-methodName=obterRestauranteUsp
c0-id=0
c0-param0=string:{}
batchId=0
instanceId=0
page=%2Frucard%2FJsp%2FcardapioSAS.jsp%3Fcodrtn%3D{}
scriptSessionId=", unit_code, unit_code);

    let name_resp = client.post(name_url)
        .header("Content-Type", "text/plain")
        .body(name_body)
        .send()
        .await?
        .text()
        .await?;

    let mut restaurant_name = get_hardcoded_name_fallback(unit_code);
    let re_name = Regex::new(r#"nomrtn:"(.*?)"#).unwrap();
    if let Some(cap) = re_name.captures(&name_resp) {
        let n = cap[1].to_string();
        if n != "null" && !n.is_empty() {
            restaurant_name = n.replace("&iacute;", "í")
                .replace("&aacute;", "á")
                .replace("&otilde;", "õ")
                .replace("&Ccedil;", "Ç")
                .replace("&ccedil;", "ç")
                .replace("&atilde;", "ã")
                .to_string();
        }
    }

    // 3. Buscar o cardápio via DWR
    let dwr_url = "https://uspdigital.usp.br/rucard/dwr/call/plaincall/CardapioControleDWR.obterCardapioRestUSP.dwr";
    let body = format!(
"callCount=1
windowName=
c0-scriptName=CardapioControleDWR
c0-methodName=obterCardapioRestUSP
c0-id=0
c0-param0=string:{}
batchId=0
instanceId=0
page=%2Frucard%2FJsp%2FcardapioSAS.jsp%3Fcodrtn%3D{}
scriptSessionId=", unit_code, unit_code);

    let dwr_resp = client.post(dwr_url)
        .header("Content-Type", "text/plain")
        .body(body)
        .send()
        .await?
        .text()
        .await?;

    let mut menu_data = parse_dwr_response(&dwr_resp);
    menu_data.restaurant_name = restaurant_name;
    
    let cache_path = base_path.join("menu.json");
    let json = serde_json::to_string_pretty(&menu_data)?;
    fs::write(cache_path, json)?;

    Ok(())
}

fn get_hardcoded_name_fallback(id: u32) -> String {
    match id {
        1 => "Central - São Paulo".to_string(),
        2 => "Prefeitura - São Paulo".to_string(),
        3 => "Física - São Paulo".to_string(),
        4 => "Química - São Paulo".to_string(),
        5 => "Bauru".to_string(),
        6 => "São Carlos".to_string(),
        7 => "Prefeitura - Piracicaba".to_string(),
        8 => "EACH - São Paulo".to_string(),
        9 => "Ribeirão Preto".to_string(),
        11 => "Direito - São Paulo".to_string(),
        12 => "Enfermagem - São Paulo".to_string(),
        13 => "Poli - São Paulo".to_string(),
        14 => "IPEN - São Paulo".to_string(),
        17 => "Pirassununga".to_string(),
        18 => "Medicina - São Paulo".to_string(),
        19 => "Saúde Pública - São Paulo".to_string(),
        20 => "Lorena".to_string(),
        23 => "Oceanográfico - São Paulo".to_string(),
        _ => format!("Restaurante (ID {})", id),
    }
}

fn parse_dwr_response(resp: &str) -> MenuData {
    let mut menu_data = MenuData::empty();
    
    if let Some(start_idx) = resp.find("[{") {
        if let Some(end_idx) = resp.rfind("}]") {
            let data_content = &resp[start_idx + 1..end_idx + 1];
            
            for obj_str in data_content.split("},{") {
                let cdpdia = extract_field(obj_str, "cdpdia");
                let weekday_raw = extract_field(obj_str, "diasemana").parse::<u32>().unwrap_or(0);
                let dtaini = extract_field(obj_str, "dtainismncdp");
                let tiprfi = extract_field(obj_str, "tiprfi");
                let vlrclorfi = extract_field(obj_str, "vlrclorfi").parse::<u32>().unwrap_or(0);

                if cdpdia.is_empty() || cdpdia == "null" || cdpdia.to_lowercase().contains("fechado") {
                    continue;
                }

                let full_date = dtaini.replace(r"\u002F", "/").replace(r"\/", "/");
                let date_display = if full_date.len() >= 5 { full_date[..5].to_string() } else { full_date };
                let weekday = if weekday_raw == 1 { 7 } else { weekday_raw - 1 };

                let meal = parse_smart_meal(&cdpdia, vlrclorfi, &tiprfi);

                if let Some(day) = menu_data.days.iter_mut().find(|d| d.weekday == weekday) {
                    if tiprfi == "A" {
                        day.almoco = Some(meal);
                    } else {
                        day.jantar = Some(meal);
                    }
                } else {
                    let mut new_day = DayMenu {
                        date: date_display,
                        weekday,
                        almoco: None,
                        jantar: None,
                    };
                    if tiprfi == "A" {
                        new_day.almoco = Some(meal);
                    } else {
                        new_day.jantar = Some(meal);
                    }
                    menu_data.days.push(new_day);
                }
            }
        }
    }

    menu_data.days.sort_by_key(|d| d.weekday);
    menu_data
}

fn extract_field(obj: &str, field: &str) -> String {
    let patterns = [
        format!(r#"{}:"(.*?)""#, field),
        format!(r#"{}:([^,}}]*)"#, field),
    ];

    for p in &patterns {
        let re = Regex::new(p).unwrap();
        if let Some(cap) = re.captures(obj) {
            let val = cap[1].to_string();
            if val != "null" { return val; }
        }
    }
    "".to_string()
}

fn parse_smart_meal(raw: &str, calories: u32, meal_type: &str) -> Meal {
    let decoded = decode_unicode_escapes(raw);
    
    let cleaned = decoded
        .split("**")
        .next()
        .unwrap_or("")
        .replace("<br><br>", "<br>")
        .replace("\\/", ",")
        .replace("/", ",")
        .replace("\\", ",")
        .trim()
        .to_string();

    let lines: Vec<String> = cleaned
        .split("<br>")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut meal = Meal {
        opcoes: Vec::new(),
        guarnicao: String::new(),
        guarnicao_pvt: String::new(),
        acompanhamento: String::new(),
        salada: String::new(),
        sobremesa: String::new(),
        outros: Vec::new(),
        calories,
        meal_type: meal_type.to_string(),
    };

    if lines.is_empty() { return meal; }

    if let Some(first) = lines.get(0) {
        meal.opcoes = first
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if let Some(second) = lines.get(1) {
        meal.guarnicao = second.clone();
    }

    for i in 2..lines.len() {
        let line = &lines[i];
        let lower = line.to_lowercase();

        if lower.starts_with("opção:") || lower.starts_with("opcao:") {
            meal.guarnicao_pvt = line.replace("Opção:", "").replace("Opcao:", "").trim().to_string();
        } else if lower.starts_with("salada") {
            meal.salada = line.clone();
        } else if i == lines.len() - 2 {
            meal.sobremesa = line.clone();
        } else if i == lines.len() - 1 {
            meal.outros = line.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        } else if meal.acompanhamento.is_empty() {
            meal.acompanhamento = line.clone();
        }
    }

    meal
}

fn decode_unicode_escapes(s: &str) -> String {
    let re = Regex::new(r"\\u([0-9a-fA-F]{4})").unwrap();
    re.replace_all(s, |caps: &regex::Captures| {
        let hex = &caps[1];
        let char_code = u32::from_str_radix(hex, 16).unwrap_or(0);
        std::char::from_u32(char_code).unwrap_or('?').to_string()
    }).to_string()
}
