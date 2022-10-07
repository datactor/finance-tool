#![allow(non_snake_case)]

use crossterm::event::{read, Event, KeyCode, KeyEvent};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::sync::{Mutex, MutexGuard};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    widgets::{Paragraph, Wrap},
    Terminal,
};

pub const API_KEY: &str = "input your keys";
// pub const API_KEY: &str = include_str!("..\\key.txt");

use lazy_static::lazy_static;

// 매크로 자체를 모름 공부할 것
lazy_static! {
    static ref CLIENT: FinanceClient = FinanceClient {
        url: "https://finnhub.io/api/v1/".to_string(),
        client: Client::default(),
        search_string: Mutex::new(String::new()),
        choice: Mutex::new(ApiChoice::CompanyInfo),
    };
}

struct FinanceClient {
    url: String,
    client: Client,
    search_string: Mutex<String>, // push + pop
    choice: Mutex<ApiChoice>,
}

impl FinanceClient {
    fn search_string(&self) -> String {
        self.search_string.lock().unwrap().to_string()
    }
    fn get_search_string(&self) -> MutexGuard<'_, String> {
        self.search_string.lock().unwrap()
    }
    fn switch(&self) {
        let current_choice = *self.choice.lock().unwrap();
        let new_choice = match current_choice {
            ApiChoice::SymbolSearch => ApiChoice::CompanyInfo,
            ApiChoice::CompanyInfo => ApiChoice::SymbolSearch,
        };
        *self.choice.lock().unwrap() = new_choice;
    }
}

#[derive(Debug, Clone, Copy)]
enum ApiChoice {
    SymbolSearch,
    CompanyInfo,
}

impl ApiChoice {
    fn switch(&self) -> Self {
        match self {
            ApiChoice::SymbolSearch => ApiChoice::CompanyInfo,
            ApiChoice::CompanyInfo => ApiChoice::SymbolSearch,
        }
    }
}

impl std::fmt::Display for ApiChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ApiChoice::*;
        let output = match self {
            SymbolSearch => "Company symbol",
            CompanyInfo => "Company info",
        };
        write!(f, "{}", output)
    }
}

/// Serialize = into JSON
///
/// Deserialize = into Rust type
#[derive(Debug, Serialize, Deserialize)]
struct CompanyInfo {
    country: String,
    currency: String,
    exchange: String,
    #[serde(rename = "finnhubIndustry")]
    industry: String,
    ipo: String, // chrono -> NaiveDate
    #[serde(rename = "marketCapitalization")] // deserializing to market_capitalization
    market_capitalization: f64,
    name: String,
    phone: String,
    #[serde(rename = "shareOutstanding")]
    shares_outstanding: f64,
    ticker: String,
    weburl: String,
}

impl std::fmt::Display for CompanyInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let CompanyInfo {
            country,
            currency,
            exchange,
            industry,
            ipo,
            market_capitalization,
            name,
            phone,
            shares_outstanding,
            ticker,
            weburl,
        } = self; // 왜?

        let company_info = format!(
            "
Company name: {name}
Country: {country}
Currency: {currency}
Exchange: {exchange}
Industry: {industry}
Ipo: {ipo}
Market capitalization: {market_capitalization}
Ticker: {ticker}
Shares: {shares_outstanding}
Phone: {phone}
Url: {weburl}
        "
        );
        write!(f, "{}", company_info) // 왜?
    }
}

impl FinanceClient {
    fn get_profile_by_symbol(&self) {
        let text = self
            .client
            .get(format!(
                "{}/stock/profile2?symbol={}",
                self.url,
                self.search_string()
            ))
            .header("X-Finnhub-Token", API_KEY)
            .send()
            .unwrap()
            .text()
            .unwrap();
        let company_info: CompanyInfo = serde_json::from_str(&text).unwrap();
        println!("Text: {company_info}");
    }
}

fn main() -> crossterm::Result<()> {
    // let mut client = FinanceClient {
    //     url: "https://finnhub.io/api/v1".to_string(),
    //     client: Client::default(),
    //     search_string: String::new(),
    // };
    //
    // let countered = std::rc::Rc::new(client);

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        match read().unwrap() {
            Event::Key(key_event) => {
                // println!("Got a KeyEvent: {key_event:?}");
                let KeyEvent {
                    code, modifiers, ..
                } = key_event;
                match (code, modifiers) {
                    (KeyCode::Char(c), _) => {
                        CLIENT.get_search_string().push(c);
                        // println!("{}", CLIENT.get_search_string());
                    }
                    (KeyCode::Esc, _) => {
                        CLIENT.get_search_string().clear();
                        // println!("{}", CLIENT.get_search_string());
                    }
                    (KeyCode::Backspace, _) => {
                        CLIENT.get_search_string().pop();
                        // println!("{}", CLIENT.get_search_string());
                    }
                    (KeyCode::Enter, _) => {
                        CLIENT.get_profile_by_symbol();
                        CLIENT.get_search_string().clear();
                    }
                    (KeyCode::Tab, _) => {
                        // CLIENT.choice.lock().unwrap().switch(); // impl로도 가능
                        CLIENT.switch();
                    }
                    (_, _) => {}
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(num1, num2) => {
                println!("Window has been resized to {num1}, {num2}")
            }
            _ => {}
        }
        // let cloned_client = std::rc::Rc::clone(&countered);
        // terminal.clear().unwrap();
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                    .split(f.size());
                let block1 = Block::default().title(format!("{}", CLIENT.choice.lock().unwrap())).borders(Borders::ALL);
                f.render_widget(block1, chunks[0]);
                let block2 = Block::default().title("Results").borders(Borders::ALL);
                f.render_widget(block2, chunks[1]);

                let paragraph = Paragraph::new(CLIENT.search_string())
                    .block(
                        Block::default()
                            .title("Search string")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                f.render_widget(paragraph, chunks[1]);
            })
            .unwrap();
    }
}
