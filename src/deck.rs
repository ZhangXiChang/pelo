use std::{fs, io::Read};

use regex::Regex;

use crate::error::Error;

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
        Self {
            id: Default::default(),
            name,
        }
    }
    pub async fn from_id(id: u32) -> Result<Self, Error> {
        let name;
        if let Some(captures) = Regex::new("<h2><span>(.*)</span>")?.captures(
            &reqwest::get(format!("https://ygocdb.com/card/{}", id))
                .await?
                .text()
                .await?,
        ) {
            if let Some(content) = captures.get(1) {
                name = content.as_str().to_string();
            } else {
                return Err("获取匹配的内容失败".into());
            }
        } else {
            return Err("获取捕获器失败".into());
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
    pub async fn from_file(name: String, file_path: String) -> Result<Self, Error> {
        let mut file = fs::File::open(file_path)?;
        let mut deck_text = String::new();
        file.read_to_string(&mut deck_text)?;
        Ok(Self::from_text(name, deck_text).await?)
    }
    async fn from_text(name: String, deck_text: String) -> Result<Self, Error> {
        let mut main = vec![];
        for id in Self::read_deck_card_id(&deck_text, DeckArea::Main)? {
            main.push(Card::from_id(id).await?);
        }
        let mut extra = vec![];
        for id in Self::read_deck_card_id(&deck_text, DeckArea::Extra)? {
            extra.push(Card::from_id(id).await?);
        }
        let mut side = vec![];
        for id in Self::read_deck_card_id(&deck_text, DeckArea::Side)? {
            side.push(Card::from_id(id).await?);
        }
        Ok(Self {
            name,
            main,
            extra,
            side,
        })
    }
    fn read_deck_card_id(deck_text: &String, deck_area: DeckArea) -> Result<Vec<u32>, Error> {
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
            return Err("读取的卡组文本不符合规则".into());
        }
        let deck_area_text = deck_text[start_pos..end_pos].trim();
        let mut row_str_list: Vec<&str> = deck_area_text.split("\n").collect();
        let mut card_id_list = vec![];
        for row_str in &mut row_str_list {
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
                card_id_list.push(row_str.parse::<u32>()?);
            }
        }
        Ok(card_id_list)
    }
    pub fn new(name: String) -> Self {
        Self {
            name,
            main: Default::default(),
            extra: Default::default(),
            side: Default::default(),
        }
    }
}
