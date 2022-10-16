// 모듈이 포함되어 있거나 lib, main.rs파일이 아닌 경우 러스트가 컴파일 시도하지 않음
// --release로 릴리즈모드로 싫행하면 진짜 러스트의 속도를 볼 수 있음
// cargo tree로 모든 dependency 볼 수 있음
// (*) 표시가 있으면 중복이 제거된 것임. --duplicates 플래그로 중복 제거 가능
pub const API_KEY: &str = include_str!("../key.txt");

// market code (ex. DJI)
pub const EXCHANGE_CODES: [&str; 72] = [
    "AS", "AT", "AX", "BA", "BC", "BD", "BE", "BK", "BO", "BR", "CA", "CN", "CO", "CR", "DB", "DE",
    "DU", "F", "HE", "HK", "HM", "IC", "IR", "IS", "JK", "JO", "KL", "KQ", "KS", "L", "LN", "LS",
    "MC", "ME", "MI", "MU", "MX", "NE", "NL", "NS", "NZ", "OL", "PA", "PM", "PR", "QA", "RG", "SA",
    "SG", "SI", "SN", "SR", "SS", "ST", "SW", "SZ", "T", "TA", "TL", "TO", "TW", "TWO", "US", "V",
    "VI", "VN", "VS", "WA", "HA", "SX", "TG", "SC",
];

pub mod app {
    use std::fmt::Debug;

    use anyhow::{Context, Error};
    use reqwest::blocking::Client;
    use serde::de::DeserializeOwned;
    use tui::{
        style::{Color, Modifier, Style},
        text::Span,
    };

    use crate::{
        api::{CompanyProfile, StockSymbol},
        API_KEY, EXCHANGE_CODES,
    };

    pub struct FinanceClient {
        pub url: String,
        pub client: Client,
        pub search_string: String,   // push + pop // MSFT
        pub current_content: String, // Results etc. of searches
        pub choice: ApiChoice,
        pub current_market: String,
        pub companies: Vec<(String, String)>,
    }

    /// Vec<StockSymbol>
    impl FinanceClient {
        pub fn single_request<T: DeserializeOwned + Debug>(&self, url: String) -> Result<T, Error> {
            let response = self
                .client
                .get(url)
                .header("X-Finnhub-Token", API_KEY)
                .send()
                .with_context(|| "Couldn't send via client")?;
            let text = response.text().with_context(|| "No text for some reason")?;

            let finnhub_reply: T = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {} into CompanyProfile struct.\nText from Finnhub: '{text}'",
                    self.search_string
                )
            })?;
            Ok(finnhub_reply)
        }

        pub fn multi_request<T: DeserializeOwned + Debug>(
            &self,
            url: String,
        ) -> Result<Vec<T>, Error> {
            let response = self
                .client
                .get(url)
                .header("X-Finnhub-Token", API_KEY)
                .send()
                .with_context(|| "Couldn't send via client")?;
            let text = response.text().with_context(|| "No text for some reason")?;

            let finnhub_reply: Vec<T> = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {} into CompanyProfile struct.\nText from Finnhub: '{text}'",
                    self.search_string
                )
            })?;
            Ok(finnhub_reply)
        }

        // 관용구 기억하기 needle in a haystack
        pub fn company_search(&self, needle: &str) -> String {
            self.companies
                .iter()
                .filter_map(|(company_name, company_symbol)| {
                    let needle = needle.to_lowercase();
                    let company_name = company_name.to_lowercase();
                    if company_name.contains(&needle) {
                        Some(format!("{}: {}\n", company_symbol, company_name))
                    } else {
                        None
                    }
                })
                .collect::<String>()
        }

        // todo! remove unwraps
        /// /stock/profile?symbol=AAPL
        pub fn company_profile(&self) -> Result<String, Error> {
            let url = format!("{}/stock/profile2?symbol={}", self.url, self.search_string);
            let company_info = self.single_request::<CompanyProfile>(url)?;
            Ok(company_info.to_string())
        }

        pub fn stock_symbols(&self) -> Result<Vec<StockSymbol>, Error> {
            //todo!() // 조용히해라(표현식이 반환값이 없어도 에러 뜨지마라)
            // /stock/symbol?exchange=US
            let url = format!("{}/stock/symbol?exchange={}", self.url, self.current_market);
            let stock_symbols = self.multi_request::<StockSymbol>(url)?;
            Ok(stock_symbols)
        }

        /// User hits enter, checks to see if market exists, if not, stay w
        pub fn choose_market(&mut self) -> String {
            match EXCHANGE_CODES
                .iter()
                .find(|code| **code == self.search_string)
            {
                // e.g. user types "US", which is valid
                Some(good_market_code) => {
                    self.current_market = good_market_code.to_string();
                    match self.stock_symbols() {
                        Ok(stock_symbols) => {
                            self.companies = stock_symbols
                                .into_iter()
                                .map(|info| (info.description, info.display_symbol))
                                .collect::<Vec<(String, String)>>();
                            format!(
                                "Successfully got company info from market {}",
                                self.current_market
                            )
                        }
                        Err(_) => {
                            format!("No market called {} exists", self.search_string)
                        }
                    }
                }
                // user types something that isn't a market
                None => format!("No market called {} exists", self.search_string),
            }
        }

        pub fn company_news(&self) -> Result<String, Error> {
            todo!() // /company-news?symbol=AAPL&from=2021-09-01&to=2021-09-09
        }

        pub fn market_news(&self) -> Result<String, Error> {
            todo!() // /news?category=general
            // enum Market {
            //     general,
            //     forex,
            //     crypto,
            //     merger
            // }
        }

        pub fn switch(&mut self) {
            use ApiChoice::*;
            self.choice = match self.choice {
                SymbolSearch => CompanyProfile,
                CompanyProfile => StockSymbol,
                StockSymbol => MarketNews,
                MarketNews => CompanyNews,
                CompanyNews => GetMarket,
                GetMarket => SymbolSearch,
            }
        }
        pub fn all_choices(&self) -> Vec<Span> {
            use ApiChoice::*; // SymbolSearch
            let choices = [
                SymbolSearch,
                CompanyProfile,
                StockSymbol,
                MarketNews,
                CompanyNews,
                GetMarket,
            ];
            let choices = choices.into_iter().map(|choice| choice.to_string());

            choices
                .into_iter()
                .map(|choice_string| {
                    let current_choice = format!("{}", self.choice);
                    if choice_string == current_choice {
                        Span::styled(
                            format!(" {choice_string} "),
                            Style::default()
                                .bg(Color::Gray)
                                .add_modifier(Modifier::UNDERLINED),
                        )
                    } else {
                        Span::styled(format!(" {choice_string} "), Style::default())
                    }
                })
                .collect::<Vec<_>>()
        }
    }

    // strum -> 모르는 개념

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ApiChoice {
        SymbolSearch,
        CompanyProfile,
        StockSymbol,
        MarketNews,
        CompanyNews,
        GetMarket,
    }

    impl std::fmt::Display for ApiChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use ApiChoice::*;
            let output = match self {
                SymbolSearch => "Company symbol",
                CompanyProfile => "Company info",
                StockSymbol => "Stock symbol",
                MarketNews => "News",
                CompanyNews => "Company news",
                GetMarket => "Get market",
            };
            write!(f, "{}", output)
        }
    }

    // strum을 쓰면 Enumiter를 쓸 수 있음
    // Todo!() probably delete this because it feels like overkill
    //     // fileter_map 함수는 some이면 유지 none이면 버리기

    // Serialize = into JSON
    //
    // Deserialize = into Rust type
}

