#![allow(non_snake_case)]

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use finance::app::{ApiChoice, FinanceClient};
use reqwest::blocking::Client;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub const API_KEY: &str = include_str!("../key.txt");

enum Market {}

/// todo! Make into real error
enum ClientError {
    IncorrectInput,
}

// 1 Make it work
// 2 Make it nice
// 3 Make it fast

// Rust 1.63
// Global variable
// static SOMETHING: Mutex<String> = Mutex::new(String::new());

// const COMPANY_STR: &str = include_str!("../company_symbols.json");

// #[derive(Debug, Deserialize)]
// struct CompanySymbolInfo((String, String));

// todo! Devide into four
// 1) init -> finance client를 defalt로 하고
// 2) read
// 3) update
// 4) draw

fn main() -> Result<(), anyhow::Error> {
    // 1. init

    // hashmap으로 검색하려면 키가 맞아야하기때문에 string의 일부로 검색하기 부적합
    // let companies: HashMap<&str, &str> = serde_json::from_str(COMPANY_STR).unwrap();
    // let companies = serde_json::from_str::<BTreeMap<&str, &str>>(COMPANY_STR)
    //     .unwrap()
    //     .into_iter()
    //     .map(|(key, value)| (key, value))
    //     .collect::<Vec<(_, _)>>();

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client = FinanceClient {
        url: "https://finnhub.io/api/v1/".to_string(),
        client: Client::default(),
        search_string: String::new(),
        current_content: String::new(),
        choice: ApiChoice::CompanyProfile,
        current_market: "US".to_string(),
        companies: Vec::new(), // stocksymbols에서 US market을 default로
    };

    let stock_symbols = client.stock_symbols()?;
    client.companies = stock_symbols
        .into_iter()
        .map(|info| (info.description, info.display_symbol))
        .collect::<Vec<(String, String)>>();
    // let company_symbols = stock_symbols
    //     .into_iter()
    //     .map(|info| (info.description, info.display_symbol))
    //     .collect::<Vec<(String, String)>>();
    // println!("{stock_symbols:#?}");

    // Input
    // State change / enum, char+, char-
    // Draw

    loop {
        // 2. read
        match read().unwrap() {
            Event::Key(key_event) => {
                // println!("Got a KeyEvent: {key_event:?}");
                let KeyEvent {
                    code, modifiers, ..
                } = key_event;
                // Typing event
                // 3. update by keycodes
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
                        match client.choice {
                            ApiChoice::CompanyProfile => {
                                client.current_content = match client.company_profile() {
                                    Ok(search_result) => search_result.to_string(),
                                    Err(e) => e.to_string(),
                                }
                            }
                            ApiChoice::GetMarket => {
                                client.current_content = client.choose_market();
                            }
                            _ => {}
                        }
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
            client.current_content = client.company_search(&client.search_string);
        }
        terminal.clear().unwrap();
        let current_search_string = client.search_string.clone();
        let current_content = client.current_content.clone();

        // 4. draw
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
