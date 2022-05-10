//! response 模型

use crate::models::{NewService, RpcCodec, RpcKind};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RegistryResponse {
    pub success: bool,
}
impl RpcCodec for RegistryResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::Registry
    }
}

/// 服务发现响应：根据service-name 获取所有的service
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryResponse {
    pub service_name: String,
    pub services: Option<Vec<NewService>>,
}
impl DiscoveryResponse {
    pub fn new(service_name: &str, services: Option<Vec<NewService>>) -> Self {
        Self {
            service_name: service_name.to_string(),
            services,
        }
    }
}
impl RpcCodec for DiscoveryResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::Discovery
    }
}

/// 所有的service name获取响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DiscoveryServiceNamesResponse {
    service_names: Vec<String>,
}
impl DiscoveryServiceNamesResponse {
    pub fn new(service_names: Vec<String>) -> Self {
        Self { service_names }
    }
}
impl RpcCodec for DiscoveryServiceNamesResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::DiscoveryNames
    }
}

/// 服务下线响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DeregistryResponse {
    pub success: bool,
}
impl RpcCodec for DeregistryResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::Deregistry
    }
}

/// 根据service-id 状态检测响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ServiceCheckResponse {
    pub service_id: String,
}
impl ServiceCheckResponse {
    pub fn new(service_id: &str) -> Self {
        Self {
            service_id: service_id.to_string(),
        }
    }
}
impl RpcCodec for ServiceCheckResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::ServiceCheck
    }
}

/// 当前客户端添加服务响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AddServiceResponse {
    service_name: String,
    service_list: Vec<NewService>,
}
impl AddServiceResponse {
    pub fn new(service_name: &str, service_list: Vec<NewService>) -> Self {
        Self {
            service_name: service_name.to_string(),
            service_list,
        }
    }
}
impl RpcCodec for AddServiceResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::AddService
    }
}

/// 当前客户端删除服务响应
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct RemoveServiceResponse {
    service_name: String,
    service_list: Vec<NewService>,
}
impl RemoveServiceResponse {
    pub fn new(service_name: &str, service_list: Vec<NewService>) -> Self {
        Self {
            service_name: service_name.to_string(),
            service_list,
        }
    }
}
impl RpcCodec for RemoveServiceResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::RemoveService
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct HeartbeatResponse {
    pub success: bool,
}
impl RpcCodec for HeartbeatResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::Heartbeat
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct HeartbeatTimeoutResponse {
    pub service_ids: Vec<String>,
}
impl HeartbeatTimeoutResponse {
    pub fn new(service_ids: Vec<String>) -> Self {
        Self { service_ids }
    }
}
impl RpcCodec for HeartbeatTimeoutResponse {
    fn rpc_kind() -> RpcKind {
        RpcKind::HeartbeatTimeout
    }
}