/// Structs add enums for the Finnhub API.
pub mod api {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CompanyProfile {
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

    impl std::fmt::Display for CompanyProfile {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let CompanyProfile {
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

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct SymbolLookup {
        pub description: String,
        #[serde(rename = "displaySymbol")] // api의 카멜케이스를 러스트가 좋아하는 snakecase로
        pub display_symbol: String,
        pub symbol: String,
        // r#type: String, idiomatic하게 변경
        #[serde(rename = "type")]
        pub type_: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct StockSymbol {
        pub currency: String,
        pub description: String,
        #[serde(rename = "displaySymbol")]
        pub display_symbol: String,
        pub figi: String,
        pub mic: String,
        pub symbol: String,
        #[serde(rename = "type")]
        pub type_: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MarketNews {
        pub category: String,
        pub datetime: i64,
        pub headline: String,
        pub id: i64,
        pub image: String,
        pub related: String,
        pub source: String,
        pub summary: String,
        pub url: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CompanyNews {
        category: String,
        datetime: i64,
        headline: String,
        id: i64,
        image: String,
        related: String,
        source: String,
        summary: String,
        url: String,
    }
}

// Company Peers: Vec<String>

// {
//     "series": {
//      "annual": {
//        "currentRatio": [
//          {
//            "period": "2019-09-28",
//            "v": 1.5401
//          },
//          {
//            "period": "2018-09-29",
//            "v": 1.1329
//          }
//        ],
//        "salesPerShare": [
//          {
//            "period": "2019-09-28",
//            "v": 55.9645
//          },
//          {
//            "period": "2018-09-29",
//            "v": 53.1178
//          }
//        ],
//        "netMargin": [
//          {
//            "period": "2019-09-28",
//            "v": 0.2124
//          },
//          {
//            "period": "2018-09-29",
//            "v": 0.2241
//          }
//        ]
//      }
//    },
//    "metric": {
//      "10DayAverageTradingVolume": 32.50147,
//      "52WeekHigh": 310.43,
//      "52WeekLow": 149.22,
//      "52WeekLowDate": "2019-01-14",
//      "52WeekPriceReturnDaily": 101.96334,
//      "beta": 1.2989,
//    },
//    "metricType": "all",
//    "symbol": "AAPL"
//  }
