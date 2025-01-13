use super::app_db::AppDB;
use super::message::{CellsMessage, Message};
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::sync::mpsc::Sender;
use std::time::Duration;
use uuid::Uuid;

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
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::Cells(CellsMessage::ExecuteCurrent))
        }
        _ => None,
    }
}

pub struct Handler {
    event_channel: Sender<Vec<Message>>,
    df_channel: Sender<(Uuid, String)>,
}

impl Handler {
    pub fn new(event_channel: Sender<Vec<Message>>, df_channel: Sender<(Uuid, String)>) -> Self {
        Self {
            event_channel,
            df_channel,
        }
    }

    pub fn handle(&self, model: &mut AppDB, msg: Message) -> Result<()> {
        match msg {
            Message::Quit => {}
            Message::Cells(cells_msg) => match cells_msg {
                CellsMessage::ExecuteCurrent => {
                    if let Some(cell_id) = model.cells.get_current_cell_id() {
                        if let Some(expr) = model.cells.get_code(&cell_id) {
                            self.df_channel.send((cell_id, expr)).unwrap();
                        }
                    }
                }
                CellsMessage::ClearCurrent => {}
                CellsMessage::SetResult(cell_id, result) => {
                    model.cells.set_result(cell_id, result);
                }
            },
        }
        Ok(())
    }
}
