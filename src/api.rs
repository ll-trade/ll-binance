use std::collections::BTreeMap;

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use session::Logon;
use sproxy::ProxyConfig;
use tracing::{info, warn};
use ws_tool::{
    codec::{StringCodec, StringRecv, StringSend},
    connector::{tcp_connect, wrap_rustls},
    errors::WsError,
    frame::OpCode,
    stream::{SyncStream, SyncStreamRead, SyncStreamWrite},
    ClientBuilder,
};

use crate::millis_ts;
use crate::string_as_f64;
use crate::string_as_u64;

pub const BASE_SPOT_URL: &str = "wss://ws-api.binance.com:443/ws-api/v3";
pub const TEST_SPOT_URL: &str = "wss://testnet.binance.vision/ws-api/v3";
pub const BASE_FUTURE_URL: &str = "wss://ws-fapi.binance.com/ws-fapi/v1";
pub const TEST_FUTURE_URL: &str = "wss://testnet.binancefuture.com/ws-fapi/v1";
pub const BACKUP_PORT: u16 = 9443;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<T> {
    pub id: u64,
    pub method: &'static str,
    pub params: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_rate_limits: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawResponse<T> {
    #[serde(default)]
    pub id: u64,
    pub status: i64,
    pub result: Option<T>,
    pub error: Option<serde_json::Value>,
    #[serde(rename = "rateLimits")]
    pub rate_limits: Vec<RateLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrResponse {
    pub id: u64,
    pub error: serde_json::Value,
    pub status: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
    pub id: i64,
    pub result: T,
    pub rate_limits: Vec<RateLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: i64,
    pub limit: i64,
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("{0}")]
    WsError(WsError),
    #[error("{0:?}")]
    ApiError(ErrResponse),
    #[error("{0}")]
    SerdeError(serde_json::Error),
}

impl From<WsError> for ClientError {
    fn from(value: WsError) -> Self {
        ClientError::WsError(value)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeError(value)
    }
}

pub struct AutoReconnectClient {
    req_id: u64,
    auth: Option<Logon>,
    conn_fn: Box<dyn FnMut() -> Result<StringCodec<SyncStream>, WsError>>,
    recv_window: i64,
    messages: BTreeMap<u64, String>,
    stream: Option<StringCodec<SyncStream>>,
}

impl AutoReconnectClient {
    pub fn new(
        auth: Option<Logon>,
        conn_fn: Box<dyn FnMut() -> Result<StringCodec<SyncStream>, WsError>>,
        recv_window: i64,
    ) -> Self {
        Self {
            req_id: 0,
            auth,
            conn_fn,
            recv_window,
            messages: Default::default(),
            stream: None,
        }
    }

    fn wrap<P>(&mut self, ty: QueryType, param: P) -> ParamWrapper<P> {
        match ty {
            QueryType::None => ParamWrapper {
                recv_window: None,
                api_key: None,
                timestamp: 0,
                other: param,
            },
            QueryType::Authorized => ParamWrapper {
                recv_window: Some(self.recv_window),
                api_key: Some(self.auth.clone().map(|a| a.api_key).unwrap_or_default()),
                timestamp: millis_ts(),
                other: param,
            },
            QueryType::AuthWithoutApiKey => ParamWrapper {
                recv_window: Some(self.recv_window),
                api_key: None,
                timestamp: millis_ts(),
                other: param,
            },
        }
    }

    pub fn client(
        url: &'static str,
        max_retries: usize,
        proxy: Option<ProxyConfig>,
        recv_window: i64,
        auth: Option<Logon>,
    ) -> Self {
        let mut try_times = 0;
        let conn_single = move || {
            let url = url::Url::parse(url).map_err(|e| WsError::InvalidUri(e.to_string()))?;
            let stream = match proxy.clone() {
                Some(config) => {
                    sproxy::create_conn(
                        &config,
                        url.host_str().unwrap().into(),
                        url.port().unwrap_or(443),
                    )?
                    .0
                }
                None => tcp_connect(&url.as_str().parse().unwrap())?,
            };
            tracing::debug!(
                "socket info local {:?} remote {:?}",
                stream.local_addr(),
                stream.peer_addr()
            );
            let stream = wrap_rustls(stream, url.host_str().unwrap(), vec![])?;
            ClientBuilder::new().with_stream(
                url.as_str().parse().unwrap(),
                SyncStream::Rustls(stream),
                StringCodec::check_fn,
            )
        };
        let conn_fn = move || loop {
            if try_times >= max_retries {
                return Err(WsError::ConnectionFailed(format!(
                    "try to connect over {} times",
                    max_retries
                )));
            }
            match conn_single() {
                Ok(s) => {
                    try_times = 0;
                    return Ok(s);
                }
                Err(e) => {
                    try_times += 1;
                    tracing::warn!("{e}");
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        };
        Self::new(auth, Box::new(conn_fn), recv_window)
    }

    // pub fn batch_query<P: ApiQuery>(
    //     &mut self,
    //     batch_size: usize,
    //     params: Vec<P>,
    // ) -> Result<Vec<Response<P::Response>>, ClientError> {
    //     let mut req_ids = vec![];
    //     let mut results = vec![];
    //     let total = params.len();
    //     for (idx, p) in params.into_iter().enumerate() {
    //         let param = self.wrap(P::TYPE, p);
    //         req_ids.push(self._send(P::METHOD, &param)?);
    //         if (idx != 0 && idx % batch_size == 0) || (idx + 1 == total) {
    //             for id in req_ids.iter() {
    //                 let resp = self._recv(*id)?;
    //                 let resp: Response<P::Response> = serde_json::from_str(&resp)?;
    //                 results.push(resp);
    //             }
    //             req_ids.clear();
    //         }
    //     }
    //     Ok(results)
    // }

    pub fn query<P: ApiQuery>(&mut self, param: P) -> ApiResult<P::Response> {
        let param = self.wrap(P::TYPE, param);
        if self.stream.is_none() {
            self.get_stream()?;
            if let Some(signed) = self.auth.as_ref().map(|auth| auth.sign(self.recv_window)) {
                self.query(signed)?;
                info!("logon ok");
            }
        }
        match self._send(P::METHOD, &param) {
            Ok(req_id) => match self._recv(req_id) {
                Ok(resp) => serde_json::from_str(&resp).map_err(|e| e.into()),
                Err(ClientError::WsError(e)) => {
                    warn!("error while recv response {e}");
                    self.stream = None;
                    if let Some(signed) = self.auth.as_ref().map(|auth| auth.sign(self.recv_window))
                    {
                        self.query(signed)?;
                        info!("logon ok");
                    }
                    self.query_without_retry(&param)
                }
                Err(e) => {
                    warn!("error while recv response {e}");
                    // self.stream = None;
                    // if let Some(signed) = self.auth.as_ref().map(|auth| auth.sign(self.recv_window))
                    // {
                    //     self.query(signed)?;
                    //     info!("logon ok");
                    // }
                    self.query_without_retry(&param)
                }
            },
            Err(e) => {
                warn!("error while send message {e}");
                self.stream = None;
                if let Some(signed) = self.auth.as_ref().map(|auth| auth.sign(self.recv_window)) {
                    self.query(signed)?;
                    info!("logon ok");
                }
                self.query_without_retry(&param)
            }
        }
    }

    fn query_without_retry<P: ApiQuery>(
        &mut self,
        param: &ParamWrapper<P>,
    ) -> Result<Response<<P as ApiQuery>::Response>, ClientError> {
        self.get_stream()?;
        let req_id = self._send(P::METHOD, param)?;
        let resp = self._recv(req_id)?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }

    fn get_stream(&mut self) -> Result<&mut StringCodec<SyncStream>, WsError> {
        let create_new_stream = self.stream.is_none();
        if create_new_stream {
            tracing::debug!("rebuild stream ...");
            self.stream = Some((self.conn_fn)()?);
        }
        Ok(self.stream.as_mut().unwrap())
    }

    fn _send<T: Serialize>(
        &mut self,
        method: &'static str,
        params: &T,
    ) -> Result<u64, ClientError> {
        self.req_id += 1;
        let id = self.req_id;
        let req = Request {
            id,
            method,
            params,
            return_rate_limits: None,
        };
        let req = serde_json::to_string(&req)?;
        tracing::debug!("{req}");
        self.get_stream()?.send(&req)?;
        Ok(self.req_id)
    }

    fn _recv(&mut self, id: u64) -> Result<String, ClientError> {
        #[derive(Deserialize)]
        struct CheckResp {
            #[serde(default)]
            id: u64,
            status: i64,
        }
        if self.messages.contains_key(&id) {
            Ok(self.messages.remove(&id).unwrap())
        } else {
            loop {
                let msg = self.get_stream()?.receive()?;
                let code = msg.code;
                let data = msg.data.to_string();
                match code {
                    OpCode::Ping => {
                        self.get_stream()?.pong(&data)?;
                        tracing::debug!("got server ping, pong back ...");
                    }
                    OpCode::Text => {
                        let resp: CheckResp = serde_json::from_str(&msg.data)?;
                        // check error response first
                        if resp.id == 0 || resp.status != 200 {
                            let err: ErrResponse = serde_json::from_str(&msg.data)?;
                            break Err(ClientError::ApiError(err));
                        } else if resp.id != id {
                            self.messages.insert(resp.id, data);
                        } else {
                            // avoid to print too much when response is large
                            if msg.data.len() <= 8192 {
                                tracing::debug!("{}", msg.data);
                            } else {
                                tracing::debug!(
                                    "truncated: {}",
                                    msg.data.chars().take(8192).collect::<String>()
                                );
                            }
                            break Ok(msg.data.to_string());
                        }
                    }
                    c => {
                        tracing::error!("unexpected frame type {c:?}");
                        continue;
                    }
                }
            }
        }
    }
}

pub struct Client {
    req_id: u64,
    reader: StringRecv<SyncStreamRead>,
    writer: StringSend<SyncStreamWrite>,
    api_key: String,
    recv_window: i64,
    messages: BTreeMap<u64, String>,
}

impl Client {
    pub fn conn(
        url: &str,
        proxy: Option<ProxyConfig>,
        recv_window: i64,
    ) -> Result<Client, WsError> {
        let url = url::Url::parse(url).map_err(|e| WsError::InvalidUri(e.to_string()))?;
        let stream = match proxy {
            Some(config) => {
                sproxy::create_conn(
                    &config,
                    url.host_str().unwrap().into(),
                    url.port().unwrap_or(443),
                )?
                .0
            }
            None => tcp_connect(&url.as_str().parse().unwrap())?,
        };
        tracing::debug!(
            "socket info local {:?} remote {:?}",
            stream.local_addr(),
            stream.peer_addr()
        );
        let stream = wrap_rustls(stream, url.host_str().unwrap(), vec![])?;
        let stream = SyncStream::Rustls(stream);
        let (reader, writer) = ClientBuilder::new()
            .with_stream(url.as_str().parse().unwrap(), stream, StringCodec::check_fn)?
            .split();
        Ok(Client {
            req_id: 0,
            reader,
            writer,
            recv_window,
            api_key: Default::default(),
            messages: Default::default(),
        })
    }
}

pub type ApiResult<T> = Result<Response<T>, ClientError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryType {
    None,
    Authorized,
    AuthWithoutApiKey,
}

pub trait ApiQuery: Sized + Serialize {
    type Response: DeserializeOwned;
    const METHOD: &'static str;
    const TYPE: QueryType;
}

macro_rules! empty_serde {
    ($t:ty) => {
        impl serde::Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeMap;
                let map = serializer.serialize_map(Some(0))?;
                map.end()
            }
        }
    };
}
/// 账户请求
pub mod account;
/// 常用API
pub mod common;
/// 行情接口
pub mod market;
/// 身份认证
pub mod session;
/// 交易接口
pub mod trade;
/// 用户数据流
pub mod user_stream;
#[derive(Debug, Clone, Serialize)]
pub struct EmptyResponse;

impl<'de> Deserialize<'de> for EmptyResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde_json::Value::deserialize(deserializer)?;
        Ok(Self)
    }
}

