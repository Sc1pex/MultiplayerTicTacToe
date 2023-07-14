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
    let stream = TcpStream::connect(addr).await.unwrap();
    let mut lines = Framed::new(stream, LinesCodec::new());

    let mut reader = BufReader::new(stdin());

    loop {
        let line = lines.next().await.unwrap().unwrap();
        match line.as_str() {
            "input" => {
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
                lines.send(num.to_string()).await.unwrap();
            }
            "board" => {
                let line = lines.next().await.unwrap().unwrap();
                let board = line.as_bytes();
                print_board(board);
            }
            "end" => {
                let line = lines.next().await.unwrap().unwrap();
                println!("{}", line);
                lines.send(String::new()).await.unwrap();
                break;
            }
            _ => unreachable!(),
        }
    }
}
