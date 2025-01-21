use super::message::{CellsMessage, Message};
use super::state::{Cell, CellStatus, ConfirmDialog, ConfirmDialogButton, Mode, State};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use std::sync::mpsc::Sender;
use tui_textarea::TextArea;
use uuid::Uuid;

pub struct Handler {
    df_channel: Sender<(Uuid, String)>,
}

impl Handler {
    pub fn new(df_channel: Sender<(Uuid, String)>) -> Self {
        Self { df_channel }
    }

    fn create_cell(&self, state: &mut State) -> Result<()> {
        let cell = Cell::new();
        let cell_id = cell.id;
        state.cells.all.insert(cell.id, cell);

        let index = state.cells.current_cell_index().map(|i| i + 1).unwrap_or(0);
        state.cells.order.insert(index, cell_id);

        self.switch_cell(state, cell_id);
        state
            .cells
            .editor
            .set_cursor_style(Style::from((Color::White, Color::Gray)));

        Ok(())
    }

    fn switch_cell(&self, state: &mut State, cell_id: Uuid) {
        state.cells.current_cell_id = Some(cell_id);
        let code = state.cells.current_mut().and_then(|c| c.code.clone());
        state.cells.editor = TextArea::from(code.unwrap_or_default().lines());
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Color::Gray);
        state.cells.editor.set_block(block);
    }

    fn handle_cell_messages(&self, state: &mut State, msg: CellsMessage) -> Result<()> {
        match msg {
            CellsMessage::ExecuteCurrent => {
                if let Some(cell) = state.cells.current_mut() {
                    if let Some(expr) = &cell.code {
                        cell.status = CellStatus::Running;
                        self.df_channel.send((cell.id, expr.clone()))?;
                    }
                }
            }
            CellsMessage::SetResult(cell_id, result) => {
                if let Some(cell) = state.cells.all.get_mut(&cell_id) {
                    cell.result = Some(result);
                    cell.status = CellStatus::Finished
                }
            }
            CellsMessage::SetError(cell_id, error) => {
                if let Some(cell) = state.cells.all.get_mut(&cell_id) {
                    cell.error = Some(error);
                    cell.status = CellStatus::Failed
                }
            }
            CellsMessage::SaveCurrent => {
                let lines = state.cells.editor.lines().join("\n").to_string();
                if let Some(cell) = state.cells.current_mut() {
                    cell.code = Some(lines)
                }
            }
            CellsMessage::Create => {
                self.create_cell(state)?;
                state
                    .cells
                    .editor
                    .set_cursor_style(Style::from((Color::White, Color::Black)));
                state.mode = Mode::EditCell;
            }
            CellsMessage::DeleteCurrent => {
                if let Some(index) = state.cells.current_cell_index() {
                    let new_current_index = {
                        if index + 1 < state.cells.order.len() {
                            Some(index)
                        } else if index > 0 {
                            Some(index - 1)
                        } else {
                            None
                        }
                    };
                    let id = state.cells.order.remove(index);
                    state.cells.all.remove(&id);

                    if let Some(id) = new_current_index.map(|i| state.cells.order[i]) {
                        self.switch_cell(state, id);
                    } else {
                        state.cells.current_cell_id = None;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_popup_messages(&self, state: &mut State, key: KeyEvent) -> Result<()> {
        if let Some(popup) = &state.popup {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    match popup.active_button {
                        ConfirmDialogButton::Yes => {
                            self.handle(state, popup.message.clone())?;
                        }
                        ConfirmDialogButton::No => {}
                    };
                    state.popup = None;
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    state.popup = None;
                }
                KeyCode::Char('y') => {
                    self.handle(state, popup.message.clone())?;
                    state.popup = None;
                }
                KeyCode::Right => {
                    if popup.active_button == ConfirmDialogButton::Yes {
                        let mut p = (*popup).clone();
                        p.active_button = ConfirmDialogButton::No;
                        state.popup.replace(p);
                    }
                }
                KeyCode::Left => {
                    if popup.active_button == ConfirmDialogButton::No {
                        let mut p = (*popup).clone();
                        p.active_button = ConfirmDialogButton::Yes;
                        state.popup.replace(p);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_navigate_messages(&self, state: &mut State, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('?') | KeyCode::F(1) => {
                state.show_help = true;
            }
            KeyCode::Char('q') => {
                self.handle(state, Message::ConfirmQuit)?;
            }
            KeyCode::Char('x') => {
                self.handle(state, Message::Cells(CellsMessage::ExecuteCurrent))?;
            }
            KeyCode::Char('n') => {
                self.handle(state, Message::Cells(CellsMessage::Create))?;
            }
            KeyCode::Char('d') => {
                state.popup = Some(ConfirmDialog {
                    message: Message::Cells(CellsMessage::DeleteCurrent),
                    body: "Delete current cell?".to_string(),
                    active_button: ConfirmDialogButton::Yes,
                });
            }
            KeyCode::Enter | KeyCode::Left | KeyCode::Char('h') => {
                state
                    .cells
                    .editor
                    .set_cursor_style(Style::from((Color::White, Color::Black)));
                state.mode = Mode::EditCell;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(index) = state.cells.current_cell_index() {
                    if index > 0 {
                        let new_index = index - 1;
                        self.switch_cell(state, state.cells.order[new_index])
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(index) = state.cells.current_cell_index() {
                    if index + 1 < state.cells.order.len() {
                        let new_index = index + 1;
                        self.switch_cell(state, state.cells.order[new_index])
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_messages(&self, state: &mut State, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Char('x') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.handle(state, Message::Cells(CellsMessage::SaveCurrent))?;
            self.handle(state, Message::Cells(CellsMessage::ExecuteCurrent))?;
        } else if key.code == KeyCode::Esc {
            state
                .cells
                .editor
                .set_cursor_style(Style::from((Color::White, Color::Gray)));
            state.mode = Mode::Navigate;
            self.handle(state, Message::Cells(CellsMessage::SaveCurrent))?;
        } else {
            state.cells.editor.input(key);
        }
        Ok(())
    }

    pub fn handle(&self, state: &mut State, msg: Message) -> Result<()> {
        state.show_help = false; // make sure help is hidden immediately on any action
        match msg {
            Message::ConfirmQuit => {
                state.popup = Some(ConfirmDialog {
                    message: Message::Quit,
                    body: "Are you sure you want to quit?".to_string(),
                    active_button: ConfirmDialogButton::Yes,
                })
            }
            Message::Quit => {
                state.quit = true;
            }
            Message::Cells(cells_msg) => self.handle_cell_messages(state, cells_msg)?,
            Message::KeyPressed(key) => {
                if state.popup.is_some() {
                    self.handle_popup_messages(state, key)?;
                } else {
                    match state.mode {
                        Mode::Navigate => self.handle_navigate_messages(state, key)?,
                        Mode::EditCell => self.handle_edit_messages(state, key)?,
                    }
                }
            }
        }
        Ok(())
    }
}
