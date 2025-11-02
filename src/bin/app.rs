use crate::common::{/* MessageType, create_msg, */ ChatMessage};

use tokio::{
    sync::Mutex,
    io::AsyncBufReadExt,
};

use std::{
    error::Error,
    sync::Arc,
    io::{ self, Write, Stdout },
};

use tui::{
    text::{ Spans, Span },
    backend::{ CrosstermBackend, Backend },
    widgets::{ Block, Borders, Paragraph, Wrap },
    layout::{ Layout, Constraint, Direction },
    style::{ Style, Color, Modifier },
    Terminal,
    Frame
};

use crossterm::{
    event::{ DisableMouseCapture, EnableMouseCapture },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};

// TODO: 
// [ ] 1. Make the viewport follow the latest message if diff() < 3.

pub fn diff(x: usize, y: usize) -> usize {
    if x > y { return x-y; } else { return y-x; }
}

pub struct App<'a, W: Write> {
        terminal:    Terminal<CrosstermBackend<W>>,
    pub messages:    Vec<ChatMessage>,
    pub input:       String,
        msg_history: Vec<Spans<'a>>,
    pub scroll:      usize,
        chat_height: usize,
}

impl<'a> App<'a, Stdout> {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        
        let backend     = CrosstermBackend::new(stdout); 

        let terminal    = Terminal::new(backend)?;
        let input       = String::new();
        let messages    = Vec::new();
        let msg_history = Vec::new();
        let scroll      = 0;
        let chat_height = 0;

        Ok(Self {
            terminal,
            input,
            messages,
            msg_history,
            scroll,
            chat_height,
        })
    }
}

impl<'a, W: Write> App<'a, W> {
    // CHAT
    pub fn push_msg(&mut self, msg: ChatMessage) {
        let length = self.messages.len();
        if diff(length, self.scroll) < 3 && length < self.chat_height {
            self.scroll = 0;
        }

        self.messages.push(msg);
    }

    // TUI
    pub async fn render(&mut self) -> Result<(), Box<dyn Error>> {
        let messages    = &self.messages;
        let input       = &self.input;
        let scroll      =  self.scroll;
        let chat_height = &mut self.chat_height;

        self.terminal.draw(|f| {
            ui(f, messages, input, scroll, chat_height);
        })?;

        Ok(())
    }



}

impl<'a, W: Write> Drop for App<'a, W> {
    fn drop(&mut self) {
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).expect("Failed to restore terminal");
        
        self.terminal.show_cursor().expect("Failed to show cursor");
    }
}

fn ui<B: Backend>(f: &mut Frame<'_, B>, messages: &[ChatMessage], input: &str, scroll: usize, chat_height: &mut usize) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(0),
                Constraint::Length(3),
            ]
        )
        .split(size);
    
    let mut message_spans: Vec<Spans> = vec![];
    for msg in messages {
        message_spans.push(
            Spans::from(vec![
                Span::styled(
                    format!(" {} ", msg.timestamp),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("{} ", msg.username),
                    Style::default()
                    .fg(
                        if msg.username == "System" {
                            Color::Red
                        } else {
                            Color::Cyan
                        }
                    )
                    .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} \n", msg.content),
                    Style::default().fg(Color::LightGreen),
                )
            ])
        );
    }

    let message_paragraph = Paragraph::new(message_spans)
        .block(Block::default().title("Cicada Cat Chat").borders(Borders::ALL))
        .scroll((scroll as u16, 0));

    f.render_widget(message_paragraph, chunks[0]);
    *chat_height = chunks[0].y as usize;

    let input_area = Paragraph::new(input)
        .wrap(Wrap { trim: true })
        .block(Block::default()
            .title("Input")
            .borders(Borders::ALL)
        );

    f.render_widget(input_area, chunks[1]);
}
