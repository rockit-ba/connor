//! 配置文件解析
use std::path::PathBuf;
use lazy_static::lazy_static;
use config::{Config, File, FileFormat};
use tracing::info;


// 加载全局 ServerConfig
lazy_static! {
    pub static ref SERVER_CONFIG: ServerConfig = ServerConfig::new();
}

/// conf.yaml 解析类
///
/// 字段含义查看 config/conf.yaml 文件
#[derive(Debug, serde_derive::Deserialize, PartialEq)]
pub struct ServerConfig {
    /// 当前服务器的名称标识
    pub server_address: String,
    pub cluster_address: Vec<String>,
}

impl ServerConfig {
    fn new() -> Self {
        let err_msg = "配置文件加载失败 \n ";
        let path_buf = Self::get_conf_path();
        let config = Config::builder()
            .add_source(File::new(path_buf.to_str().unwrap(), FileFormat::Yaml))
            .build().expect(err_msg);
        config.try_deserialize::<ServerConfig>().expect(err_msg)
    }

    /// 获取配置文件path
    fn get_conf_path() -> PathBuf  {
        let work_dir = std::env::current_exe().unwrap();
        let mut path_buf = work_dir.parent().unwrap().join("config");
        path_buf.push("conf.yaml");
        info!("加载配置文件：{:?}",&path_buf);
        path_buf
    }
}