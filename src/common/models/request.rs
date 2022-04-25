//! request 模型

use crate::models::{NewService, RpcCodec, RpcKind};
use serde_derive::{Serialize, Deserialize};

/// 注册服务请求
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RegistryRequest {
    pub service: NewService,
}
impl RpcCodec for RegistryRequest {
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

/// 所有的service name获取请求
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryServiceNamesRequest {}

impl RpcCodec for DiscoveryServiceNamesRequest {
    fn rpc_kind() -> RpcKind {
        RpcKind::DiscoveryNames
    }
}

/// 服务下线请求
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DeregistryRequest {
    pub service_name: String,
    pub service_id: String,
}
impl RpcCodec for DeregistryRequest {
    fn rpc_kind() -> RpcKind {
        RpcKind::Deregistry
    }
}

/// 根据service-id 状态检测请求
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ServiceCheckRequest {
    pub service_id: String,
}
impl RpcCodec for ServiceCheckRequest {
    fn rpc_kind() -> RpcKind {
        RpcKind::ServiceCheck
    }
}