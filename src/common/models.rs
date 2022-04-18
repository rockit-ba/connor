use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub const REGISTRY: &str = "0";
pub const DISCOVERY: &str = "1";

pub type TcpReader = SplitStream<Framed<TcpStream, LengthDelimitedCodec>>;
pub type TcpWriter = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;

/// 请求的公共方法
pub trait RpcCodec: Debug {
    fn get_rpc_kind(&self) -> String;
}

/// 注册服务请求
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RegistryRequest {
    rpc_kind: String,
    pub service: NewService,
}
/// 注册服务信息
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NewService {
    pub id: String,
    pub name: String,
    pub port: u32,
    pub host: String,
    pub meta: Option<HashMap<String, String>>,
}
impl RpcCodec for RegistryRequest {
    fn get_rpc_kind(&self) -> String {
        self.rpc_kind.clone()
    }
}

/// 注册服务响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RegistryResponse {
    rpc_kind: String,
    pub flag: bool,
    pub service_name: String,
}
impl RegistryResponse {
    pub fn new(flag: bool, service_name: &str) -> Self {
        Self {
            rpc_kind: String::from(REGISTRY),
            flag,
            service_name: service_name.to_string(),
        }
    }
}
impl RpcCodec for RegistryResponse {
    fn get_rpc_kind(&self) -> String {
        String::from(REGISTRY)
    }
}

/// 服务发现请求：根据service-name 获取所有的service
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryRequest {
    rpc_kind: String,
    pub service_name: String,
}
impl RpcCodec for DiscoveryRequest {
    fn get_rpc_kind(&self) -> String {
        DISCOVERY.to_string()
    }
}
