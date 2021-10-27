use std::borrow::Borrow;

use crate::entity::*;
use crate::SENDER;
use anyhow::{anyhow, Result};
use chrono::Utc;
use inline_python::{python, Context};
use log::{error, info};
use pyo3::{prelude::*, wrap_pyfunction};
use serde_json::Value;

#[pyfunction]
fn handle_message(msg: &str) {
    info!("message comming: len={}", msg.len());
    if let Err(err) = (|| -> Result<()> {
        let v: Value = serde_json::from_str(msg)?;

        let is_group = v["User"]["MemberCount"].as_i64().unwrap_or(0) > 0;
        
        let time = v["CreateTime"].as_i64().unwrap_or(Utc::now().timestamp());
        let content = v["Content"].as_str().unwrap_or("").to_string();
        let from = (if is_group {
            v["User"]["NickName"].as_str()
        } else {
            v["ActualNickName"].as_str()
        }).unwrap_or("").to_string();

        let sender = SENDER.lock().map_err(|err| anyhow!("{:?}", err))?;
        if let Some(sender) = sender.borrow().as_ref() {
            sender.send(BotMessage::Text(TextMessage{from, time, content}))?;
        }
        Ok(())
    })() {
        error!("send message: {:?}", err);
    }
}

/// https://github.com/why2lyj/ItChat-UOS/blob/master/itchat/components/login.py
#[pyfunction]
fn qr_callback(uuid: &str, status: &str, qrcode: &[u8]) {
    info!("qr_callback:{} {} byte len={:?}", uuid, status, qrcode.len());
    if qrcode.len() == 0{
        info!("no qrcode.");
        return;
    }
    if let Err(err) = (|| -> Result<()> {
        let img = image::load_from_memory(qrcode)?;
        let gray = img.to_luma8();
        let mut gray = rqrr::PreparedImage::prepare(gray);
        // Scan QR code

        // Search for grids, without decoding
        let grids = gray.detect_grids();
        if grids.len() == 0{
            return Err(anyhow!("empy qrcode"));
        }
        // Decode the grid
        let (_meta, content) = grids[0].decode()?;
        if content.len() == 0{
            return Err(anyhow!("empy qrcode content"));
        }
        let sender = SENDER.lock().map_err(|err| anyhow!("{:?}", err))?;
        if let Some(sender) = sender.borrow().as_ref() {
            sender.send(BotMessage::QRCode(QRCodeMessage {
                uuid: uuid.to_string(),
                status: status.to_string(),
                content,
            }))?;
        }else{
            info!("no sender.");
        }
        Ok(())
    })() {
        error!("send qrcode: {:?}", err);
    }
}

pub fn run() {
    let c = Context::new();
    c.add_wrapped(wrap_pyfunction!(handle_message));
    c.add_wrapped(wrap_pyfunction!(qr_callback));
    
    c.run(python! {
        import itchat, time, json
        from itchat.content import *

        @itchat.msg_register([TEXT, MAP, CARD, NOTE, SHARING], isFriendChat=True)
        def accept_friend_chat(msg):
            handle_message(json.dumps(msg))

        @itchat.msg_register([TEXT, MAP, CARD, NOTE, SHARING], isGroupChat=True)
        def accept_group_chat(msg):
            handle_message(json.dumps(msg))

        itchat.auto_login(True, qrCallback=qr_callback)
        itchat.run(True)
    });
}