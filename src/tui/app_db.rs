#[derive(Debug)]
pub struct AppDB {
    pub result: String,
}

impl Default for AppDB {
    fn default() -> Self {
        Self {
            result: "Press x to run query".to_string(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}
