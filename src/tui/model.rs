use crate::tui::model::RunningState::Running;

#[derive(Debug)]
pub struct Model {
    pub result: String,
    pub running_state: RunningState,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            result: "Press x to run query".to_string(),
            running_state: Running,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}
