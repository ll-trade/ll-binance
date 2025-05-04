use std::fmt;

use rust_decimal::Decimal;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{SeqAccess, Unexpected, Visitor},
};

use crate::realtime_market::Interval;

use super::{ApiQuery, QueryType};

/// 最新价格
#[derive(Debug, Clone, Deserialize)]
pub enum QLatestPrice {
    #[serde(rename = "symbol")]
    Symbol(String),
    #[serde(rename = "symbols")]
    Symbols(String),
    All,
}

impl Serialize for QLatestPrice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        match self {
            QLatestPrice::Symbol(s) => {
                map.serialize_entry("symbol", s)?;
            }
            QLatestPrice::Symbols(symbols) => {
                map.serialize_entry("symbols", symbols)?;
            }
            QLatestPrice::All => {}
        }
        map.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LatestPrices {
    Single(LatestPrice),
    Many(Vec<LatestPrice>),
}

impl LatestPrices {
    pub fn to_list(self) -> Vec<LatestPrice> {
        match self {
            LatestPrices::Single(i) => vec![i],
            LatestPrices::Many(v) => v,
        }
    }
}

impl ApiQuery for QLatestPrice {
    type Response = LatestPrices;
    const METHOD: &'static str = "ticker.price";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestPrice {
    pub symbol: String,
    #[serde()]
    pub price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QDepth {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Depth {
    pub last_update_id: usize,
    /// 买单
    pub bids: Vec<PriceVol>,
    /// 卖单
    pub asks: Vec<PriceVol>,
}

/// (价格, 数量)
#[derive(Debug)]
pub struct PriceVol(pub Decimal, pub Decimal);

impl<'de> Deserialize<'de> for PriceVol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, DepthOrderVisitor)
    }
}

struct DepthOrderVisitor;

impl<'de> Visitor<'de> for DepthOrderVisitor {
    type Value = PriceVol;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple of (String, String)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let first: &'de str = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"first element"))?;
        let first_val = first.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(first), &"first element")
        })?;
        let second: &'de str = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"first element"))?;
        let second_val = second.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(second), &"first element")
        })?;
        Ok(PriceVol(first_val, second_val))
    }
}

impl ApiQuery for QDepth {
    type Response = Depth;
    const METHOD: &'static str = "depth";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRecentTrade {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeRecord {
    pub id: usize,
    #[serde()]
    pub price: Decimal,
    #[serde()]
    pub qty: Decimal,
    #[serde()]
    pub quote_qty: Decimal,
    /// 交易成交时间, 和websocket中的T一致
    pub time: i64,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

impl ApiQuery for QRecentTrade {
    type Response = Vec<TradeRecord>;
    const METHOD: &'static str = "trades.recent";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QHistoryTrade {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_id: Option<i64>,
}

impl ApiQuery for QHistoryTrade {
    type Response = Vec<TradeRecord>;
    const METHOD: &'static str = "trades.historical";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QAggTrade {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggTrade {
    #[serde(rename = "a")]
    pub agg_trade_id: i64,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub qty: Decimal,
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    #[serde(rename = "l")]
    pub last_trade_id: i64,
    #[serde(rename = "T")]
    pub time: i64,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QKline {
    pub symbol: String,
    pub interval: Interval,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Kline {
    pub open_time: i64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub close_time: i64,
    pub amount: Decimal,
    pub count: usize,
    pub buy_volume: Decimal,
    pub buy_amount: Decimal,
}

impl<'de> Deserialize<'de> for Kline {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(12, KlineVisitor)
    }
}

struct KlineVisitor;

impl<'de> Visitor<'de> for KlineVisitor {
    type Value = Kline;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple of (i64, String, String, String, String, String, i64, String, usize, String, String, String)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let open_time: i64 = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_value(Unexpected::Option, &"expect open time")
        })?;
        let open_str: &'de str = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"open price"))?;
        let open = open_str.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(open_str), &"f64 string")
        })?;
        let high_str: &'de str = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"highest price"))?;
        let high = high_str.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(high_str), &"f64 string")
        })?;
        let low_str: &'de str = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"lowest price"))?;
        let low = low_str.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(low_str), &"f64 string")
        })?;
        let close_str: &'de str = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"close price"))?;
        let close = close_str.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(close_str), &"f64 string")
        })?;
        let vol_str = seq.next_element::<&'de str>()?.ok_or_else(|| {
            serde::de::Error::invalid_value(Unexpected::Option, &"expect volumne")
        })?;
        let volume = vol_str.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(close_str), &"f64 string")
        })?;
        let close_time: i64 = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_value(Unexpected::Option, &"expect close time")
        })?;
        let amount = seq
            .next_element::<&'de str>()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"amount"))?;
        let amount = amount.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(close_str), &"f64 string")
        })?;
        let count: usize = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_value(Unexpected::Option, &"expect close time")
        })?;
        let buy_volume = seq
            .next_element::<&'de str>()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"amount"))?;
        let buy_volume = buy_volume.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(close_str), &"f64 string")
        })?;
        let buy_amount = seq
            .next_element::<&'de str>()?
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Option, &"amount"))?;
        let buy_amount = buy_amount.parse::<Decimal>().map_err(|_| {
            serde::de::Error::invalid_value(Unexpected::Str(close_str), &"f64 string")
        })?;
        for _ in 0..1 {
            seq.next_element::<&'de str>()?.ok_or_else(|| {
                serde::de::Error::invalid_value(Unexpected::Option, &"expect ignored padded field")
            })?;
        }
        Ok(Kline {
            open_time,
            open,
            high,
            low,
            close,
            close_time,
            count,
            volume,
            amount,
            buy_volume,
            buy_amount,
        })
    }
}

