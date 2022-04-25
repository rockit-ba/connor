//! 消息出站模块

use bytes::Bytes;
use futures::SinkExt;
use tracing::{error, info};
use crate::models::{InboundHandleEvent, RpcCodec, TcpWriter};
use crate::models::response::{DeregistryResponse, DiscoveryResponse, DiscoveryServiceNamesResponse, RegistryResponse, ServiceCheckResponse};

/// 根据inbound handle 发送的消息进行响应
pub async fn outbound_handle(data: InboundHandleEvent, writer: &mut TcpWriter){
    match data {
        // 服务注册
        InboundHandleEvent::ServiceRegistryResp {success} => {
            info!("Listener ServiceRegistry event");
            let registry_response = RegistryResponse {success};
            response(writer, registry_response.to_json()).await;
        }
        // 服务发现
        InboundHandleEvent::ServiceDiscovery
        { service_name, services } => {
            info!("Listener ServiceDiscovery event");
            let discovery_resp = DiscoveryResponse::new(&service_name, services);
            response(writer, discovery_resp.to_json()).await;
        }
        // 获取所有的 service name list
        InboundHandleEvent::ServiceNames { service_names } => {
            info!("Listener ServiceNames event");
            let names_response = DiscoveryServiceNamesResponse::new(service_names);
            response(writer, names_response.to_json()).await;
        }
        // 服务下线
        InboundHandleEvent::ServiceOfOut {
            service_name,
            service_id,
        } => {
            info!("Listener ServiceOfOut event");
            let dereg_response = DeregistryResponse::new(&service_name, &service_id);
            response(writer, dereg_response.to_json()).await;
        }
        // service 状态检测
        InboundHandleEvent::ServiceCheck { service_id } => {
            info!("Listener ServiceCheck event");
            let check_response = ServiceCheckResponse::new(&service_id);
            response(writer, check_response.to_json()).await;
        }
        // 服务刷新
        InboundHandleEvent::ServiceRefresh {
            service_name,
            service_list,
        } => {
            info!("Listener ServiceRefresh event");
            let discovery_response = DiscoveryResponse::new(&service_name, service_list);
            response(writer, discovery_response.to_json()).await;
        }
        InboundHandleEvent::ServiceDeregistryResp { success: _ } => {

        }
    }
}

/// 响应客户端
///
/// 抛错用户结束该socket的消息监听
async fn response(writer: &mut TcpWriter, content: String)  {
    if let Err(err) = writer.send(Bytes::copy_from_slice(content.as_bytes())).await {
        error!("{:?}",err);
    }
}
