//! 心跳检测

use crate::models::request::HeartbeatRequest;
use crate::models::{InboundHandleSingleEvent, RpcCodec};
use crate::server_bootstrap::{ServersHeartbeatMap, ServersMap};
use std::time::SystemTime;
use tracing::info;

/// 更新 ServersHeartbeatMap 数据
///
/// 判断servers_map中是否存在该实例，如果不存在，表明是之前心跳超时被删除的实例，需要通知客户端重新注册实例
///
/// 无返回值
pub async fn handle(json: &str, services_heartbeat_map: ServersHeartbeatMap, services_map: ServersMap) -> InboundHandleSingleEvent {
    let heartbeat_req = HeartbeatRequest::from_json(json);
    let service_id = &heartbeat_req.service_id;
    info!("inbound data [ {:?} ]", &heartbeat_req);
    {
        let mut write_guard = services_heartbeat_map.write();
        match write_guard.get_mut(service_id) {
            None => {
                // 不存在则插入
                write_guard.insert(service_id.clone(), SystemTime::now());
            }
            Some(time) => {
                // 存在则更新时间
                *time = SystemTime::now();
            }
        }
        info!(
            "concurrent services_heartbeat_map [ {:?} ]",
            write_guard.keys()
        );
    }

    {
        let read_guard = services_map.read();
        let flag = read_guard.values()
            .flatten()
            .find(|service| { service.id.eq(service_id) })
            .map_or(false, |_| true);
        InboundHandleSingleEvent::HeartbeatResp {success: flag}
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test() {
        let now = SystemTime::now();

        // we sleep for 2 seconds
        sleep(Duration::new(2, 0));
        match now.elapsed() {
            Ok(elapsed) => {
                // it prints '2'
                println!("{}", elapsed.as_secs());
            }
            Err(e) => {
                // an error occurred!
                println!("Error: {:?}", e);
            }
        }
    }

}
