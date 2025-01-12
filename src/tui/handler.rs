use super::app_db::AppDB;
use super::message::Message;
use crate::core::{LocalSession, Session};
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use std::time::Duration;

pub fn user_event() -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(10))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

pub fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('x') => Some(Message::RunTestQuery),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

pub async fn update(model: &mut AppDB, msg: Message) -> Result<()> {
    match msg {
        Message::RunTestQuery => {
            // Just a test. Real session should be persisted.
            let session = LocalSession::default();
            let result = session.sql("select now();").await?;
            model.result = format!("{:?}", result.first().unwrap().columns());
        }
        Message::Quit => {}
    }
    Ok(())
}
