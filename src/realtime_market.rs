use std::{collections::HashSet, str::FromStr};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sproxy::ProxyConfig;
use tracing::debug;
use ws_tool::{
    ClientBuilder,
    codec::{StringCodec, StringRecv, StringSend},
    connector::{tcp_connect, wrap_rustls},
    errors::WsError,
    frame::OpCode,
    stream::{SyncStream, SyncStreamRead, SyncStreamWrite},
};

use crate::{OrderSide, OrderStatus, OrderType, TimeInForce, api::market::Kline};

pub const SPOT_MARKET_URL: &str = "wss://stream.binance.com:9443/stream";
pub const FUTURE_MARKET_URL: &str = "wss://fstream.binance.com/stream";

pub struct AutoReconnectMarketClient {
    req_id: u64,
    conn_fn: Box<dyn FnMut() -> Result<StringCodec<SyncStream>, WsError> + Send + Sync + 'static>,
    subscriptions: HashSet<String>,
    stream: Option<StringCodec<SyncStream>>,
}

impl AutoReconnectMarketClient {
    pub fn new(url: &'static str, max_retries: usize, proxy: Option<ProxyConfig>) -> Self {
        let mut try_times = 0;
        let conn_single = move || {
            let url = url::Url::parse(url).map_err(|e| WsError::InvalidUri(e.to_string()))?;
            let stream = match proxy.clone() {
                Some(config) => {
                    sproxy::create_conn(
                        &config,
                        url.host_str().unwrap().into(),
                        url.port().unwrap_or(443),
                    )?
                    .0
                }
                None => tcp_connect(&url.as_str().parse().unwrap())?,
            };
            let stream = wrap_rustls(stream, url.host_str().unwrap(), vec![])?;
            let stream = SyncStream::Rustls(stream);
            ClientBuilder::new().with_stream(
                url.as_str().parse().unwrap(),
                stream,
                StringCodec::check_fn,
            )
        };
        let conn_fn = move || loop {
            if try_times >= max_retries {
                return Err(WsError::ConnectionFailed(format!(
                    "try to connect over {} times",
                    max_retries
                )));
            }
            match conn_single() {
                Ok(s) => {
                    try_times = 0;
                    return Ok(s);
                }
                Err(e) => {
                    try_times += 1;
                    tracing::warn!("{e}");
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        };
        Self {
            req_id: 0,
            conn_fn: Box::new(conn_fn),
            subscriptions: Default::default(),
            stream: None,
        }
    }

    fn get_stream(&mut self) -> Result<&mut StringCodec<SyncStream>, WsError> {
        if self.stream.is_none() {
            debug!("create new stream ...");
            self.stream = Some((self.conn_fn)()?);
        }
        Ok(self.stream.as_mut().unwrap())
    }

    fn sub_without_retried(&mut self, params: Vec<String>) -> Result<(), WsError> {
        self.req_id += 1;
        let id = self.req_id;
        let req = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": params,
            "id": id
        });
        tracing::debug!("{}", &req);
        self.get_stream()?.send(&req.to_string())?;
        Ok(())
    }

