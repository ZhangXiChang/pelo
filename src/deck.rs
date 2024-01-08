use std::{fs, io::Read};

use anyhow::{anyhow, Result};
use regex::Regex;

enum CardFromType {
    Id(u32),
}
#[allow(unused)]
pub enum DeckFromType {
    Text(String),
    File(String),
}
enum DeckArea {
    Main,
    Extra,
    Side,
}

#[allow(unused)]
struct Card {
    id: u32,
    name: String,
}
impl Card {
    async fn from(from_type: CardFromType) -> Result<Self> {
        match from_type {
            CardFromType::Id(id) => {
                if let Some(captures) = Regex::new("<h2><span>(.*)</span>")?.captures(
                    &reqwest::get(format!("https://ygocdb.com/card/{}", id))
                        .await?
                        .text()
                        .await?,
                ) {
                    if let Some(content) = captures.get(1) {
                        return Ok(Self {
                            id,
                            name: content.as_str().to_string(),
                        });
                    } else {
                        return Err(anyhow!("获取匹配的内容失败"));
                    }
                } else {
                    return Err(anyhow!("获取捕获器失败"));
                }
            }
        }
    }
}

#[allow(unused)]
pub struct Deck {
    name: String,
    main: Vec<Card>,
    extra: Vec<Card>,
    side: Vec<Card>,
}
impl Deck {
    pub async fn from(name: String, from_type: DeckFromType) -> Result<Self> {
        let mut main = vec![];
        let mut extra = vec![];
        let mut side = vec![];
        match from_type {
            DeckFromType::Text(deck_text) => {
                for id in Self::read_deck_card_id(&deck_text, DeckArea::Main)? {
                    main.push(Card::from(CardFromType::Id(id)).await?);
                }
                for id in Self::read_deck_card_id(&deck_text, DeckArea::Extra)? {
                    extra.push(Card::from(CardFromType::Id(id)).await?);
                }
                for id in Self::read_deck_card_id(&deck_text, DeckArea::Side)? {
                    side.push(Card::from(CardFromType::Id(id)).await?);
                }
            }
            DeckFromType::File(deck_file_path) => {
                let mut file = fs::File::open(deck_file_path)?;
                let mut deck_text = String::new();
                file.read_to_string(&mut deck_text)?;
                for id in Self::read_deck_card_id(&deck_text, DeckArea::Main)? {
                    main.push(Card::from(CardFromType::Id(id)).await?);
                }
                for id in Self::read_deck_card_id(&deck_text, DeckArea::Extra)? {
                    extra.push(Card::from(CardFromType::Id(id)).await?);
                }
                for id in Self::read_deck_card_id(&deck_text, DeckArea::Side)? {
                    side.push(Card::from(CardFromType::Id(id)).await?);
                }
            }
        }
        Ok(Self {
            name,
            main,
            extra,
            side,
        })
    }
    fn read_deck_card_id(deck_text: &String, deck_area: DeckArea) -> Result<Vec<u32>> {
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
            return Err(anyhow!("读取的卡组文本不符合规则"));
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
}
