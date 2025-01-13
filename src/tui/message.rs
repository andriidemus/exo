use uuid::Uuid;

#[derive(PartialEq)]
pub enum Message {
    Cells(CellsMessage),
    Quit,
}

#[derive(PartialEq)]
pub enum CellsMessage {
    ExecuteCurrent,
    ClearCurrent,
    SetResult(Uuid, serde_json::Value),
}