impl Client {
    pub fn batch_query<P: ApiQuery>(
        &mut self,
        batch_size: usize,
        params: Vec<P>,
    ) -> Result<Vec<Response<P::Response>>, ClientError> {
        let mut req_ids = vec![];
        let mut results = vec![];
        let total = params.len();
        for (idx, p) in params.into_iter().enumerate() {
            let param = self.wrap(P::TYPE, p);
            req_ids.push(self._send(P::METHOD, param)?);
            if (idx != 0 && idx % batch_size == 0) || (idx + 1 == total) {
                for id in req_ids.iter() {
                    let resp = self._recv(*id)?;
                    let resp: Response<P::Response> = serde_json::from_str(&resp)?;
                    results.push(resp);
                }
                req_ids.clear();
            }
        }
        Ok(results)
    }

    pub fn query<P: ApiQuery>(&mut self, param: P) -> ApiResult<P::Response> {
        let param = self.wrap(P::TYPE, param);
        let req_id = self._send(P::METHOD, param)?;
        let resp = self._recv(req_id)?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }

    fn wrap<P>(&mut self, ty: QueryType, param: P) -> ParamWrapper<P> {
        match ty {
            QueryType::None => ParamWrapper {
                recv_window: None,
                api_key: None,
                timestamp: 0,
                other: param,
            },
            QueryType::Authorized => ParamWrapper {
                recv_window: Some(self.recv_window),
                api_key: Some(self.api_key.clone()),
                timestamp: millis_ts(),
                other: param,
            },
            QueryType::AuthWithoutApiKey => ParamWrapper {
                recv_window: Some(self.recv_window),
                api_key: None,
                timestamp: millis_ts(),
                other: param,
            },
        }
    }