impl ApiQuery for QKline {
    type Response = Vec<Kline>;
    const METHOD: &'static str = "klines";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QUIKline {
    pub symbol: String,
    pub interval: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl ApiQuery for QUIKline {
    type Response = Vec<Kline>;
    const METHOD: &'static str = "uiKlines";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QAvgPrice {
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AvgPrice {
    pub min: i64,
    #[serde()]
    pub price: Decimal,
    pub close_time: i64,
}

impl ApiQuery for QAvgPrice {
    type Response = AvgPrice;
    const METHOD: &'static str = "avgPrice";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum QMiniTicker {
    Single {
        symbol: String,
        #[serde(rename = "type")]
        ty: MiniTickerType,
    },
    Many {
        symbols: Vec<String>,
        #[serde(rename = "type")]
        ty: MiniTickerType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MiniTickerType {
    FULL,
    MINI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H24Ticker {
    pub symbol: String,
    #[serde(default)]
    pub price_change: Decimal,
    #[serde(default)]
    pub price_change_percent: Decimal,
    #[serde(default)]
    pub weighted_avg_price: Decimal,
    #[serde()]
    pub last_price: Decimal,
    #[serde(default)]
    pub last_qty: Decimal,
    #[serde(default)]
    pub bid_price: Decimal,
    #[serde(default)]
    pub bid_qty: Decimal,
    #[serde(default)]
    pub ask_price: Decimal,
    #[serde(default)]
    pub ask_qty: Decimal,
    #[serde()]
    pub open_price: Decimal,
    #[serde()]
    pub high_price: Decimal,
    #[serde()]
    pub low_price: Decimal,
    #[serde()]
    pub volume: Decimal,
    #[serde()]
    pub quote_volume: Decimal,
    pub open_time: i64,
    pub close_time: i64,
    pub first_id: i64,
    pub last_id: i64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "untagged")]
pub enum H24TickerResult {
    Single(H24Ticker),
    Many(Vec<H24Ticker>),
}

impl ApiQuery for QMiniTicker {
    type Response = H24TickerResult;
    const METHOD: &'static str = "ticker.24hr";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QSingleTickerBook {
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleTickerBook {
    pub last_update_id: i64,
    pub symbol: String,
    #[serde()]
    pub ask_price: Decimal,
    #[serde()]
    pub ask_qty: Decimal,
    #[serde()]
    pub bid_price: Decimal,
    #[serde()]
    pub bid_qty: Decimal,
    pub time: i64,
}

impl ApiQuery for QSingleTickerBook {
    type Response = SingleTickerBook;
    const METHOD: &'static str = "ticker.book";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QAllTickerBook {}

impl ApiQuery for QAllTickerBook {
    type Response = Vec<SingleTickerBook>;
    const METHOD: &'static str = "ticker.book";
    const TYPE: QueryType = QueryType::None;
}