    pub fn subscribe(&mut self, params: Vec<String>) -> Result<(), WsError> {
        self.req_id += 1;
        let id = self.req_id;
        self.subscriptions.extend(params.iter().cloned());
        let req = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": params,
            "id": id
        });
        tracing::debug!("{}", &req);
        let ori_stream = self.get_stream()?;
        match ori_stream.send(&req.to_string()) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("send subscribe {e}");
                self.stream = None;
                self.sub_without_retried(self.subscriptions.clone().into_iter().collect())
            }
        }
    }

    pub fn unsubscribe(&mut self, params: Vec<String>) -> Result<(), WsError> {
        self.req_id += 1;
        let id = self.req_id;
        for p in params.iter() {
            self.subscriptions.remove(p);
        }
        let req = serde_json::json!({
            "method": "UNSUBSCRIBE",
            "params": params,
            "id": id
        });
        tracing::debug!("{}", &req);
        let ori_stream = self.get_stream()?;
        match ori_stream.send(&req.to_string()) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::warn!("send subscribe {e}");
                self.stream = None;
                self.sub_without_retried(self.subscriptions.clone().into_iter().collect())
            }
        }
    }

    pub fn recv(&mut self) -> Result<String, WsError> {
        loop {
            match self
                .get_stream()?
                .receive()
                .map(|msg| (msg.data.to_string(), msg.code))
            {
                Ok((data, code)) => match code {
                    OpCode::Ping => {
                        tracing::debug!("recv server ping, pong back");
                        self.get_stream()?.pong(&data).ok();
                    }
                    OpCode::Text => break Ok(data),
                    _ => {}
                },
                Err(e) => {
                    tracing::warn!("recv message {e}");
                    self.stream = None;
                    self.sub_without_retried(self.subscriptions.clone().into_iter().collect())?;
                    let (data, code) = self
                        .get_stream()?
                        .receive()
                        .map(|msg| (msg.data.to_string(), msg.code))?;
                    match code {
                        OpCode::Ping => {
                            tracing::debug!("recv server ping, pong back");
                            self.get_stream()?.pong(&data).ok();
                        }
                        OpCode::Text => break Ok(data),
                        _ => {}
                    }
                }
            }
        }
    }
}

pub struct MarketClient {
    req_id: u64,
    reader: StringRecv<SyncStreamRead>,
    writer: StringSend<SyncStreamWrite>,
}

impl MarketClient {
    pub fn conn(url: &str, proxy: Option<ProxyConfig>) -> Result<Self, WsError> {
        let url = url::Url::parse(url).map_err(|e| WsError::InvalidUri(e.to_string()))?;
        let stream = match proxy {
            Some(config) => {
                sproxy::create_conn(
                    &config,
                    url.host_str().unwrap().into(),
                    url.port().unwrap_or(443),
                )?
                .0
            }
            None => tcp_connect(&url.as_str().parse().unwrap())?,
        };
        let stream = wrap_rustls(stream, url.host_str().unwrap(), vec![])?;
        let stream = SyncStream::Rustls(stream);
        let (reader, writer) = ClientBuilder::new()
            .with_stream(url.as_str().parse().unwrap(), stream, StringCodec::check_fn)?
            .split();
        Ok(Self {
            req_id: 0,
            reader,
            writer,
        })
    }

