//! 消息出站模块

use bytes::Bytes;
use futures::SinkExt;
use tracing::{error, info};
use crate::models::{InboundHandleBroadcastEvent, InboundHandleSingleEvent, RpcCodec, TcpWriter};
use crate::models::response::{AddServiceResponse, DeregistryResponse, DiscoveryResponse, DiscoveryServiceNamesResponse,
                              RegistryResponse, RemoveServiceResponse, ServiceCheckResponse};

/// 根据inbound handle 发送的消息进行响应
pub async fn outbound_handle(data: InboundHandleSingleEvent, writer: &mut TcpWriter){
    match data {
        // 服务注册
        InboundHandleSingleEvent::ServiceRegistryResp {success} => {
            info!("Listener ServiceRegistry event");
            let registry_response = RegistryResponse {success};
            response(writer, registry_response.to_json()).await;
        }
        // 服务发现
        InboundHandleSingleEvent::ServiceDiscoveryResp
        { service_name, services } => {
            info!("Listener ServiceDiscovery event");
            let discovery_resp = DiscoveryResponse::new(&service_name, services);
            response(writer, discovery_resp.to_json()).await;
        }
        // 获取所有的 service name list
        InboundHandleSingleEvent::ServiceNamesResp { service_names } => {
            info!("Listener ServiceNames event");
            let names_response = DiscoveryServiceNamesResponse::new(service_names);
            response(writer, names_response.to_json()).await;
        }
        // service 状态检测
        InboundHandleSingleEvent::ServiceCheckResp { service_id } => {
            info!("Listener ServiceCheck event");
            let check_response = ServiceCheckResponse::new(&service_id);
            response(writer, check_response.to_json()).await;
        }
        // 服务下线
        InboundHandleSingleEvent::ServiceDeregistryResp { success } => {
            info!("Listener ServiceDeregistry event");
            let dereg_response = DeregistryResponse { success };
            response(writer, dereg_response.to_json()).await;
        }

    }
}

pub async fn outbound_broad_handle(data: InboundHandleBroadcastEvent, writer: &mut TcpWriter){
    match data {
        InboundHandleBroadcastEvent::None => {}
        InboundHandleBroadcastEvent::AddServiceResp { service_name, service_list } => {
            info!("Listener AddService event");
            let add_service_response = AddServiceResponse::new(&service_name,service_list);
            response(writer, add_service_response.to_json()).await;
        }
        InboundHandleBroadcastEvent::RemoveServiceResp { service_id, service_name } => {
            info!("Listener RemoveService event");
            let remove_service_response = RemoveServiceResponse::new(&service_id, &service_name);
            response(writer, remove_service_response.to_json()).await;
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
