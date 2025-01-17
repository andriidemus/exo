use super::app_db::{AppDB, Cell, Mode};
use super::message::{CellsMessage, Message};
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use std::sync::mpsc::Sender;
use std::time::Duration;
use tui_textarea::TextArea;
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
    df_channel: Sender<(Uuid, String)>,
}

impl Handler {
    pub fn new(df_channel: Sender<(Uuid, String)>) -> Self {
        Self { df_channel }
    }

    fn create_cell(&self, app_db: &mut AppDB) -> Result<()> {
        let cell = Cell::new();
        let cell_id = cell.id;
        app_db.cells.add(cell);

        let index = app_db.cells.current_cell_index.map(|i| i + 1).unwrap_or(0);
        app_db.cells.current_cell_index = Some(index);
        app_db.cells.order.insert(index, cell_id);

        self.switch_cell(app_db, cell_id);
        app_db
            .cells
            .editor
            .set_cursor_style(Style::from((Color::White, Color::Gray)));

        Ok(())
    }

    pub fn switch_cell(&self, app_db: &mut AppDB, cell_id: Uuid) {
        app_db.cells.current_cell_id = Some(cell_id);
        let code = app_db.cells.get_code(&cell_id);
        app_db.cells.editor = TextArea::from(code.unwrap_or_default().lines());
        let editor_title = format!(
            "Cell {} of {}",
            app_db.cells.current_cell_index.unwrap() + 1,
            app_db.cells.cells.len()
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .title(editor_title)
            .border_style(Color::Black);
        app_db.cells.editor.set_block(block);
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
                            self.df_channel.send((cell_id, expr))?;
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
                    app_db
                        .cells
                        .editor
                        .set_cursor_style(Style::from((Color::White, Color::Black)));
                    app_db.mode = Mode::EditCell;
                }
                CellsMessage::DeleteCurrent => {
                    if let Some(index) = app_db.cells.current_cell_index {
                        let new_current_index = {
                            if index + 1 < app_db.cells.order.len() {
                                Some(index)
                            } else if index > 0 {
                                Some(index - 1)
                            } else {
                                None
                            }
                        };
                        let id = app_db.cells.order.remove(index);
                        app_db.cells.cells.remove(&id);

                        if let Some(id) = new_current_index.map(|i| app_db.cells.order[i]) {
                            self.switch_cell(app_db, id);
                        } else {
                            app_db.cells.current_cell_id = None;
                        }
                    }
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
                    KeyCode::Char('d') => {
                        self.handle(app_db, Message::Cells(CellsMessage::DeleteCurrent))?;
                    }
                    KeyCode::Enter | KeyCode::Left => {
                        app_db
                            .cells
                            .editor
                            .set_cursor_style(Style::from((Color::White, Color::Black)));
                        app_db.mode = Mode::EditCell;
                    }
                    KeyCode::Up => {
                        if let Some(index) = app_db.cells.current_cell_index {
                            if index > 0 {
                                let new_index = index - 1;
                                app_db.cells.current_cell_index = Some(new_index);
                                self.switch_cell(app_db, app_db.cells.order[new_index])
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Some(index) = app_db.cells.current_cell_index {
                            if index + 1 < app_db.cells.order.len() {
                                let new_index = index + 1;
                                app_db.cells.current_cell_index = Some(new_index);
                                self.switch_cell(app_db, app_db.cells.order[new_index])
                            }
                        }
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
