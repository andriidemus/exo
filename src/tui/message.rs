use crossterm::event::KeyEvent;
use datafusion::arrow::array::RecordBatch;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone)]
pub enum Message {
    Cells(CellsMessage),
    KeyPressed(KeyEvent),
    ConfirmQuit,
    Quit,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CellsMessage {
    ExecuteCurrent,
    SaveCurrent,
    SetResult(Uuid, Vec<RecordBatch>),
    SetError(Uuid, String),
    Create(Option<String>),
    DeleteCurrent,
}
