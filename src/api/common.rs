use super::string_as_f64;
use super::string_as_u64;
use super::ApiQuery;
use super::EmptyResponse;
use super::QueryType;
use super::RateLimit;
use serde::Deserialize;
use serde::Serialize;

/// 测试能否联通 WebSocket API
#[derive(Debug, Clone, Deserialize)]
pub struct Ping;

empty_serde!(Ping);

impl ApiQuery for Ping {
    type Response = EmptyResponse;
    const METHOD: &'static str = "ping";
    const TYPE: QueryType = QueryType::None;
}

/// 测试与 WebSocket API 的连通性并获取当前服务器时间
#[derive(Debug, Clone, Deserialize)]
pub struct QServerTime;

empty_serde!(QServerTime);

impl ApiQuery for QServerTime {
    type Response = ServerTime;
    const METHOD: &'static str = "time";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerTime {
    #[serde(rename = "serverTime")]
    pub time: i64,
}
/// 获取交易规则，速率限制，和交易对信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QExchangeInfo {
    #[serde(rename = "symbol")]
    Symbol(String),
    #[serde(rename = "symbols")]
    Symbols(Vec<String>),
    #[serde(rename = "permissions")]
    Perm(Vec<String>),
}

impl ApiQuery for QExchangeInfo {
    type Response = ExchangeInfo;
    const METHOD: &'static str = "exchangeInfo";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    pub exchange_filters: Vec<String>,
    pub rate_limits: Vec<RateLimit>,
    pub server_time: i64,
    pub symbols: Vec<ExSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExSymbol {
    /// 交易对
    pub symbol: String,
    /// 交易对状态
    pub status: String,
    /// 标的资产
    pub base_asset: String,
    /// 报价资产
    pub quote_asset: String,
    /// 保证金资产
    pub margin_asset: String,
    /// 标的资产精度
    pub price_precision: usize,
    pub quantity_precision: usize,
    #[serde(default)]
    pub base_asset_precision: usize,
    pub quote_precision: usize,
    #[serde(default)]
    pub filters: Vec<SymbolFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "filterType")]
pub enum SymbolFilter {
    #[serde(rename = "NOTIONAL")]
    Notional {
        #[serde(deserialize_with = "string_as_f64", rename = "minNotional")]
        min_notional: f64,
        #[serde(rename = "applyMinToMarket")]
        apply_min_to_market: bool,
        #[serde(deserialize_with = "string_as_f64", rename = "maxNotional")]
        max_notional: f64,
        #[serde(rename = "applyMaxToMarket")]
        apply_max_to_market: bool,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: i64,
    },
    #[serde(rename = "PERCENT_PRICE_BY_SIDE")]
    PercentPriceBySide {
        #[serde(deserialize_with = "string_as_f64", rename = "bidMultiplierUp")]
        bid_multiplier_up: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "bidMultiplierDown")]
        bid_multiplier_down: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "askMultiplierUp")]
        ask_multiplier_up: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "askMultiplierDown")]
        ask_multiplier_down: f64,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: i64,
    },
    #[serde(rename = "TRAILING_DELTA")]
    TrailingDelta {
        #[serde(rename = "minTrailingAboveDelta")]
        min_trailing_above_delta: i64,
        #[serde(rename = "maxTrailingAboveDelta")]
        max_trailing_above_delta: i64,
        #[serde(rename = "minTrailingBelowDelta")]
        min_trailing_below_delta: i64,
        #[serde(rename = "maxTrailingBelowDelta")]
        max_trailing_below_delta: i64,
    },

    #[serde(rename = "PRICE_FILTER")]
    PriceFilter {
        #[serde(deserialize_with = "string_as_f64", rename = "minPrice")]
        min_price: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "maxPrice")]
        max_price: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "tickSize")]
        tick_size: f64,
    },
    #[serde(rename = "PERCENT_PRICE")]
    PercentPrice {
        #[serde(deserialize_with = "string_as_f64", rename = "multiplierDown")]
        multiplier_down: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "multiplierUp")]
        multiplier_up: f64,
        #[serde(deserialize_with = "string_as_u64", rename = "multiplierDecimal")]
        multiplier_decimal: u64,
    },
    #[serde(rename = "LOT_SIZE")]
    LOTSize {
        #[serde(deserialize_with = "string_as_f64", rename = "stepSize")]
        step_size: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "maxQty")]
        max_qty: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "minQty")]
        min_qty: f64,
    },
    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional {
        #[serde(deserialize_with = "string_as_f64")]
        notional: f64,
    },
    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts { limit: usize },
    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLOTSize {
        #[serde(deserialize_with = "string_as_f64", rename = "stepSize")]
        step_size: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "maxQty")]
        max_qty: f64,
        #[serde(deserialize_with = "string_as_f64", rename = "minQty")]
        min_qty: f64,
    },
    #[serde(rename = "MAX_NUM_ORDERS")]
    MaxNumOrders { limit: usize },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    MaxNumAlgoOrders { limit: usize },
    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS")]
    MaxNumIcebergOrders {
        #[serde(rename = "maxNumIcebergOrders")]
        max_num_iceberg_orders: usize,
    },
    #[serde(rename = "MAX_POSITION")]
    MaxPosition {
        #[serde(deserialize_with = "string_as_f64")]
        #[serde(rename = "maxPosition")]
        max_position: f64,
    },
    #[serde(rename = "EXCHANGE_MAX_NUM_ORDERS")]
    ExchangeMaxNumOrders {
        #[serde(rename = "maxNumOrders")]
        max_num_orders: usize,
    },
    #[serde(rename = "EXCHANGE_MAX_ALGO_ORDERS")]
    ExchangeMaxAlgoOrders {
        #[serde(rename = "maxNumAlgoOrders")]
        max_num_algo_orders: usize,
    },
}
