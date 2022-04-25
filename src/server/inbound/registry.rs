//! 服务注册

use std::ops::Deref;
use crate::models::{RpcCodec};
use crate::server_bootstrap::ServersMap;
use tracing::info;
use crate::models::request::RegistryRequest;

/// 请求处理
///
/// 返回此次注册的服务结构体
pub async fn handle(json: &str, map: ServersMap) -> RegistryRequest {
    let registry_req = RegistryRequest::from_json(json);
    info!("inbound data [ {:?} ]", &registry_req);
    // 存储注册的服务
    let service = &registry_req.service;
    {
        let mut servers = map.write();
        match servers.get_mut(&service.name) {
            Some(list) => {
                list.push(service.clone());
            }
            None => {
                servers.insert(service.name.clone(), vec![service.clone()]);
            }
        }

        info!("service lists [ {:?} ]", &servers);
        registry_req.deref().clone()
    }
}
