use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::net::{
    tcp::{ReadHalf, WriteHalf},
    TcpStream,
};
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    Input,
    Board(Vec<u8>),
    End(GameEnd),
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Input(usize),
}

#[derive(Serialize, Deserialize)]
pub enum GameEnd {
    Win,
    Lose,
    Draw,
}

impl Display for GameEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameEnd::Win => write!(f, "You win!"),
            GameEnd::Lose => write!(f, "You lose!"),
            GameEnd::Draw => write!(f, "Draw!"),
        }
    }
}

pub type DelimitedR<'a> = FramedRead<ReadHalf<'a>, LengthDelimitedCodec>;
pub type DelimitedW<'a> = FramedWrite<WriteHalf<'a>, LengthDelimitedCodec>;

pub type SerializedR<'a, Item> = Framed<DelimitedR<'a>, Item, (), Json<Item, ()>>;
pub type SerializedW<'a, Item> = Framed<DelimitedW<'a>, (), Item, Json<(), Item>>;

pub fn make_connection<'a, R, S>(
    socket: &'a mut TcpStream,
) -> (SerializedR<'a, R>, SerializedW<'a, S>) {
    let (r, w) = socket.split();

    let delimited_r: DelimitedR = FramedRead::new(r, LengthDelimitedCodec::new());
    let delimited_w: DelimitedW = FramedWrite::new(w, LengthDelimitedCodec::new());

    let serialized_r = Framed::new(delimited_r, Json::default());
    let serialized_w = Framed::new(delimited_w, Json::default());

    (serialized_r, serialized_w)
}

pub fn make_server_connection<'a>(
    socket: &'a mut TcpStream,
) -> (
    SerializedR<'a, ClientMessage>,
    SerializedW<'a, ServerMessage>,
) {
    make_connection(socket)
}

pub fn make_client_connection<'a>(
    socket: &'a mut TcpStream,
) -> (
    SerializedR<'a, ServerMessage>,
    SerializedW<'a, ClientMessage>,
) {
    make_connection(socket)
}
