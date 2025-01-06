use anyhow::Result;
use ratatui::widgets::Paragraph;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Frame, Terminal,
};
use std::{io::stdout, panic};

use super::handler;
use super::model::{Model, RunningState};

fn init_terminal() -> Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

fn view(model: &mut Model, frame: &mut Frame) {
    frame.render_widget(Paragraph::new(model.result.clone()), frame.area());
}

pub async fn start() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut model = Model::default();

    while model.running_state != RunningState::Done {
        terminal.draw(|f| view(&mut model, f))?;

        let mut current_msg = handler::handle_event(&model)?;

        while current_msg.is_some() {
            current_msg = handler::update(&mut model, current_msg.unwrap()).await?;
        }
    }

    restore_terminal()?;
    Ok(())
}
