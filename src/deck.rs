use regex::Regex;

pub enum DeckArea {
    Main,
    Extra,
    Side,
}

pub struct Card {
    pub id: u32,
    pub name: String,
}

pub fn read_card_id_vec_string_ydk(deck_text: &String, deck_area: DeckArea) -> Vec<u32> {
    let start_pos;
    let end_pos;
    match deck_area {
        DeckArea::Main => {
            start_pos = deck_text.find("#main").unwrap();
            end_pos = deck_text.find("#extra").unwrap();
        }
        DeckArea::Extra => {
            start_pos = deck_text.find("#extra").unwrap();
            end_pos = deck_text.find("!side").unwrap();
        }
        DeckArea::Side => {
            start_pos = deck_text.find("!side").unwrap();
            end_pos = deck_text.len();
        }
    }
    let deck_area_text = &deck_text[start_pos..end_pos].trim();
    let mut row_str_vec: Vec<&str> = deck_area_text.split("\n").collect();
    let mut card_id_vec: Vec<u32> = vec![];
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
            card_id_vec.push(row_str.parse::<u32>().unwrap());
        }
    }
    return card_id_vec;
}
pub async fn card_query_ygocdb(id: u32) -> Card {
    let page = reqwest::get(format!("https://ygocdb.com/card/{}", id.to_string()))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut name = String::new();
    let name_regex = Regex::new("<h2><span>(.*)</span>").unwrap();
    if let Some(captures) = name_regex.captures(&page) {
        if let Some(content) = captures.get(1) {
            name = content.as_str().to_string();
        }
    }
    Card { id, name }
}
