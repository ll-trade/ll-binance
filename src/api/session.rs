use ed25519_dalek::{ed25519::signature::SignerMut, pkcs8::DecodePrivateKey};
use serde::{Deserialize, Serialize};

use crate::millis_ts;

use super::{ApiQuery, QueryType};

#[derive(Debug, Clone, Deserialize)]
pub struct QSessionStatus;

empty_serde!(QSessionStatus);

impl ApiQuery for QSessionStatus {
    type Response = SessionStatus;
    const METHOD: &'static str = "session.status";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logout;

empty_serde!(Logout);

impl ApiQuery for Logout {
    type Response = SessionStatus;
    const METHOD: &'static str = "session.logout";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatus {
    pub api_key: String,
    pub authorized_since: i64,
    pub connected_since: i64,
    pub return_rate_limits: bool,
    pub server_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logon {
    pub api_key: String,
    pub perm_key: String,
}

impl Logon {
    pub fn sign(&self, recv_window: i64) -> SignedLogon {
        use base64::prelude::*;
        let timestamp = millis_ts();
        let qs = format!(
            "apiKey={}&recvWindow={recv_window}&timestamp={timestamp}",
            self.api_key,
        );
        let mut sign_key =
            ed25519_dalek::SigningKey::from_pkcs8_pem(&self.perm_key).expect("invalid key data");
        let signature = BASE64_STANDARD.encode(sign_key.sign(qs.as_bytes()).to_bytes());
        SignedLogon {
            api_key: self.api_key.clone(),
            signature,
            recv_window,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedLogon {
    api_key: String,
    signature: String,
    timestamp: i64,
    recv_window: i64,
}

impl ApiQuery for SignedLogon {
    type Response = SessionStatus;
    const METHOD: &'static str = "session.logon";
    const TYPE: QueryType = QueryType::None;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignOn {
    pub api_key: String,
    pub recv_window: i64,
    pub signature: String,
    pub timestamp: i64,
}

impl SignOn {
    pub fn new(api_key: String, private_key: &str, recv_window: i64, timestamp: i64) -> Self {
        use base64::prelude::*;
        let qs = format!("apiKey={api_key}&recvWindow={recv_window}&timestamp={timestamp}");
        let mut sign_key =
            ed25519_dalek::SigningKey::from_pkcs8_pem(private_key).expect("invalid key data");
        let signature = BASE64_STANDARD.encode(sign_key.sign(qs.as_bytes()).to_bytes());
        Self {
            api_key,
            recv_window,
            signature,
            timestamp,
        }
    }
}
