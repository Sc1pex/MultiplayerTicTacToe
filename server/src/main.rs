use common::{ClientMessage, GameEnd, ServerMessage};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, Mutex},
};
use tokio_util::codec::{Framed, LinesCodec};

struct State {
    current_player: usize,
    board: Vec<u8>,
    running: bool,
    moves: usize,
}

type SharedState = Arc<Mutex<State>>;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:6969";
    let listener = TcpListener::bind(addr).await.unwrap();

    let state = Arc::new(Mutex::new(State {
        current_player: 0,
        board: vec![b' '; 9],
        running: false,
        moves: 0,
    }));

    let (tx, _) = broadcast::channel::<()>(10);

    for i in 0..2 {
        let (socket, _) = listener.accept().await.unwrap();
        {
            let state = state.clone();
            let tx = tx.clone();
            let rx = tx.subscribe();
            tokio::spawn(handle_player(state, socket, i, tx, rx));
        }
    }

    {
        let mut state = state.lock().await;
        state.running = true;
    }
    println!("Game started!");

    loop {
        let state = state.lock().await;
        if !state.running {
            break;
        }
        drop(state);

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}

async fn handle_player_old(
    state: SharedState,
    socket: TcpStream,
    player: usize,
    tx: broadcast::Sender<()>,
    mut rx: broadcast::Receiver<()>,
) {
    let mut lines = Framed::new(socket, LinesCodec::new());

    loop {
        let started = {
            let state = state.lock().await;
            state.running
        };
        if !started {
            continue;
        }

        let current_player = {
            let state = state.lock().await;
            state.current_player
        };
        if current_player == player {
            lines.send("input".to_string()).await.unwrap();
            let line = loop {
                match lines.next().await {
                    Some(l) => match l {
                        Ok(l) => break l,
                        Err(_) => {}
                    },
                    None => {}
                }
            };
            let num = line.trim().parse::<usize>().unwrap();
            let mut state = state.lock().await;
            if state.board[num] != b' ' {
                continue;
            }
            state.board[num] = if player == 0 { b'X' } else { b'O' };
            state.current_player = (player + 1) % 2;
            state.moves += 1;

            if check_win(&state.board) {
                state.running = false;
                lines.send("end".to_string()).await.unwrap();
                lines.send("win".to_string()).await.unwrap();
            } else if state.moves == 9 {
                state.running = false;
                lines.send("end".to_string()).await.unwrap();
                lines.send("draw".to_string()).await.unwrap();
            }

            tx.send(()).unwrap();
        } else {
            rx.recv().await.unwrap();
            let board = {
                let state = state.lock().await;
                state.board.clone()
            };

            lines.send("board".to_string()).await.unwrap();
            lines.send(String::from_utf8(board).unwrap()).await.unwrap();

            let state = state.lock().await;
            if check_win(&state.board) {
                lines.send("end".to_string()).await.unwrap();
                lines.send("lose".to_string()).await.unwrap();
                break;
            } else if state.moves == 9 {
                lines.send("end".to_string()).await.unwrap();
                lines.send("draw".to_string()).await.unwrap();
                break;
            }
        }
    }

    let _ = lines.next().await;
}

const WIN_PATTERNS: [[usize; 3]; 8] = [
    [0, 1, 2], // top row
    [3, 4, 5], // middle row
    [6, 7, 8], // bottom row
    [0, 3, 6], // left column
    [1, 4, 7], // middle column
    [2, 5, 8], // right column
    [0, 4, 8], // top-left to bottom-right diagonal
    [2, 4, 6], // top-right to bottom-left diagonal
];

async fn handle_player(
    state: SharedState,
    mut socket: TcpStream,
    player: usize,
    tx: broadcast::Sender<()>,
    mut rx: broadcast::Receiver<()>,
) {
    let (mut r, mut w) = common::make_server_connection(&mut socket);

    loop {
        let started = {
            let state = state.lock().await;
            state.running
        };
        if !started {
            continue;
        }

        let current_player = {
            let state = state.lock().await;
            state.current_player
        };
        if current_player == player {
            w.send(ServerMessage::Input).await.unwrap();
            let num = loop {
                match r.next().await.unwrap().unwrap() {
                    ClientMessage::Input(num) => break num,
                    _ => {}
                }
            };
            let mut state = state.lock().await;
            if state.board[num] != b' ' {
                continue;
            }
            state.board[num] = if player == 0 { b'X' } else { b'O' };
            state.current_player = (player + 1) % 2;
            state.moves += 1;

            if check_win(&state.board) {
                state.running = false;
                w.send(ServerMessage::End(GameEnd::Win)).await.unwrap();
            } else if state.moves == 9 {
                state.running = false;
                w.send(ServerMessage::End(GameEnd::Draw)).await.unwrap();
            }

            tx.send(()).unwrap();
        } else {
            rx.recv().await.unwrap();
            let board = {
                let state = state.lock().await;
                state.board.clone()
            };

            w.send(ServerMessage::Board(board)).await.unwrap();

            let state = state.lock().await;
            if check_win(&state.board) {
                w.send(ServerMessage::End(GameEnd::Lose)).await.unwrap();
                break;
            } else if state.moves == 9 {
                w.send(ServerMessage::End(GameEnd::Draw)).await.unwrap();
                break;
            }
        }
    }
}

fn check_win(board: &[u8]) -> bool {
    for pat in WIN_PATTERNS {
        let mut win = true;
        for &x in &pat {
            if board[x] == b' ' {
                win = false;
                break;
            }
            if board[x] != board[pat[0]] {
                win = false;
                break;
            }
        }
        if win {
            return true;
        }
    }

    false
}
