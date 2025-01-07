use super::handler;
use super::model::{Model, RunningState};
use crate::tui::message::Message;
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
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::{io::stdout, panic, thread};

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

fn view(model: Arc<Mutex<Model>>, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new(model.lock().unwrap().result.clone()),
        frame.area(),
    );
}

pub fn start() -> Result<()> {
    install_panic_hook();
    let terminal = Arc::new(Mutex::new(init_terminal()?));
    let model = Arc::new(Mutex::new(Model::default()));
    let (sender, receiver) = mpsc::channel::<Message>();
    let model2 = model.clone();
    let terminal2 = terminal.clone();
    let h = tokio::spawn(async move {
        while let Ok(msg) = receiver.recv() {
            handler::update(model2.clone(), msg).await.unwrap();
            // todo: redraw not more often than 60 times per sec
            terminal2
                .lock()
                .unwrap()
                .draw(|f| view(model2.clone(), f))
                .unwrap();
        }
    });

    let ev_sender = sender.clone();
    let eh = tokio::spawn(async move {
        loop {
            if let Some(msg) = handler::user_event().unwrap() {
                ev_sender.send(msg).unwrap();
            }
        }
    });

    terminal.lock().unwrap().draw(|f| view(model.clone(), f))?;
    while model.clone().lock().unwrap().running_state != RunningState::Done {
        thread::sleep(Duration::from_millis(50));
    }
    h.abort();
    eh.abort();

    restore_terminal()?;
    Ok(())
}
