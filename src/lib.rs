// 모듈이 포함되어 있거나 lib, main.rs파일이 아닌 경우 러스트가 컴파일 시도하지 않음
// --release로 릴리즈모드로 싫행하면 진짜 러스트의 속도를 볼 수 있음
// cargo tree로 모든 dependency 볼 수 있음
// (*) 표시가 있으면 중복이 제거된 것임. --duplicates 플래그로 중복 제거 가능
pub const API_KEY: &str = include_str!("../key.txt");

pub mod app {
    use std::{
        fmt::{Display, Formatter},
        str::FromStr,
    };

    use anyhow::{anyhow, Context, Error};
    use reqwest::blocking::Client;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use tui::{
        style::{Color, Modifier, Style},
        text::Span,
    };

    use crate::API_KEY;

    pub struct FinanceClient {
        pub url: String,
        pub client: Client,
        pub search_string: String,   // push + pop
        pub current_content: String, // Results etc. of searches
        pub choice: ApiChoice,
    }

    impl FinanceClient {
        pub fn finnhub_request<T: DeserializeOwned + Display>(
            // deserializeOwned?
            &self,
            url: String,
        ) -> Result<String, Error> {
            let response = self
                .client
                .get(url)
                .header("X-Finnhub-Token", API_KEY)
                .send() // anyhow Error를 추가하면 ?를 쓸 수 있다
                .with_context(|| "Couldn't send via client")?; // expect와 비슷한 역할
            let text = response.text().with_context(|| "No text for some reason")?;
            let finnhub_reply: T = serde_json::from_str(&text).with_context(|| {
                format!(
                    "Couldn't deserialize {} into CompanyInfo struct.\nText from Finnhub: '{text}'",
                    self.search_string
                )
            })?;
            Ok(finnhub_reply.to_string())
        }
        // todo! remove unwraps
        pub fn company_profile(&self) -> Result<String, Error> {
            let url = format!("{}/stock/profile2?symbol={}", self.url, self.search_string);
            let company_info = self.finnhub_request::<CompanyProfile>(url)?;
            Ok(company_info)
        }

        pub fn stock_symbol(&self) -> Result<String, Error> {
            todo!() // 조용히해라(표현식이 반환값이 없어도 에러 뜨지마라)
                    // /stock/symbol?exchange=US
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
                SymbolSearch => CompanyInfo,
                CompanyInfo => StockSymbol,
                StockSymbol => MarketNews,
                MarketNews => CompanyNews,
                CompanyNews => SymbolSearch,
            }
        }
        pub fn all_choices(&self) -> Vec<Span<'static>> {
            use ApiChoice::*; // SymbolSearch
            let choices = [
                SymbolSearch,
                CompanyInfo,
                StockSymbol,
                MarketNews,
                CompanyNews,
            ];
            let choices = choices.into_iter().map(|choice| choice.to_string());
            let mut even_odd = std::iter::repeat(true);
            // let choices = vec![format!("{}", SymbolSearch), format!("{}", CompanyInfo)];

            choices
                .into_iter()
                .map(|choice_string| {
                    let black = even_odd.next().unwrap();
                    let bg = if black { Color::Black } else { Color::DarkGray };
                    let current_choice = format!("{}", self.choice);
                    if choice_string == current_choice {
                        Span::styled(
                            format!("{choice_string} "),
                            Style::default()
                                .fg(Color::LightYellow)
                                .bg(Color::Blue)
                                .add_modifier(Modifier::UNDERLINED),
                        )
                    } else {
                        Span::styled(
                            format!("{choice_string} "),
                            Style::default().fg(Color::White).bg(bg),
                        )
                    }
                })
                .collect::<Vec<_>>()
        }
    }

    // strum -> 모르는 개념

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ApiChoice {
        SymbolSearch,
        CompanyInfo,
        StockSymbol,
        MarketNews,
        CompanyNews,
    }

    impl std::fmt::Display for ApiChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use ApiChoice::*;
            let output = match self {
                SymbolSearch => "Company symbol",
                CompanyInfo => "Company info",
                StockSymbol => "Stock symbol",
                MarketNews => "Market news",
                CompanyNews => "Company news",
            };
            write!(f, "{}", output)
        }
    }

    // strum을 쓰면 Enumiter를 쓸 수 있음
    // Todo!() probably delete this because it feels like overkill
    #[derive(Debug, Display, EnumIter, PartialEq, Eq)]
    pub enum ExchangeCodes {
        AS,
        AT,
        AX,
        BA,
        BC,
        BD,
        BE,
        BK,
        BO,
        BR,
        CA,
        CN,
        CO,
        CR,
        DB,
        DE,
        DU,
        F,
        HE,
        HK,
        HM,
        IC,
        IR,
        IS,
        JK,
        JO,
        KL,
        KQ,
        KS,
        L,
        LN,
        LS,
        MC,
        ME,
        MI,
        MU,
        MX,
        NE,
        NL,
        NS,
        NZ,
        OL,
        PA,
        PM,
        PR,
        QA,
        RG,
        SA,
        SG,
        SI,
        SN,
        SR,
        SS,
        ST,
        SW,
        SZ,
        T,
        TA,
        TL,
        TO,
        TW,
        TWO,
        US,
        V,
        VI,
        VN,
        VS,
        WA,
        HA,
        SX,
        TG,
        SC,
    }

    use strum::IntoEnumIterator;
    use strum_macros::{Display, EnumIter};

    impl FromStr for ExchangeCodes {
        type Err = anyhow::Error;

        // fileter_map 함수는 some이면 유지 none이면 버리기
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            ExchangeCodes::iter()
                .find(|code| &code.to_string() == s)
                .ok_or_else(|| anyhow!("Couldn't get ExchangeCode from {s}"))
        }
    }

    /// Serialize = into JSON
    ///
    /// Deserialize = into Rust type
    #[derive(Debug, Serialize, Deserialize)]
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
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
}

/// Structs add enums for the Finnhub API.
pub mod api {
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

    #[derive(Serialize, Deserialize)]
    struct StockSymbol {
        currency: String,
        description: String,
        #[serde(rename = "displaySymbol")]
        display_symbol: String,
        figi: String,
        mic: String,
        symbol: String,
        #[serde(rename = "type")]
        type_: String,
    }

    #[derive(Serialize, Deserialize)]
    struct MarketNews {
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

    #[derive(Serialize, Deserialize)]
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
