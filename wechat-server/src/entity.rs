use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub struct QRCodeMessage {
    pub uuid: String,
    pub status: String,
    pub content: String,
}

#[derive(Encode, Decode, Debug)]
pub struct TextMessage {
    pub from: String,
    pub content: String,
    pub time: i64,
}

#[derive(Encode, Decode, Debug)]
pub enum BotMessage {
    QRCode(QRCodeMessage),
    Text(TextMessage),
}