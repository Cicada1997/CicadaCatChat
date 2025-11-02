#![allow(dead_code)]
#![allow(unused_labels)]
#![allow(unused_imports)]

pub mod common;
pub use common::{/* MessageType, create_msg, */ ChatMessage};

mod app;
use app::App;

use tokio::{
    net::TcpStream,
    sync::Mutex,
    io::{ AsyncBufReadExt, AsyncWriteExt, BufReader },
};

use std::{
    env,
    error::Error,
    sync::Arc,
    time::Duration,
};

use crossterm::event::{ 
    self, 
    KeyEvent, 
    KeyCode 
};


// MAIN
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // ESTABLISHING CONNECTION
    let server_addr = env::args()
        .nth(1)
        .expect("args: [{server_ip}] [username]");

    let username    = env::args()
        .nth(2)
        .expect("args: [{server_ip}] [username]");


    let socket = TcpStream::connect(format!("{server_addr}:1997")).await.unwrap();
    let (reader, writer) = socket.into_split();

    let     writer       = Arc::new(Mutex::new(writer));
    let     writer_clone = Arc::clone(&writer);

    let     reader       = BufReader::new(reader);
    let mut lines        = reader.lines();

    {
        let mut writer_locked = writer.lock().await;
        writer_locked.write_all(format!("{username}\n").as_bytes()).await?;
    }

    // SETUP SHARED STATE & TUI
    let app       = Arc::new(Mutex::new(App::new().unwrap()));
    let app_recv  = Arc::clone(&app);
    let app_event = Arc::clone(&app);
    let app_ui    = Arc::clone(&app);


    // UPDATE APP ATTRIBUTES
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(msg) = serde_json::from_str::<ChatMessage>(&line) {
                let mut app = app_recv.lock().await;
                app.push_msg(msg);
            }
        }
    });

    // UPDATE UI
    tokio::spawn(async move {
        'rendering: loop {
            {
                let mut app = app_ui.lock().await;

                if let Err(e) = app.render().await {
                    eprintln!("Error rendering UI: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_millis(8)).await; // About 120 FPS; cap the intensity
        }
    });

    // MAIN LOOP
    'running: loop {
        if event::poll(Duration::from_millis(20)).unwrap() {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {

                match code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        let mut app = app_event.lock().await;

                        if app.input.len() > 0 {
                            let mut writer = writer_clone.lock().await;

                            let msg = format!("{}\n", app.input.clone());
                            writer.write_all(msg.as_bytes()).await?;

                            app.input.clear();
                        }
                    },

                    KeyCode::Up => {
                        let mut app = app_event.lock().await;
                        app.scroll -= if app.scroll > 0 { 1 } else { 0 };
                    },

                    KeyCode::Down => {
                        let mut app = app_event.lock().await;
                        app.scroll += if app.scroll < app.messages.len() -1 { 1 } else { 0 };
                    },

                    KeyCode::Char(key) => {
                        let mut app = app_event.lock().await;
                        app.input.push(key);
                    },

                    KeyCode::Backspace => {
                        let mut app = app_event.lock().await;
                        app.input.pop();
                    },

                    _ => {}
                }
            }
        }
    }

    Ok(())
} 
