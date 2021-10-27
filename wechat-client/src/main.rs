mod entity{
    include!("../../wechat-server/src/entity.rs");
}
use std::io::Write;
use anyhow::{anyhow, Result};
use bincode::config::Configuration;
use log::{LevelFilter, error, info};
use tungstenite::{Message, connect};
use url::Url;
use entity::*;

//https://lib.rs/crates/embedded-websocket

fn main() -> Result<()>{
    let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();

    info!("Connecting...");

    let (mut socket, response) = connect(Url::parse("ws://127.0.0.1:9001/socket")?)?;
    
    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());

    loop {
        let msg = socket.read_message()?;
        match msg{
            Message::Binary(msg) => {
                if let Err(err) = parse(&msg){
                    error!("parse: {:?}", err);
                }
            }
            other => info!("other: {:?}", other)
        }
    }
}

fn parse(buf: &[u8]) -> Result<()>{
    //decompress
    let mut debuf = vec![];
    {
        let mut dwriter = brotli::DecompressorWriter::new(&mut debuf, 1024 /* buffer size */);
        dwriter.write_all(&buf)?;
        dwriter.flush()?;
    }

    match bincode::decode_from_slice(&debuf, Configuration::standard()).map_err(|err| anyhow!("{:?}", err))?{
        BotMessage::QRCode(code) => {
            info!("qr code: {}", code.content);
            let _ = qr2term::print_qr(code.content);
        }
        BotMessage::Text(text) => {
            info!("{:?}", text);
        }
    }
    Ok(())
}
