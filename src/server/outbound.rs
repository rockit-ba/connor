//! 消息出站模块

use bytes::Bytes;
use futures::SinkExt;
use tokio::sync::broadcast::Receiver;
use tracing::error;
use crate::models::{InboundHandleEvent, RpcCodec, TcpWriter};
use crate::models::response::{DeregistryResponse, DiscoveryResponse, DiscoveryServiceNamesResponse, ServiceCheckResponse};

/// 根据inbound handle 发送的消息进行响应
pub async fn outbound_handle(receiver: &mut Receiver<InboundHandleEvent>, writer: &mut TcpWriter) {
    while let Ok(data) = receiver.recv().await {
        match data {
            // 服务刷新
            InboundHandleEvent::ServiceRefresh {
                service_name,
                service_list,
            } => {
                let discovery_response = DiscoveryResponse::new(&service_name, service_list);
                response(writer, discovery_response.to_json()).await;
            }
            // 获取所有的 service name list
            InboundHandleEvent::ServiceNames { service_names } => {
                let names_response = DiscoveryServiceNamesResponse::new(service_names);
                response(writer, names_response.to_json()).await;
            }
            // 服务下线
            InboundHandleEvent::ServiceOfOut {
                service_name,
                service_id,
            } => {
                let dereg_response = DeregistryResponse::new(&service_name, &service_id);
                response(writer, dereg_response.to_json()).await;
            }
            // service 状态检测
            InboundHandleEvent::ServiceCheck { service_id } => {
                let check_response = ServiceCheckResponse::new(&service_id);
                response(writer, check_response.to_json()).await;
            }
        }
    }
}

/// 响应客户端
async fn response(writer: &mut TcpWriter, content: String) {
    if let Err(err) = writer
        .send(Bytes::copy_from_slice(content.as_bytes()))
        .await
    {
        error!("Response Error [{:?}]", err);
    }
}
