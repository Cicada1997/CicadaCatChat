#![allow(unused_labels)]

mod common;
use common::{MessageType, ChatMessage, create_msg};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

use chrono::Local;
use std::error::Error;

fn log(msg: String) {
    let time = Local::now().format("%H:%M:%S");

    println!("[ {time} ] {msg}")
}

pub fn create_sys_msg(msg: String) -> ChatMessage {
    create_msg(
        "".to_string(), // Will be overwritten
        msg,
        MessageType::SystemMessage
    )
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1";
    let port = 1997;

    let listener = TcpListener::bind(format!("{addr}:{port}")).await?;
    log(format!("Server is acitve on {addr}:{port} !"));


    let (tx, _) = broadcast::channel::<String>(128);


    log(format!("now listening for new clients... "));
    'accept: loop {
        let (socket, addr) = listener.accept().await?;
        log(format!("New connection: {addr}"));

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move {
            handle_client(socket, tx, rx, addr.to_string()).await;
        });
    }
}

pub async fn handle_client(
    mut socket: TcpStream, 
        tx:     broadcast::Sender<String>,
    mut rx:     broadcast::Receiver<String>,
        addr:   String)
{
    let (reader, mut writer) = socket.split();
    let mut reader           = BufReader::new(reader);
    
    let mut username         = String::new();

    reader.read_line(&mut username).await.unwrap();
    let username = username
        .trim()
        .to_string();

    let msg = format!("User \"{username}\" ({addr}) has connected to the chatroom.");
    let join_msg = create_sys_msg(msg.clone()).json().unwrap();
    log(msg);
    tx.send(join_msg.clone()).unwrap();

    let mut buffer_line = String::new();
    'msg_recv: loop {
        tokio::select!{
            res = reader.read_line(&mut buffer_line) => {
                if res.unwrap() == 0 {
                    let msg = format!("User \"{username}\" ({addr}) was disconnected from the chatroom.");
                    let disconnect_msg = create_sys_msg(msg.clone()).json().unwrap();
                    log(msg);

                    tx.send(disconnect_msg).unwrap();
                    break 'msg_recv;
                }

                let msg = create_msg(
                    username.clone(),
                    buffer_line.trim().to_string(),
                    MessageType::UserMessage,
                );

                log(format!("[ MESSAGE SENT ] <{}>: {}", msg.username, msg.content));

                tx.send(msg.json().unwrap()).unwrap();
                buffer_line.clear();
            }

            res = rx.recv() => {
                let msg = res.unwrap();

                let _ = writer.write_all(msg.as_bytes()).await.unwrap();
                let _ = writer.write_all(b"\n").await.unwrap();
            }
        }
    }
}
