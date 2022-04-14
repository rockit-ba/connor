use anyhow::Result;
use bytes::{Bytes};
use futures::{SinkExt, StreamExt, TryStreamExt};
use serde_derive::{Deserialize, Serialize};
use tokio::net::{TcpListener};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use connor::TcpWriter;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut listener_stream = TcpListenerStream::new(listener);
    while let Some(socket) = listener_stream.try_next().await? {
        println!("有连接进入：{:?}",&socket.local_addr().unwrap());
        // 每次有一个客户端的连接进来就创建一个 任务
        // 如果不使用 move 是不能使用swap 外部的变量的，用了move之后，该数据就只能被当前的 任务使用。
        // 当然可以使用 Arc
        tokio::spawn(async move {
            let framed = Framed::new(socket, LengthDelimitedCodec::new());
            let (writer, reader) = &mut framed.split();
            // 注意这里的Ok(Some(req)) 不能拆开写，这样会导致一直 ok()
            while let Ok(Some(req)) = reader.try_next().await {
                let string = String::from_utf8((&req).to_vec()).expect("byte to json 失败！");
                let entry = serde_json::from_str::<Entry>(&string).expect("json to struct 失败！");
                println!("解码入站数据 {:?}", &entry);
                inbound_handle(&entry, writer).await;
            }
            println!("socket 已回收");
        });
    }
    Ok(())
}

async fn inbound_handle(entry: &Entry, writer: &mut TcpWriter) {
    println!("处理入站数据 {:?}", &entry);
    let resp = Entry { name: "回复".to_string(), age: 13 };
    let string = serde_json::to_string(&resp).expect("struct to json 失败");
    let bytes = Bytes::copy_from_slice(string.as_bytes());

    if let Err(e) = writer.send(bytes).await {
        println!("回发消息失败：{:?}", e);
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Entry {
    name: String,
    age: u32,
}
