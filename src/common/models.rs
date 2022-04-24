//! 通用模型

pub mod request;
pub mod response;

use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use crate::custom_error::{Json2StructErr, Struct2JsonErr};

pub type TcpReader = SplitStream<Framed<TcpStream, LengthDelimitedCodec>>;
pub type TcpWriter = SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>;

/// 通信类型枚举
#[derive(PartialEq, Debug, Clone)]
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
    ServiceCheck,
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
            "0" => Ok(RpcKind::Registry),
            "1" => Ok(RpcKind::Discovery),
            "2" => Ok(RpcKind::DiscoveryNames),
            "3" => Ok(RpcKind::Deregistry),
            "4" => Ok(RpcKind::ServiceCheck),
            &_ => Err("RpcKind Parser Fail"),
        }
    }
}

/// 入站处理器处理之后发送的响应客户端的事件
#[derive(PartialEq, Debug, Clone)]
pub enum InboundHandleEvent {
    /// 服务刷新
    ServiceRefresh {
        service_name: String,
        service_list: Option<Vec<NewService>>,
    },
    /// 获取所有的 service name list
    ServiceNames { service_names: Vec<String> },
    /// 服务下线
    ServiceOfOut {
        service_name: String,
        service_id: String,
    },
    /// service 状态检测
    ServiceCheck { service_id: String },
}

/// 请求/响应实体的公共方法
pub trait RpcCodec: Debug {
    /// 获取类型
    fn rpc_kind() -> RpcKind;

    /// 从json转换为struct
    fn from_json<'a>(json: &'a str) -> Box<Self>
    where
        Self: Sized + Deserialize<'a>,
    {
        Box::new(
            serde_json::from_str::<Self>(json)
                .unwrap_or_else(|_| panic!("{}", Json2StructErr)),
        )
    }

    /// 将自己转换为 传输的json，并在前面添加了 kind 头标识
    fn to_json(&self) -> String
    where
        Self: Serialize,
    {
        let json = serde_json::to_string(self)
            .unwrap_or_else(|_| panic!("{}", Struct2JsonErr));

        format!("{}{}", Self::rpc_kind(), json)
    }
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

