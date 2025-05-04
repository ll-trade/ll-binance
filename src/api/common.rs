use super::ApiQuery;
use super::EmptyResponse;
use super::QueryType;
use super::RateLimit;
use rust_decimal::Decimal;
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
        #[serde(rename = "minNotional")]
        min_notional: Decimal,
        #[serde(rename = "applyMinToMarket")]
        apply_min_to_market: bool,
        #[serde(rename = "maxNotional")]
        max_notional: Decimal,
        #[serde(rename = "applyMaxToMarket")]
        apply_max_to_market: bool,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: Decimal,
    },
    #[serde(rename = "PERCENT_PRICE_BY_SIDE")]
    PercentPriceBySide {
        #[serde(rename = "bidMultiplierUp")]
        bid_multiplier_up: Decimal,
        #[serde(rename = "bidMultiplierDown")]
        bid_multiplier_down: Decimal,
        #[serde(rename = "askMultiplierUp")]
        ask_multiplier_up: Decimal,
        #[serde(rename = "askMultiplierDown")]
        ask_multiplier_down: Decimal,
        #[serde(rename = "avgPriceMins")]
        avg_price_mins: i64,
    },
    #[serde(rename = "POSITION_RISK_CONTROL")]
    PositionRiskControl {
        #[serde(rename = "positionControlSide")]
        position_control_side: String,
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
        #[serde(rename = "minPrice")]
        min_price: Decimal,
        #[serde(rename = "maxPrice")]
        max_price: Decimal,
        #[serde(rename = "tickSize")]
        tick_size: Decimal,
    },
    #[serde(rename = "PERCENT_PRICE")]
    PercentPrice {
        #[serde(rename = "multiplierDown")]
        multiplier_down: Decimal,
        #[serde(rename = "multiplierUp")]
        multiplier_up: Decimal,
        #[serde(rename = "multiplierDecimal")]
        multiplier_decimal: Decimal,
    },
    #[serde(rename = "LOT_SIZE")]
    LOTSize {
        #[serde(rename = "stepSize")]
        step_size: Decimal,
        #[serde(rename = "maxQty")]
        max_qty: Decimal,
        #[serde(rename = "minQty")]
        min_qty: Decimal,
    },
    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional {
        #[serde()]
        notional: Decimal,
    },
    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts { limit: usize },
    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLOTSize {
        #[serde(rename = "stepSize")]
        step_size: Decimal,
        #[serde(rename = "maxQty")]
        max_qty: Decimal,
        #[serde(rename = "minQty")]
        min_qty: Decimal,
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
        #[serde()]
        #[serde(rename = "maxPosition")]
        max_position: Decimal,
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
