use super::app_db::{AppDB, Mode};
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
                return Ok(Some(Message::KeyPressed(key)));
            }
        }
    }
    Ok(None)
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

    pub fn handle(&self, app_db: &mut AppDB, msg: Message) -> Result<()> {
        match msg {
            Message::Quit => {
                app_db.quit = true;
            }
            Message::Cells(cells_msg) => match cells_msg {
                CellsMessage::ExecuteCurrent => {
                    if let Some(cell_id) = app_db.cells.get_current_cell_id() {
                        if let Some(expr) = app_db.cells.get_code(&cell_id) {
                            self.df_channel.send((cell_id, expr)).unwrap();
                        }
                    }
                }
                CellsMessage::ClearCurrent => {}
                CellsMessage::SetResult(cell_id, result) => {
                    app_db.cells.set_result(cell_id, result);
                }
                CellsMessage::SaveCurrent => {
                    if let Some(current) = app_db.cells.get_current_cell_id() {
                        app_db
                            .cells
                            .set_code(current, app_db.cells.editor.lines().join("\n").to_string())
                    }
                }
            },
            Message::KeyPressed(key) => match app_db.mode {
                Mode::Navigate => match key.code {
                    KeyCode::Char('q') => {
                        app_db.quit = true;
                    }
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.handle(app_db, Message::Cells(CellsMessage::ExecuteCurrent))?;
                    }
                    KeyCode::Enter => app_db.mode = Mode::EditCell,
                    _ => {}
                },
                Mode::EditCell => {
                    if key.code == KeyCode::Esc {
                        app_db.mode = Mode::Navigate;
                        self.handle(app_db, Message::Cells(CellsMessage::SaveCurrent))?;
                    } else {
                        app_db.cells.editor.input(key);
                    }
                }
            },
        }
        Ok(())
    }
}
