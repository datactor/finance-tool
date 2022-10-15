#![allow(non_snake_case)]

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use finance::app::{ApiChoice, FinanceClient};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Mutex;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub const API_KEY: &str = include_str!("../key.txt");

/// todo! Make into real error
enum ClientError {
    IncorrectInput,
}

// 1 Make it work
// 2 Make it nice
// 3 Make it fast

// 관용구 기억하기 needle in a haystack
fn company_search(needle: &str, haystack: &[(&str, &str)]) -> String {
    haystack
        .iter()
        .filter_map(|(company_name, company_symbol)| {
            let needle = needle.to_lowercase();
            let company_name = company_name.to_lowercase();
            if company_name.contains(&needle) {
                Some(format!("{}: {}", company_symbol, company_name))
            } else {
                None
            }
        })
        .collect::<String>()
}

// Rust 1.63
// Global variable
static SOMETHING: Mutex<String> = Mutex::new(String::new());

const COMPANY_STR: &str = include_str!("../company_symbols.json");

#[derive(Debug, Deserialize)]
struct CompanySymbolInfo((String, String));

fn main() -> Result<(), anyhow::Error> {
    // hashmap으로 검색하려면 키가 맞아야하기때문에 string의 일부로 검색하기 부적합
    // let companies: HashMap<&str, &str> = serde_json::from_str(COMPANY_STR).unwrap();
    let companies = serde_json::from_str::<BTreeMap<&str, &str>>(COMPANY_STR)
        .unwrap()
        .into_iter()
        .map(|(key, value)| (key, value))
        .collect::<Vec<(_, _)>>();

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client = FinanceClient {
        url: "https://finnhub.io/api/v1/".to_string(),
        client: Client::default(),
        search_string: String::new(),
        current_content: String::new(),
        choice: ApiChoice::CompanyInfo,
    };

    // Input
    // State change / enum, char+, char-
    // Draw

    loop {
        match read().unwrap() {
            Event::Key(key_event) => {
                // println!("Got a KeyEvent: {key_event:?}");
                let KeyEvent {
                    code, modifiers, ..
                } = key_event;
                // Typing event
                match (code, modifiers) {
                    (KeyCode::Char(c), modifier)
                        if c == 'q' && modifier == KeyModifiers::CONTROL =>
                    // ctrl-c는 os가 먼저 가져감
                    {
                        // tokio graceful shutdown도 있음
                        break;
                    }
                    (KeyCode::Char(c), _) => {
                        client.search_string.push(c);
                    }
                    (KeyCode::Esc, _) => {
                        client.search_string.clear();
                    }
                    (KeyCode::Backspace, _) => {
                        client.search_string.pop();
                    }
                    (KeyCode::Enter, _) => {
                        client.current_content = match client.company_profile() {
                            Ok(search_result) => search_result,
                            Err(e) => e.to_string(),
                        };
                    }
                    (KeyCode::Tab, _) => {
                        client.switch();
                    }
                    (_, _) => {}
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(num1, num2) => {
                println!("Window has been resized to {num1}, {num2}")
            }
            Event::Paste(_) => {}
            _ => {}
        }
        if client.choice == ApiChoice::SymbolSearch && !client.search_string.is_empty() {
            client.current_content = company_search(&client.search_string, &companies);
        }
        terminal.clear().unwrap();
        let current_search_string = client.search_string.clone();
        let current_content = client.current_content.clone();
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(3)
                    .constraints(
                        [
                            Constraint::Percentage(20), // Choice enum(company search, etc.)
                            Constraint::Percentage(20), // Search string
                            Constraint::Percentage(60), // Results
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let block1 = Block::default()
                    .title(client.all_choices())
                    .borders(Borders::ALL);
                f.render_widget(block1, chunks[0]);

                let block2 = Block::default().title("Search for:").borders(Borders::ALL);
                let paragraph1 = Paragraph::new(current_search_string)
                    .block(block2)
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                f.render_widget(paragraph1, chunks[1]);

                let block3 = Block::default().title("Results").borders(Borders::ALL);
                let paragraph2 = Paragraph::new(current_content)
                    .block(block3)
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                f.render_widget(paragraph2, chunks[2]);
            })
            .unwrap();
    }
    Ok(()) // break 했을 경우 result값을 리턴해야하기 때문
}

// cargo clippy에게 idiomatic을 물어볼때 그대로 따라하면 좋음
// 게다가 src\main.rs:179:43 이런 문구를 그대로 복사해서 ctrl-g 후에 붙여넣기로 들어가면 바로 진입
