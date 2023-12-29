use std::{fs, io::Read};

use regex::Regex;

enum DeckArea {
    Main,
    Extra,
    Side,
}

pub struct Card {
    pub id: u32,
    pub name: String,
}
#[allow(dead_code)]
impl Card {
    pub fn new(name: String) -> Self {
        Self { id: 0, name }
    }
    pub async fn from_id(id: u32) -> Result<Self, String> {
        let name;
        match reqwest::get(format!("https://ygocdb.com/card/{}", id.to_string())).await {
            Ok(r) => match r.text().await {
                Ok(p) => {
                    let name_regex;
                    match Regex::new("<h2><span>(.*)</span>") {
                        Ok(r) => name_regex = r,
                        Err(e) => return Err(format!("{}", e)),
                    }
                    if let Some(captures) = name_regex.captures(&p) {
                        if let Some(content) = captures.get(1) {
                            name = content.as_str().to_string();
                        } else {
                            return Err("获取匹配的内容失败".to_string());
                        }
                    } else {
                        return Err("获取捕获器失败".to_string());
                    }
                }
                Err(e) => return Err(format!("{}", e)),
            },
            Err(e) => return Err(format!("{}", e)),
        }
        Ok(Self { id, name })
    }
}

pub struct Deck {
    pub name: String,
    pub main: Vec<Card>,
    pub extra: Vec<Card>,
    pub side: Vec<Card>,
}
#[allow(dead_code)]
impl Deck {
    pub fn new(name: String) -> Self {
        Self {
            name,
            main: vec![],
            extra: vec![],
            side: vec![],
        }
    }
    pub async fn from_file(name: String, file_path: String) -> Result<Self, String> {
        let mut deck_text = String::new();
        match fs::File::open(file_path) {
            Ok(mut f) => match f.read_to_string(&mut deck_text) {
                Ok(_) => (),
                Err(e) => return Err(format!("{}", e)),
            },
            Err(e) => return Err(format!("{}", e)),
        }
        Self::from_text(name, deck_text).await
    }
    pub async fn from_text(name: String, deck_text: String) -> Result<Self, String> {
        let mut main = vec![];
        match Self::read_deck_card_id(&deck_text, DeckArea::Main) {
            Ok(dil) => {
                for id in dil {
                    match Card::from_id(id).await {
                        Ok(c) => main.push(c),
                        Err(e) => return Err(e),
                    }
                }
            }
            Err(e) => return Err(e),
        }
        let mut extra = vec![];
        match Self::read_deck_card_id(&deck_text, DeckArea::Extra) {
            Ok(dil) => {
                for id in dil {
                    match Card::from_id(id).await {
                        Ok(c) => extra.push(c),
                        Err(e) => return Err(e),
                    }
                }
            }
            Err(e) => return Err(e),
        }
        let mut side = vec![];
        match Self::read_deck_card_id(&deck_text, DeckArea::Side) {
            Ok(dil) => {
                for id in dil {
                    match Card::from_id(id).await {
                        Ok(c) => side.push(c),
                        Err(e) => return Err(e),
                    }
                }
            }
            Err(e) => return Err(e),
        }
        Ok(Self {
            name,
            main,
            extra,
            side,
        })
    }
    fn read_deck_card_id(deck_text: &String, deck_area: DeckArea) -> Result<Vec<u32>, String> {
        let mut start_pos = 0;
        let mut end_pos = 0;
        match deck_area {
            DeckArea::Main => {
                if let Some(i) = deck_text.find("#main") {
                    start_pos = i;
                }
                if let Some(i) = deck_text.find("#extra") {
                    end_pos = i;
                }
            }
            DeckArea::Extra => {
                if let Some(i) = deck_text.find("#extra") {
                    start_pos = i;
                }
                if let Some(i) = deck_text.find("!side") {
                    end_pos = i;
                }
            }
            DeckArea::Side => {
                if let Some(i) = deck_text.find("!side") {
                    start_pos = i;
                    end_pos = deck_text.len();
                }
            }
        }
        if end_pos == 0 {
            return Err("读取的卡组文本不符合规则".to_string());
        }
        let deck_area_text = deck_text[start_pos..end_pos].trim();
        let mut row_str_vec: Vec<&str> = deck_area_text.split("\n").collect();
        let mut card_id_vec = vec![];
        for row_str in &mut row_str_vec {
            //清理行后检查是否为纯数字
            *row_str = row_str.trim();
            let mut is_ascii_digit = true;
            for c in row_str.chars() {
                if !c.is_ascii_digit() {
                    is_ascii_digit = false;
                    break;
                }
            }
            //如果不为纯数字则跳过
            if is_ascii_digit {
                let card_id;
                match row_str.parse::<u32>() {
                    Ok(id) => card_id = id,
                    Err(e) => return Err(format!("{}", e)),
                }
                card_id_vec.push(card_id);
            }
        }
        Ok(card_id_vec)
    }
}
