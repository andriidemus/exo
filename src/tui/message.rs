use crossterm::event::KeyEvent;
use datafusion::arrow::array::RecordBatch;
use uuid::Uuid;

#[derive(PartialEq)]
pub enum Message {
    Cells(CellsMessage),
    KeyPressed(KeyEvent),
    Quit,
}

#[derive(PartialEq)]
pub enum CellsMessage {
    ExecuteCurrent,
    ClearCurrent,
    SaveCurrent,
    SetResult(Uuid, Vec<RecordBatch>),
    SetError(Uuid, String),
    Create,
}
