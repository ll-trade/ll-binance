use serde::{Deserialize, Serialize};

use super::{ApiQuery, EmptyResponse, QueryType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartUserStream {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
    pub listen_key: String,
}

impl ApiQuery for StartUserStream {
    type Response = ListenKey;
    const METHOD: &'static str = "userDataStream.start";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingUserStream {
    pub api_key: String,
    pub listen_key: String,
}

impl ApiQuery for PingUserStream {
    type Response = EmptyResponse;
    const METHOD: &'static str = "userDataStream.ping";
    const TYPE: QueryType = QueryType::None;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseUserStream {
    pub api_key: String,
    pub listen_key: String,
}

impl ApiQuery for CloseUserStream {
    type Response = EmptyResponse;
    const METHOD: &'static str = "userDataStream.close";
    const TYPE: QueryType = QueryType::None;
}
