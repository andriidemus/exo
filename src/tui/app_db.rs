use datafusion::arrow::array::RecordBatch;
use std::collections::HashMap;
use tui_textarea::TextArea;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct AppDB<'a> {
    pub cells: Cells<'a>,
    pub mode: Mode,
    pub quit: bool,
}

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Navigate,
    EditCell,
}

#[derive(Debug)]
pub struct Cells<'a> {
    pub editor: TextArea<'a>,
    cells: HashMap<Uuid, Cell>,
    current_cell_id: Option<Uuid>,
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
            cells: HashMap::new(),
            current_cell_id: Default::default(),
        }
    }

    pub fn add(&mut self, cell: Cell) {
        self.cells.insert(cell.id, cell);
    }

    pub fn remove(&mut self, cell_id: &Uuid) {
        self.cells.remove(cell_id);
    }

    pub fn set_code(&mut self, cell_id: Uuid, code: String) {
        self.cells
            .entry(cell_id)
            .and_modify(|e| e.code = Some(code));
    }

    pub fn set_status(&mut self, cell_id: Uuid, state: CellState) {
        self.cells.entry(cell_id).and_modify(|e| e.state = state);
    }

    pub fn set_result(&mut self, cell_id: Uuid, result: Vec<RecordBatch>) {
        self.cells.entry(cell_id).and_modify(|e| {
            e.result = Some(result);
            e.state = CellState::Finished
        });
    }

    pub fn set_failure(&mut self, cell_id: Uuid, result: Vec<RecordBatch>) {
        self.cells.entry(cell_id).and_modify(|e| {
            e.result = Some(result);
            e.state = CellState::Finished
        });
    }

    pub fn switch_cell(&mut self, cell_id: Uuid) {
        self.current_cell_id = Some(cell_id);
        let code = self.get_code(&cell_id);
        self.editor = TextArea::from(code.unwrap_or(String::new()).lines());
    }

    pub fn get_current_cell_id(&self) -> Option<Uuid> {
        self.current_cell_id
    }

    pub fn get_code(&self, cell_id: &Uuid) -> Option<String> {
        self.cells.get(cell_id).and_then(|c| c.code.clone())
    }

    pub fn get_cell(&self, cell_id: &Uuid) -> Option<&Cell> {
        self.cells.get(cell_id)
    }
}

#[derive(Debug)]
pub struct Cell {
    pub id: Uuid,
    pub code: Option<String>,
    pub result: Option<Vec<RecordBatch>>,
    pub error: Option<String>,
    pub state: CellState,
}

#[derive(Debug)]
pub enum CellState {
    Clean,
    Running,
    Finished,
    Failed,
    Aborted,
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
            state: CellState::Clean,
        }
    }
}