    pub fn subscribe(&mut self, params: Vec<String>) -> Result<(), WsError> {
        self.req_id += 1;
        let id = self.req_id;
        let req = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": params,
            "id": id
        });
        tracing::debug!("{}", &req);
        self.writer.send(&req.to_string())?;
        Ok(())
    }

    pub fn unsubscribe(&mut self, params: Vec<String>) -> Result<(), WsError> {
        self.req_id += 1;
        let id = self.req_id;
        let req = serde_json::json!({
            "method": "UNSUBSCRIBE",
            "params": params,
            "id": id
        });
        tracing::debug!("{}", &req);
        self.writer.send(&req.to_string())?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<String, WsError> {
        loop {
            let msg = self.reader.receive()?;
            match msg.code {
                OpCode::Ping => {
                    tracing::debug!("recv server ping, pong back");
                    self.writer.pong(&msg.data)?;
                }
                OpCode::Text => break Ok(msg.data.to_string()),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub enum Interval {
    #[serde(rename = "1s")]
    Sec1,
    #[default]
    #[serde(rename = "1m")]
    Min1,
    #[serde(rename = "3m")]
    Min3,
    #[serde(rename = "5m")]
    Min5,
    #[serde(rename = "15m")]
    Min15,
    #[serde(rename = "30m")]
    Min30,
    #[serde(rename = "1h")]
    Hour1,
    #[serde(rename = "2h")]
    Hour2,
    #[serde(rename = "4h")]
    Hour4,
    #[serde(rename = "6h")]
    Hour6,
    #[serde(rename = "8h")]
    Hour8,
    #[serde(rename = "12h")]
    Hour12,
    #[serde(rename = "1d")]
    Day1,
    #[serde(rename = "3d")]
    Day3,
    #[serde(rename = "1w")]
    Week1,
    #[serde(rename = "1M")]
    Month1,
}

impl FromStr for Interval {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i = match s {
            "1s" => Self::Sec1,
            "1m" => Self::Min1,
            "3m" => Self::Min3,
            "5m" => Self::Min5,
            "15m" => Self::Min15,
            "30m" => Self::Min30,
            "1h" => Self::Hour1,
            "2h" => Self::Hour2,
            "4h" => Self::Hour4,
            "6h" => Self::Hour6,
            "8h" => Self::Hour8,
            "12h" => Self::Hour12,
            "1d" => Self::Day1,
            "3d" => Self::Day3,
            "1w" => Self::Week1,
            "1M" => Self::Month1,
            _ => return Err(format!("invalid interval {s}")),
        };
        Ok(i)
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Sec1 => "1s",
            Self::Min1 => "1m",
            Self::Min3 => "3m",
            Self::Min5 => "5m",
            Self::Min15 => "15m",
            Self::Min30 => "30m",
            Self::Hour1 => "1h",
            Self::Hour2 => "2h",
            Self::Hour4 => "4h",
            Self::Hour6 => "6h",
            Self::Hour8 => "8h",
            Self::Hour12 => "12h",
            Self::Day1 => "1d",
            Self::Day3 => "3d",
            Self::Week1 => "1w",
            Self::Month1 => "1M",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventData<T> {
    /// 事件类型 24hrMiniTicker
    #[serde(rename = "e")]
    pub event_type: String,
    /// 事件时间(ms)
    #[serde(rename = "E")]
    pub event_time: i64,
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KData {
    /// 这根K线的起始时间
    #[serde(rename = "t")]
    pub open_time: i64,
    /// 这根K线的结束时间
    #[serde(rename = "T")]
    pub close_time: i64,
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    /// K线间隔
    #[serde(rename = "i")]
    pub interval: Interval,
    /// 这根K线期间第一笔成交ID
    #[serde(rename = "f")]
    pub first_id: i64,
    /// 这根K线期间末一笔成交ID
    #[serde(rename = "L")]
    pub last_id: i64,
    /// 这根K线期间第一笔成交价
    #[serde(rename = "o")]
    pub open: Decimal,
    /// 这根K线期间末一笔成交价
    #[serde(rename = "c")]
    pub close: Decimal,
    /// 这根K线期间最高成交价
    #[serde(rename = "h")]
    pub high: Decimal,
    /// 这根K线期间最低成交价
    #[serde(rename = "l")]
    pub low: Decimal,
    /// 这根K线期间成交量
    #[serde(rename = "v")]
    pub volume: Decimal,
    /// 这根K线期间成交笔数
    #[serde(rename = "n")]
    pub trade_num: u64,
    /// 这根K线是否完结(是否已经开始下一根K线)
    #[serde(rename = "x")]
    pub is_end: bool,
    /// 这根K线期间成交额
    #[serde(rename = "q")]
    pub qty: Decimal,
    /// 主动买入的成交量
    #[serde(rename = "V")]
    pub take_volume: Decimal,
    /// 主动买入的成交额
    #[serde(rename = "Q")]
    pub take_qty: Decimal,
    /// 忽略此参数
    #[serde(rename = "B")]
    pub __ignore: String,
}

impl From<Kline> for KData {
    fn from(value: Kline) -> Self {
        let Kline {
            open_time,
            open,
            high,
            low,
            close,
            volume,
            close_time,
            amount,
            count,
            buy_volume,
            buy_amount,
        } = value;
        KData {
            open_time,
            close_time,
            symbol: Default::default(),
            interval: Interval::Min1,
            first_id: 0,
            last_id: 0,
            open,
            close,
            high,
            low,
            volume,
            trade_num: count as u64,
            is_end: true,
            qty: amount,
            take_volume: buy_volume,
            take_qty: buy_amount,
            __ignore: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    /// 事件时间(ms)
    #[serde(rename = "E")]
    pub event_time: i64,
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub data: KData,
}

impl KlineEvent {
    pub fn params(symbols: Vec<(String, Interval)>) -> Vec<String> {
        symbols
            .into_iter()
            .map(|(s, i)| format!("{}@kline_{}", s.to_lowercase(), i))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BookTickerEvent {
    #[serde(rename = "u")]
    pub update_id: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "b")]
    pub buy_price: Decimal,
    #[serde(rename = "B")]
    pub buy_qty: Decimal,
    #[serde(rename = "a")]
    pub sell_price: Decimal,
    #[serde(rename = "A")]
    pub sell_qty: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiniTicker {
    /// 事件类型 24hrMiniTicker
    #[serde(rename = "e")]
    pub event_type: String,
    /// 事件时间(ms)
    #[serde(rename = "E")]
    pub event_time: i64,
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    /// 最新成交价格
    #[serde(rename = "c")]
    pub price_last_trade: Decimal,
    /// 24小时前开始第一笔成交价格
    #[serde(rename = "o")]
    pub price_24h_first_trade: Decimal,
    /// 24小时内最高成交价
    #[serde(rename = "h")]
    pub high: Decimal,
    /// 24小时内最低成交价
    #[serde(rename = "l")]
    pub low: Decimal,
    /// 成交量
    #[serde(rename = "v")]
    pub volume: Decimal,
    /// 成交额
    #[serde(rename = "q")]
    pub amount: Decimal,
}

impl MiniTicker {
    pub fn params(symbols: Vec<String>) -> Vec<String> {
        symbols
            .into_iter()
            .map(|s| format!("{s}@miniTicker"))
            .collect()
    }

    pub fn check_stream(stream: &str) -> bool {
        stream.ends_with("miniTicker")
    }

    pub fn check_event(event: &str) -> bool {
        event == "24hrMiniTicker"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamData {
    pub stream: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "e")]
pub enum UserStreamData {
    ListenKeyExpired(ListenKeyExpired),
    OutboundAccountPosition(OutboundAccountPosition),
    BalanceUpdate(Box<BalanceUpdated>),
    ExecutionReport(Box<ExecutionReport>),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKeyExpired {
    /// 事件推送时间
    #[serde(rename = "E")]
    pub event_time: i64,
    pub listen_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundAccountPosition {
    /// 事件推送时间
    #[serde(rename = "E")]
    pub event_time: i64,
    /// 事件推送时间
    #[serde(rename = "u")]
    pub update_time: i64,
    pub balances: Vec<UserStreamBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStreamBalance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "f")]
    pub free: Decimal,
    #[serde(rename = "l")]
    pub lock: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdated {
    /// 事件推送时间
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "d")]
    pub delta: Decimal,
    #[serde(rename = "T")]
    pub clear_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// 事件推送时间
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "o")]
    pub order_type: OrderType,
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "q")]
    pub origin_num: Decimal,
    #[serde(rename = "p")]
    pub origin_price: Decimal,
    #[serde(rename = "P")]
    pub stop_price: Decimal,
    #[serde(rename = "F")]
    pub iceberg_num: Decimal,
    #[serde(rename = "g")]
    pub order_list_id: i64,
    #[serde(rename = "x")]
    pub exec_type: String,
    #[serde(rename = "X")]
    pub order_status: OrderStatus,
    #[serde(rename = "r")]
    pub reject_type: String,
    #[serde(rename = "i")]
    pub order_id: i64,
    #[serde(rename = "l")]
    pub free: Decimal,
    #[serde(rename = "Y")]
    pub free_amount: Decimal,
    #[serde(rename = "z")]
    pub fill: Decimal,
    #[serde(rename = "Z")]
    pub fill_amount: Decimal,
    #[serde(rename = "L")]
    pub last_trade_price: Decimal,
    #[serde(rename = "n")]
    pub commission: Decimal,
    #[serde(rename = "N")]
    pub commission_type: Option<String>,
    #[serde(rename = "T")]
    pub trade_time: i64,
    #[serde(rename = "t")]
    pub trade_id: i64,
    #[serde(rename = "w")]
    pub on_book: bool,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "O")]
    pub create_time: i64,
}
