#![allow(unused_labels)]

mod common;
use common::{MessageType, ChatMessage, create_msg};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    signal,
};

use serde::{
    Serialize,
    Deserialize,
};

use chrono::Local;
use std::error::Error;
use std::sync::{ Arc, Mutex };
use std::cmp;

use std::fs::File;
use std::io::{ /*self, Read, */ Write };

static MSG_HISTORY_PATH: &str = "messages.json";
static CACHE_PATH: &str = "cache.json";

macro_rules! clone {
    ($var:ident) => {
        {
            let var = $var.lock().unwrap().clone();

            var
        }
    }
}

macro_rules! push {
    ($history:ident, $message:ident) => {
        {
            let mut messages = $history.lock().unwrap();
            messages.push($message.clone());
        }
    }
}

macro_rules! extend {
    ($history:ident, $other:ident) => {
        {
            let mut messages = $history.lock().unwrap();
            messages.extend($other.clone());
        }
    }
}


fn load_msg_history() -> Result<Vec<ChatMessage>, Box<dyn Error>>{
    let file   = File::open(MSG_HISTORY_PATH)?;
    let reader = std::io::BufReader::new(file);

    let data: Vec<ChatMessage> = serde_json::from_reader(reader)?;

    Ok(data)
}

fn save_msg_history(data: &Vec<ChatMessage>) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(MSG_HISTORY_PATH)?; // create / overwrite
    let json = serde_json::to_string_pretty(data)?;
    
    file.write_all(json.as_bytes())?;
    
    Ok(())
}

fn load_cache() -> Result<Cache, Box<dyn Error>>{
    let file   = File::open(MSG_HISTORY_PATH)?;
    let reader = std::io::BufReader::new(file);

    let data: Cache = serde_json::from_reader(reader)?;

    Ok(data)
}


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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cache {
    msg_id_max: u128,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1";
    let port = 1998;

    // load cache from json
    // let local: Cache = load_cache().unwrap();

    // Load message data from json
    let history  = Arc::new(Mutex::new(vec![]));

    let json_history = load_msg_history().unwrap();

    extend!(history, json_history);

    // Setup connection
    let listener = TcpListener::bind(format!("{addr}:{port}")).await?;
    log(format!("Server is acitve on {addr}:{port} !"));


    let (tx, _) = broadcast::channel::<String>(128);

    log(format!("now listening for new clients... "));
    'accept: loop {
        tokio::select! {
            res = listener.accept() => {
                let (socket, addr) = res?;//listener.accept().await?;
                log(format!("New connection: {addr}"));

                let tx = tx.clone();
                let rx = tx.subscribe();

                let history_clone = Arc::clone(&history);

                let mut net = Network {
                    socket: socket,
                    tx:     tx,
                    rx:     rx,
                    addr:   addr.to_string(),
                };

                tokio::spawn(async move {
                    handle_client(net, history_clone).await;
                });
            }

            _ = signal::ctrl_c() => {
                println!("terminating accept loop...");
                break 'accept;
            }
        }
    }

    let mut messages = history.lock().unwrap();
    match save_msg_history(&messages) {
        Ok(()) => {
            println!("Successfully saved message history to \"{}\"", MSG_HISTORY_PATH);
        }

        Err(e) => {
            let json = serde_json::to_string_pretty(&*messages).unwrap();
            println!("{}, dumping json in chat...\n{}", e, json);
            println!("Json dumped");
        }
    };

    Ok(())
}

struct Network {
    socket:  TcpStream, 
    tx:      broadcast::Sender<String>,
    rx:      broadcast::Receiver<String>,
    addr:    String,
}

pub async fn handle_client(
        mut net:     Network,
        history: Arc<Mutex<Vec<ChatMessage>>>
    )
{
    // Setup socket connection
    let (reader, mut writer) = net.socket.split();
    let mut reader           = BufReader::new(reader);


    // Get the username
    eprintln!("TODO: login logic");
    let mut username = String::new();
    reader.read_line(&mut username).await.unwrap();
    let username = username.trim().to_string();


    // Load old messages
    let messages = clone!(history);
    // let messages;
    // {
    //     messages = history.lock().unwrap().clone();
    // }

    let len        = messages.len();
    let batch_size = cmp::min(25, len); // -1 to not send 

    let batch      = &messages[len - batch_size..];

    // Send loaded messages
    for chat_msg in batch {
        let msg = chat_msg.json().unwrap();

        let _   = writer.write_all(msg.as_bytes()).await.unwrap();
        let _   = writer.write_all(b"\n").await.unwrap();
    }

    // Send join message
    let msg      = format!("User \"{}\" ({}) has connected to the chatroom.", username, net.addr);
    let join_msg = create_sys_msg(msg.clone());
    let json_msg = join_msg.json().unwrap();

    push!(history, join_msg);
    // {
    //     let mut messages = history.lock().unwrap();
    //     messages.push(join_msg.clone());
    // }

    net.tx.send(json_msg.clone()).unwrap();
    log(msg);

    let mut buffer_line = String::new();
    'msg_recv: loop {
        tokio::select!{
            res = reader.read_line(&mut buffer_line) => {
                if res.unwrap() == 0 {
                    let msg = format!("User \"{}\" ({}) was disconnected from the chatroom.", username, net.addr);
                    let disconnect_msg = create_sys_msg(msg.clone());
                    let json_msg = disconnect_msg.json().unwrap();
                    log(msg);
                    {
                        let mut messages = history.lock().unwrap();

                        messages.push(disconnect_msg.clone());
                    }

                    net.tx.send(json_msg).unwrap();
                    break 'msg_recv;
                }

                let msg = create_msg(
                    username.clone(),
                    buffer_line.trim().to_string(),
                    MessageType::UserMessage,
                );

                {
                    let mut messages = history.lock().unwrap();

                    messages.push(msg.clone());
                }

                log(format!("[ MESSAGE SENT ] <{}>: {}", msg.username, msg.content));

                net.tx.send(msg.json().unwrap()).unwrap();
                buffer_line.clear();
            }

            res = net.rx.recv() => {
                let msg = res.unwrap();

                let _ = writer.write_all(msg.as_bytes()).await.unwrap();
                let _ = writer.write_all(b"\n").await.unwrap();
            }

            // _ = shutdown_signal => {
            //     log("exiting accept loop...".to_string());
            //     break 'msg_recv;
            // }
        }
    }
}
