use serde::{Deserialize, Serialize};

use crate::{string_as_f64, FutureOrderType, OrderSide, OrderStatus, OrderType, TimeInForce};

use super::{ApiQuery, QueryType};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QPositionRisk {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl ApiQuery for QPositionRisk {
    type Response = Vec<PositionRisk>;

    const METHOD: &'static str = "v2/account.position";

    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionRisk {
    pub symbol: String,
    pub position_side: PositionSide,
    #[serde(deserialize_with = "string_as_f64")]
    pub position_amt: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub entry_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub break_even_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub mark_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub un_realized_profit: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub liquidation_price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub isolated_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub notional: f64,
    pub margin_asset: String,
    #[serde(deserialize_with = "string_as_f64")]
    pub isolated_wallet: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub initial_margin: f64,
    #[serde(rename = "maintMargin", deserialize_with = "string_as_f64")]
    pub main_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub position_initial_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub open_order_initial_margin: f64,
    pub adl: i64,
    #[serde(deserialize_with = "string_as_f64")]
    pub bid_notional: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub ask_notional: f64,
    pub update_time: i64,
}

impl ApiQuery for OrderSpec {
    type Response = CreateOrderResponse;
    const METHOD: &'static str = "order.place";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestOrderSpec {
    #[serde(flatten)]
    pub order: OrderSpec,
    pub compute_commission_rates: bool,
}

impl ApiQuery for TestOrderSpec {
    type Response = CreateOrderResponse;
    const METHOD: &'static str = "order.test";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QOrderStatus {
    pub symbol: String,
    pub order_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
}

impl ApiQuery for QOrderStatus {
    type Response = QueriedOrder;
    const METHOD: &'static str = "order.status";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrder {
    pub symbol: String,
    pub order_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_restrictions: Option<CancelRestrictions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancelRestrictions {
    OnlyNew,
    OnlyPartiallyFilled,
}

impl ApiQuery for CancelOrder {
    type Response = OrderResult;
    const METHOD: &'static str = "order.cancel";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureCancelOrder {
    pub symbol: String,
    pub order_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
}

impl ApiQuery for FutureCancelOrder {
    type Response = FutureOrderResult;
    const METHOD: &'static str = "order.cancel";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PositionSide {
    BOTH,
    LONG,
    SHORT,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FutureOrderSpec {
    pub symbol: String,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: FutureOrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_side: Option<PositionSide>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iceberg_qty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_mode: Option<String>,
}

impl ApiQuery for FutureOrderSpec {
    type Response = FutureOrderResult;
    const METHOD: &'static str = "order.place";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FutureOrderResult {
    /// 交易对
    pub symbol: String,
    /// 系统订单ID
    pub order_id: i64,
    /// 订单状态
    pub status: OrderStatus,
    /// 客户自己设置的ID
    pub client_order_id: String,

    #[serde(deserialize_with = "string_as_f64")]
    pub price: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub avg_price: f64,

    /// 用户设置的原始订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub orig_qty: f64,
    /// 交易的订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub cum_qty: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub cum_quote: f64,
    /// 订单的时效方式
    pub time_in_force: TimeInForce,
    /// 订单类型， 比如市价单，现价单等
    #[serde(rename = "type")]
    pub order_type: FutureOrderType,
    pub reduce_only: bool,
    pub close_position: bool,
    /// 订单方向
    pub side: OrderSide,
    pub position_side: PositionSide,
    #[serde(deserialize_with = "string_as_f64")]
    pub stop_price: f64,

    /// 交易时间戳
    pub update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OrderSpec {
    pub symbol: String,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_order_qty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_delta: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iceberg_qty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_type: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum CreateOrderResponse {
    Ack(OrderAck),
    Result(OrderResult),
    Full(OrderFull),
}

impl CreateOrderResponse {
    pub fn order_id(&self) -> i64 {
        match self {
            CreateOrderResponse::Ack(o) => o.order_id,
            CreateOrderResponse::Result(o) => o.order_id,
            CreateOrderResponse::Full(o) => o.order_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueriedOrder {
    /// 交易对
    pub symbol: String,
    /// 系统订单ID
    pub order_id: i64,
    /// OCO订单ID,否则为-1
    #[serde(default)]
    pub order_list_id: i64,
    /// 客户自己设置的ID
    pub client_order_id: String,
    /// 订单价格
    #[serde(deserialize_with = "string_as_f64")]
    pub price: f64,
    /// 订单价格
    #[serde(deserialize_with = "string_as_f64", default)]
    pub avg_price: f64,
    /// 用户设置的原始订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub orig_qty: f64,
    /// 交易的订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "string_as_f64", default)]
    pub cum_quote: f64,
    #[serde(default)]
    pub reduce_only: bool,
    #[serde(default)]
    pub close_position: bool,
    /// 累计交易的金额
    #[serde(deserialize_with = "string_as_f64", default)]
    pub cummulative_quote_qty: f64,
    /// 订单状态
    pub status: OrderStatus,
    /// 订单的时效方式
    pub time_in_force: TimeInForce,
    /// 订单类型， 比如市价单，现价单等
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// 订单方向
    pub side: OrderSide,
    /// 止损价格
    #[serde(deserialize_with = "string_as_f64")]
    pub stop_price: f64,
    /// 冰山数量
    #[serde(deserialize_with = "string_as_f64", default)]
    pub iceberg_qty: f64,
    /// 订单时间
    pub time: i64,
    /// 最后更新时间
    pub update_time: i64,
    /// 订单是否出现的 order book 中
    #[serde(default)]
    pub is_working: bool,
    /// 原始交易金额
    #[serde(deserialize_with = "string_as_f64", default)]
    pub orig_quote_order_qty: f64,
    pub self_trade_prevention_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderAck {
    /// 交易对
    pub symbol: String,
    /// 系统订单ID
    pub order_id: i64,
    /// OCO订单ID,否则为-1
    pub order_list_id: i64,
    /// 客户自己设置的ID
    pub client_order_id: String,
    /// 交易时间戳
    pub transact_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResult {
    /// 交易对
    pub symbol: String,
    /// 系统订单ID
    pub order_id: i64,
    /// OCO订单ID,否则为-1
    pub order_list_id: i64,
    /// 客户自己设置的ID
    pub client_order_id: String,
    /// 交易时间戳
    pub transact_time: i64,
    /// 订单价格
    #[serde(deserialize_with = "string_as_f64")]
    pub price: f64,
    /// 用户设置的原始订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub orig_qty: f64,
    /// 交易的订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub executed_qty: f64,
    /// 累计交易的金额
    #[serde(deserialize_with = "string_as_f64")]
    pub cummulative_quote_qty: f64,
    /// 订单状态
    pub status: OrderStatus,
    /// 订单的时效方式
    pub time_in_force: TimeInForce,
    /// 订单类型， 比如市价单，现价单等
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// 订单方向
    pub side: OrderSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFull {
    /// 交易对
    pub symbol: String,
    /// 系统订单ID
    pub order_id: i64,
    /// OCO订单ID,否则为-1
    pub order_list_id: i64,
    /// 客户自己设置的ID
    pub client_order_id: String,
    /// 交易时间戳
    pub transact_time: i64,
    /// 订单价格
    #[serde(deserialize_with = "string_as_f64")]
    pub price: f64,
    /// 用户设置的原始订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub orig_qty: f64,
    /// 交易的订单数量
    #[serde(deserialize_with = "string_as_f64")]
    pub executed_qty: f64,
    /// 累计交易的金额
    #[serde(deserialize_with = "string_as_f64")]
    pub cummulative_quote_qty: f64,
    /// 订单状态
    pub status: OrderStatus,
    /// 订单的时效方式
    pub time_in_force: TimeInForce,
    /// 订单类型， 比如市价单，现价单等
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// 订单方向
    pub side: OrderSide,
    /// 订单中交易的信息
    pub fills: Vec<OrderFill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFill {
    /// 交易的价格
    #[serde(deserialize_with = "string_as_f64")]
    pub price: f64,
    /// 交易的数量
    #[serde(deserialize_with = "string_as_f64")]
    pub qty: f64,
    /// 手续费金额
    #[serde(deserialize_with = "string_as_f64")]
    pub commission: f64,
    /// 手续费的币种
    pub commission_asset: String,
}
