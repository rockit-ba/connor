//! 实体类

use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use crate::custom_error::{Json2StructErr, Struct2JsonErr};


pub type TcpReader = SplitStream<Framed<TcpStream, LengthDelimitedCodec>>;
pub type TcpWriter = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;

/// 通信类型枚举
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum RpcKind {
    /// 服务注册
    Registry,
    /// 服务发现：根据 service-name 查询 service list
    Discovery,
    /// 服务发现:获取所有的 service names
    DiscoveryNames,
    /// 服务下线
    Deregistry,
    /// 服务检测
    ServiceCheck

}
/// 序列化时用到
impl Display for RpcKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone() as u8)
    }
}
/// 反序列化用到
impl FromStr for RpcKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => {
                Ok(RpcKind::Registry)
            }
            "1" => {
                Ok(RpcKind::Discovery)
            }
            "2" => {
                Ok(RpcKind::DiscoveryNames)
            }
            "3" => {
                Ok(RpcKind::Deregistry)
            }
            "4" => {
                Ok(RpcKind::ServiceCheck)
            }
            &_ => {Err("RpcKind Parser Fail")}
        }
    }
}

/// 请求/响应实体的公共方法
pub trait RpcCodec: Debug {
    /// 获取类型
    fn rpc_kind() -> RpcKind;

    /// 从json转换为struct
    fn from_json<'a>(json: &'a str) -> Box<Self>
        where Self: Sized + Deserialize<'a>
    {
        Box::new(serde_json::from_str::<Self>(json)
            .unwrap_or_else(|_| { panic!("{}", Json2StructErr) }))
    }

    /// 将自己转换为 传输的json，并在前面添加了 kind 头标识
    fn to_json(&self) -> String
        where Self: Serialize
    {
        let json = serde_json::to_string(self)
            .unwrap_or_else(|_| { panic!("{}", Struct2JsonErr) });

        format!("{}{}", Self::rpc_kind(), json)
    }
}

/// 注册服务请求
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RegistryRequest {
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
    fn rpc_kind() -> RpcKind {
        RpcKind::Registry
    }
}

/// 注册服务响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RegistryResponse {
    // 是否成功
    pub success: bool,
    pub service_name: String,
}
impl RegistryResponse {
    pub fn new(success: bool, service_name: &str) -> Self {
        Self {
            success,
            service_name: service_name.to_string(),
        }
    }
}
impl RpcCodec for RegistryResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::Registry
    }
}


/// 服务发现请求：根据service-name 获取所有的service
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryRequest {
    pub service_name: String,
}
impl RpcCodec for DiscoveryRequest {
    fn rpc_kind() -> RpcKind {
        RpcKind::Discovery
    }
}

/// 服务发现响应：根据service-name 获取所有的service
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryResponse {
    pub service_name: String,
    pub services: Option<Vec<NewService>>,
}
impl DiscoveryResponse {
    pub fn new(service_name: &str,services: Option<Vec<NewService>>) -> Self {
        Self {
            service_name: service_name.to_string(),
            services
        }
    }
}
impl RpcCodec for DiscoveryResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::Discovery
    }
}

/// 所有的service name获取请求
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryServiceNamesRequest {}

impl RpcCodec for DiscoveryServiceNamesRequest {
    fn rpc_kind() -> RpcKind {
        RpcKind::DiscoveryNames
    }
}

/// 所有的service name获取响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryServiceNamesResponse {
    service_names: Vec<String>
}
impl DiscoveryServiceNamesResponse {
    pub fn new(service_names: Vec<String>) -> Self {
        Self {
            service_names
        }
    }
}
impl RpcCodec for DiscoveryServiceNamesResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::DiscoveryNames
    }
}

/// 服务下线请求
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DeregistryRequest {
    pub service_name: String,
    pub service_id: String
}
impl RpcCodec for DeregistryRequest {
    fn rpc_kind() -> RpcKind {
        RpcKind::Deregistry
    }
}

/// 根据service-id 状态检测请求
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ServiceCheckRequest {
    pub service_id: String
}
impl RpcCodec for ServiceCheckRequest {
    fn rpc_kind() -> RpcKind {
        RpcKind::ServiceCheck
    }
}

/// 根据service-id 状态检测响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ServiceCheckResponse {
    pub service_id: String
}
impl RpcCodec for ServiceCheckResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::ServiceCheck
    }
}

#[cfg(test)]
mod test {
    use crate::models::{RegistryResponse, RpcCodec, RpcKind};

    #[test]
    fn test() {
        let response = RegistryResponse::new(true, "test");
        let string = response.to_json();
        println!("{}",string);
    }
}
