use std::ops::Not;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    api::{common::ExchangeInfo, market::Kline},
    realtime_market::Interval,
};

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
    pub limit: Option<i64>,
}

fn init_http_client() -> reqwest::blocking::Client {
    let mut builder = reqwest::blocking::Client::builder();
    if let Ok(p) = std::env::var("HTTPS_PROXY") {
        builder = builder.proxy(reqwest::Proxy::https(p).unwrap())
    }
    builder.build().unwrap()
}

pub fn future_kline_line(param: QKline) -> std::io::Result<Vec<Kline>> {
    let client = init_http_client();
    let qs_str = serde_qs::to_string(&param).unwrap();
    let resp = client
        .get(format!("https://fapi.binance.com/fapi/v1/klines?{qs_str}"))
        .send()
        .unwrap();
    if resp.status().is_success().not() {
        warn!("{:?}", resp.text());
        return Ok(vec![]);
    }
    let data = resp
        .json()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(data)
}

pub fn exchange_info() -> std::io::Result<ExchangeInfo> {
    let client = init_http_client();
    let resp = client
        .get(format!("https://fapi.binance.com/fapi/v1/exchangeInfo"))
        .send()
        .unwrap();
    if resp.status().is_success().not() {
        let resp = resp.text();
        warn!("{:?}", resp);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            resp.unwrap().to_string(),
        ));
    }
    let data = resp
        .json()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(data)
}
