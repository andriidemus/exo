use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct AppDB {
    pub cells: Cells,
}

#[derive(Debug)]
pub struct Cells {
    cells: HashMap<Uuid, Cell>,
    current_cell_id: Option<Uuid>,
}

impl Default for Cells {
    fn default() -> Self {
        Self::new()
    }
}

impl Cells {
    pub fn new() -> Self {
        Self {
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

    pub fn set_result(&mut self, cell_id: Uuid, result: serde_json::Value) {
        self.cells.entry(cell_id).and_modify(|e| {
            e.result = Some(result);
            e.state = CellState::Finished
        });
    }

    pub fn set_failure(&mut self, cell_id: Uuid, result: serde_json::Value) {
        self.cells.entry(cell_id).and_modify(|e| {
            e.result = Some(result);
            e.state = CellState::Finished
        });
    }

    pub fn switch_cell(&mut self, cell_id: Uuid) {
        self.current_cell_id = Some(cell_id);
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
    pub result: Option<serde_json::Value>,
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
