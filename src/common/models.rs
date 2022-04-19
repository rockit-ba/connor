use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// 服务注册
pub const REGISTRY: &str = "0";
/// 服务发现：根据 service-name 查询 service list
pub const DISCOVERY: &str = "1";
#[allow(dead_code)]
/// 服务发现:获取所有的 service IDS
pub const DISCOVERY_IDS: &str = "2";

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
/// 服务信息
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NewService {
    pub id: String,
    pub name: String,
    pub port: u32,
    pub host: String,
    // 元数据，可选
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
    // 是否成功
    pub success: bool,
    pub service_name: String,
}
impl RegistryResponse {
    pub fn new(success: bool, service_name: &str) -> Self {
        Self {
            rpc_kind: String::from(REGISTRY),
            success,
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryResponse {
    rpc_kind: String,
    pub service_name: String,
    pub services: Vec<NewService>,
}
impl DiscoveryResponse {
    pub fn new(service_name: &str,services: Vec<NewService>) -> Self {
        Self {
            rpc_kind: DISCOVERY.to_string(),
            service_name: service_name.to_string(),
            services
        }
    }
}
