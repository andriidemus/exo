use super::app_db::{AppDB, Cell, Mode};
use super::message::{CellsMessage, Message};
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::style::{Color, Style};
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

    fn create_cell(&self, app_db: &mut AppDB) -> Result<()> {
        if let Some(current_id) = app_db.cells.get_current_cell_id() {
        } else {
            let cell = Cell::new();
            let cell_id = cell.id;
            app_db.cells.add(cell);
            app_db.cells.switch_cell(cell_id);
        }
        Ok(())
    }

    pub fn handle(&self, app_db: &mut AppDB, msg: Message) -> Result<()> {
        app_db.show_help = false; // make sure help is hidden immediately on any action
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
                CellsMessage::SetError(cell_id, error) => {
                    app_db.cells.set_error(cell_id, error);
                }
                CellsMessage::SaveCurrent => {
                    if let Some(current) = app_db.cells.get_current_cell_id() {
                        app_db
                            .cells
                            .set_code(current, app_db.cells.editor.lines().join("\n").to_string())
                    }
                }
                CellsMessage::Create => {
                    self.create_cell(app_db)?;
                }
            },
            Message::KeyPressed(key) => match app_db.mode {
                Mode::Navigate => match key.code {
                    KeyCode::Char('?') | KeyCode::F(1) => {
                        app_db.show_help = true;
                    }
                    KeyCode::Char('q') => {
                        app_db.quit = true;
                    }
                    KeyCode::Char('e') => {
                        self.handle(app_db, Message::Cells(CellsMessage::ExecuteCurrent))?;
                    }
                    KeyCode::Char('n') => {
                        self.handle(app_db, Message::Cells(CellsMessage::Create))?;
                    }
                    KeyCode::Enter | KeyCode::Left => {
                        app_db
                            .cells
                            .editor
                            .set_cursor_style(Style::from((Color::White, Color::Black)));
                        app_db.mode = Mode::EditCell;
                    }
                    _ => {}
                },
                Mode::EditCell => {
                    if key.code == KeyCode::Char('e')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        self.handle(app_db, Message::Cells(CellsMessage::ExecuteCurrent))?;
                    } else if key.code == KeyCode::Esc {
                        app_db
                            .cells
                            .editor
                            .set_cursor_style(Style::from((Color::White, Color::Gray)));
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
