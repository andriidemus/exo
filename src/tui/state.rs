use crate::tui::message::Message;
use datafusion::arrow::array::RecordBatch;
use std::collections::HashMap;
use tui_textarea::TextArea;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct State<'a> {
    pub cells: Cells<'a>,
    pub mode: Mode,
    pub quit: bool,
    pub show_help: bool,
    pub popup: Option<ConfirmDialog>,
}

#[derive(Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    Navigate,
    EditCell,
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub message: Message,
    pub body: String,
    pub active_button: ConfirmDialogButton,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ConfirmDialogButton {
    Yes,
    No,
}

#[derive(Debug)]
pub struct Cells<'a> {
    pub editor: TextArea<'a>,
    pub all: HashMap<Uuid, Cell>,
    pub order: Vec<Uuid>,
    pub current_cell_id: Option<Uuid>,
}

impl Default for Cells<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Cells<'_> {
    pub fn new() -> Self {
        Self {
            editor: Default::default(),
            all: HashMap::new(),
            order: Vec::new(),
            current_cell_id: Default::default(),
        }
    }

    pub fn current_mut(&mut self) -> Option<&mut Cell> {
        self.current_cell_id.and_then(|id| self.all.get_mut(&id))
    }

    pub fn current(&self) -> Option<&Cell> {
        self.current_cell_id.and_then(|id| self.all.get(&id))
    }

    pub fn current_cell_index(&self) -> Option<usize> {
        self.current_cell_id
            .and_then(|id| self.order.iter().position(|item| *item == id))
    }
}

#[derive(Debug)]
pub struct Cell {
    pub id: Uuid,
    pub code: Option<String>,
    pub result: Option<Vec<RecordBatch>>,
    pub error: Option<String>,
    pub status: CellStatus,
}

#[derive(Debug, Clone)]
pub enum CellStatus {
    Clean,
    Running,
    Finished,
    Failed,
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

impl Cell {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            code: None,
            result: None,
            error: None,
            status: CellStatus::Clean,
        }
    }
}
