use super::message::Message;
use super::model::{Model, RunningState};
use crate::core::{LocalSession, Session};
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use std::sync::{Arc, Mutex};
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

pub async fn update(model: Arc<Mutex<Model>>, msg: Message) -> Result<()> {
    match msg {
        Message::RunTestQuery => {
            // Just a test. Real session should be persisted.
            let session = LocalSession::default();
            let result = session.sql("select now();").await?;
            model.lock().unwrap().result = format!("{:?}", result.first().unwrap().columns());
        }
        Message::Quit => {
            model.lock().unwrap().running_state = RunningState::Done;
        }
    }
    Ok(())
}
