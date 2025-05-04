use std::str::FromStr;

use serde::{Deserialize, Serialize};
pub use sproxy;
use time::OffsetDateTime;

/// <https://binance-docs.github.io/apidocs/websocket_api/cn/#45fa4e00db>
pub mod api;
pub mod realtime_market;
pub mod rest;

fn millis_ts() -> i64 {
    (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl FromStr for OrderSide {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "b" | "buy" => Ok(Self::Buy),
            "s" | "sell" => Ok(Self::Sell),
            _ => Err(format!("invalid {s}")),
        }
    }
}

impl OrderSide {
    pub fn rev(&self) -> Self {
        match self {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            OrderSide::Buy => true,
            OrderSide::Sell => false,
        }
    }
}

impl Default for OrderSide {
    fn default() -> Self {
        Self::Buy
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FutureOrderType {
    #[default]
    ///限价单
    Limit,
    ///市价单
    Market,
    ///止损单
    Stop,
    ///限价止损单
    StopMarket,
    ///止盈单
    TakeProfit,
    ///限价止盈单
    TakeProfitMarket,
    ///限价只挂单
    TailingStopMarket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    ///限价单
    Limit,
    ///市价单
    Market,
    ///止损单
    StopLoss,
    ///限价止损单
    StopLossLimit,
    ///止盈单
    TakeProfit,
    ///限价止盈单
    TakeProfitLimit,
    ///限价只挂单
    LimitMaker,
}

impl Default for OrderType {
    fn default() -> Self {
        Self::Limit
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeInForce {
    #[default]
    GTC,
    IOC,
    FOK,
    GTX,
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    #[default]
    /// 新建订单
    New,
    /// 部分成交
    PartiallyFilled,
    /// 全部成交
    Filled,
    /// 已撤销
    Canceled,
    /// 撤销中(目前并未使用)
    PendingCancel,
    /// 订单被拒绝
    Rejected,
    /// 订单过期(根据timeInForce参数规则)
    Expired,
    /// 表示订单由于 STP 触发而过期
    ExpiredInMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    PreTrading,
    Trading,
    PostTrading,
    EndOfDay,
    HALT,
    AuctionMatch,
    BREAK,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(non_snake_case)]
pub enum Permission {
    SPOT,
    MARGIN,
    LEVERAGED,
    TrdGrp002,
    TrdGrp003,
    TrdGrp004,
    TrdGrp005,
    TrdGrp006,
    TrdGrp007,
    TrdGrp008,
    TrdGrp009,
    TrdGrp010,
    TrdGrp011,
    TrdGrp012,
    TrdGrp013,
    TrdGrp014,
    TrdGrp015,
    TrdGrp016,
    TrdGrp017,
    TrdGrp018,
    TrdGrp019,
    TrdGrp020,
    TrdGrp021,
    TrdGrp022,
    TrdGrp023,
    TrdGrp024,
    TrdGrp025,
}
