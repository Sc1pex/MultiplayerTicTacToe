use common::{make_client_connection, ClientMessage, ServerMessage};
use futures::{SinkExt, StreamExt};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    net::TcpStream,
};
use tokio_util::codec::{Framed, LinesCodec};

fn print_board(board: &[u8]) {
    // clear screen
    print!("\x1B[2J\x1B[1;1H");
    println!("+---+---+---+");
    for row in board.chunks(3) {
        print!("|");
        for &cell in row {
            print!(" {} |", cell as char);
        }
        println!("\n+---+---+---+");
    }
}

#[tokio::main]
async fn main() {
    let addr = "localhost:6969";
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let (mut r, mut w) = make_client_connection(&mut stream);

    let mut reader = BufReader::new(stdin());

    loop {
        let msg = r.next().await.unwrap().unwrap();
        match msg {
            ServerMessage::Input => {
                println!("Your turn!");
                let num = loop {
                    let mut buf = String::new();
                    reader.read_line(&mut buf).await.unwrap();
                    match buf.trim().parse::<usize>() {
                        Ok(x) => {
                            if x < 9 {
                                break x;
                            }
                        }
                        Err(_) => {}
                    }
                };
                w.send(ClientMessage::Input(num)).await.unwrap();
            }
            ServerMessage::Board(b) => {
                print_board(&b);
            }
            ServerMessage::End(e) => {
                println!("{}", e.to_string());
                break;
            }
        }
    }
}
