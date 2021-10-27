mod robot;
mod entity;

use anyhow::{anyhow, Result};
use brotli::CompressorWriter;
use log::{LevelFilter, error, info};
use once_cell::sync::Lazy;
use std::borrow::BorrowMut;
use std::io::Write;
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread::{self, spawn};
use std::time::Duration;
use tungstenite::{Message, accept};
use bincode::{config::Configuration};
use entity::*;

static SENDER: Lazy<Mutex<Option<Sender<BotMessage>>>> = Lazy::new(|| Mutex::new(None));

fn main() -> Result<()> {
    let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init();

    spawn(||{
        info!("start itchat");
        robot::run();
        error!("itchat stop");
        thread::sleep(Duration::from_secs(3));
    });
	
    let server = TcpListener::bind("127.0.0.1:9001")?;
    for stream in server.incoming() {
        spawn(move || -> Result<()> {
            let mut websocket = accept(stream?)?;
            //set sender
            let (sender, receiver): (Sender<BotMessage>, Receiver<BotMessage>) = channel();
            if let Ok(mut s) = SENDER.lock() {
                s.borrow_mut().replace(sender);
            }
            loop {
                if let Err(err) = (||-> Result<()>{
                    let msg = receiver.recv()?;
                    let data = bincode::encode_to_vec(&msg, Configuration::standard()).map_err(|err| anyhow!("{:?}", err))?;
                    let mut buf = vec![];
                    {
                        let mut writer = CompressorWriter::new(&mut buf, 512, 11, 22);
                        writer.write(&data)?;
                        writer.flush()?;
                    }
                    websocket.write_message(Message::Binary(buf))?;
                    Ok(())
                })(){
                    error!("websocket error: {:?}", err);
                    break;
                }
            }
            //delete sender
            if let Ok(mut s) = SENDER.lock() {
                let _ = s.borrow_mut().take();
            }
            Ok(())
        });
    }
    Ok(())
}