    fn _send<T: Serialize>(&mut self, method: &'static str, params: T) -> Result<u64, ClientError> {
        self.req_id += 1;
        let id = self.req_id;
        let req = Request {
            id,
            method,
            params,
            return_rate_limits: None,
        };
        let req = serde_json::to_string(&req)?;
        tracing::debug!("{req}");
        self.writer.send(&req)?;
        Ok(self.req_id)
    }

    fn _recv(&mut self, id: u64) -> Result<String, ClientError> {
        #[derive(Deserialize)]
        struct CheckResp {
            #[serde(default)]
            id: u64,
            status: i64,
        }
        if self.messages.contains_key(&id) {
            Ok(self.messages.remove(&id).unwrap())
        } else {
            loop {
                let msg = self.reader.receive()?;
                match msg.code {
                    OpCode::Ping => {
                        self.writer.pong(&msg.data)?;
                        tracing::debug!("got server ping, pong back ...");
                    }
                    OpCode::Text => {
                        let resp: CheckResp = serde_json::from_str(&msg.data)?;
                        // check error response first
                        if resp.id == 0 || resp.status != 200 {
                            let err: ErrResponse = serde_json::from_str(&msg.data)?;
                            break Err(ClientError::ApiError(err));
                        } else if resp.id != id {
                            self.messages.insert(resp.id, msg.data.to_string());
                        } else {
                            // avoid to print too much when response is large
                            if msg.data.len() <= 1024 {
                                tracing::debug!("{}", msg.data);
                            } else {
                                tracing::debug!(
                                    "truncated: {}",
                                    msg.data.chars().take(1024).collect::<String>()
                                );
                            }
                            break Ok(msg.data.to_string());
                        }
                    }
                    c => {
                        tracing::error!("unexpected frame type {c:?}");
                        continue;
                    }
                }
            }
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamWrapper<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "is_zero")]
    pub timestamp: i64,
    #[serde(flatten)]
    pub other: T,
}

fn is_zero(v: &i64) -> bool {
    *v == 0
}
