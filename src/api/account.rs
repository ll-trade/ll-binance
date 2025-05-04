use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{ApiQuery, QueryType, trade::PositionSide};

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
    #[serde()]
    pub total_initial_margin: Decimal,
    #[serde(rename = "totalMaintMargin")]
    pub total_main_margin: Decimal,
    #[serde()]
    pub total_wallet_balance: Decimal,
    #[serde()]
    pub total_unrealized_profit: Decimal,
    #[serde()]
    pub total_margin_balance: Decimal,
    #[serde()]
    pub total_position_initial_margin: Decimal,
    #[serde()]
    pub total_open_order_initial_margin: Decimal,
    #[serde()]
    pub total_cross_wallet_balance: Decimal,
    #[serde()]
    pub total_cross_un_pnl: Decimal,
    #[serde()]
    pub available_balance: Decimal,
    #[serde()]
    pub max_withdraw_amount: Decimal,
    pub assets: Vec<Asset>,
    pub positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub asset: String,
    #[serde()]
    pub wallet_balance: Decimal,
    #[serde()]
    pub unrealized_profit: Decimal,
    #[serde()]
    pub margin_balance: Decimal,
    #[serde(rename = "maintMargin")]
    pub main_margin: Decimal,
    #[serde()]
    pub initial_margin: Decimal,
    #[serde()]
    pub position_initial_margin: Decimal,
    #[serde()]
    pub open_order_initial_margin: Decimal,
    #[serde()]
    pub cross_wallet_balance: Decimal,
    #[serde()]
    pub cross_un_pnl: Decimal,
    #[serde()]
    pub available_balance: Decimal,
    #[serde()]
    pub max_withdraw_amount: Decimal,
    pub update_time: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    pub position_side: PositionSide,
    #[serde()]
    pub position_amt: Decimal,
    #[serde()]
    pub unrealized_profit: Decimal,
    #[serde()]
    pub isolated_margin: Decimal,
    #[serde()]
    pub notional: Decimal,
    #[serde()]
    pub isolated_wallet: Decimal,
    #[serde()]
    pub initial_margin: Decimal,
    #[serde(rename = "maintMargin")]
    pub main_margin: Decimal,
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
    #[serde()]
    pub maker: Decimal,
    #[serde()]
    pub taker: Decimal,
    #[serde()]
    pub buyer: Decimal,
    #[serde()]
    pub seller: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub asset: String,
    #[serde()]
    pub free: Decimal,
    #[serde()]
    pub locked: Decimal,
}
