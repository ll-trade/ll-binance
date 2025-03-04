use serde::{Deserialize, Serialize};

use crate::string_as_f64;

use super::{trade::PositionSide, ApiQuery, QueryType};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QV2FutureAccountStatus {}

impl ApiQuery for QV2FutureAccountStatus {
    type Response = FutureAccountStatus;

    const METHOD: &'static str = "v2/account.status";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FutureAccountStatus {
    #[serde(deserialize_with = "string_as_f64")]
    pub total_initial_margin: f64,
    #[serde(rename = "totalMaintMargin", deserialize_with = "string_as_f64")]
    pub total_main_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub total_wallet_balance: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub total_unrealized_profit: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub total_margin_balance: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub total_position_initial_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub total_open_order_initial_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub total_cross_wallet_balance: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub total_cross_un_pnl: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub available_balance: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub max_withdraw_amount: f64,
    pub assets: Vec<Asset>,
    pub positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub asset: String,
    #[serde(deserialize_with = "string_as_f64")]
    pub wallet_balance: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub unrealized_profit: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub margin_balance: f64,
    #[serde(rename = "maintMargin", deserialize_with = "string_as_f64")]
    pub main_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub initial_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub position_initial_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub open_order_initial_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub cross_wallet_balance: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub cross_un_pnl: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub available_balance: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub max_withdraw_amount: f64,
    pub update_time: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    pub position_side: PositionSide,
    #[serde(deserialize_with = "string_as_f64")]
    pub position_amt: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub unrealized_profit: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub isolated_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub notional: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub isolated_wallet: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub initial_margin: f64,
    #[serde(rename = "maintMargin", deserialize_with = "string_as_f64")]
    pub main_margin: f64,
    pub update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QAccountStatus {
    pub omit_zero_balances: bool,
}

impl Default for QAccountStatus {
    fn default() -> Self {
        Self {
            omit_zero_balances: true,
        }
    }
}

impl ApiQuery for QAccountStatus {
    type Response = AccountStatus;
    const METHOD: &'static str = "account.status";
    const TYPE: QueryType = QueryType::AuthWithoutApiKey;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountStatus {
    pub maker_commission: usize,
    pub taker_commission: usize,
    pub buyer_commission: usize,
    pub seller_commission: usize,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub commission_rates: CommissionRate,
    pub brokered: bool,
    pub require_self_trade_prevention: bool,
    pub update_time: i64,
    pub account_type: String,
    pub balances: Vec<Balance>,
    pub permissions: Vec<String>,
    pub uid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRate {
    #[serde(deserialize_with = "string_as_f64")]
    pub maker: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub taker: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub buyer: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub seller: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub asset: String,
    #[serde(deserialize_with = "string_as_f64")]
    pub free: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub locked: f64,
}
