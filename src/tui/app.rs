use super::app_db::AppDB;
use super::handler;
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
use std::sync::mpsc;
use std::{io::stdout, panic};

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

fn view(model: &AppDB, frame: &mut Frame) {
    frame.render_widget(Paragraph::new(model.result.clone()), frame.area());
}

pub async fn start() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut model = AppDB::default();
    let (sender, receiver) = mpsc::channel::<Vec<Message>>();

    let event_loop = tokio::spawn(async move {
        terminal.draw(|f| view(&model, f)).unwrap();
        while let Ok(msgs) = receiver.recv() {
            match msgs[..] {
                [Message::Quit] => {
                    break;
                }
                _ => {
                    for msg in msgs {
                        handler::update(&mut model, msg).await.unwrap();
                    }
                    terminal.draw(|f| view(&model, f)).unwrap();
                }
            }
        }
    });

    let sender_from_ue = sender.clone();
    let ui_event_loop = tokio::spawn(async move {
        loop {
            if let Some(msg) = handler::user_event().unwrap() {
                let quit = msg == Message::Quit;
                sender_from_ue.send(vec![msg]).unwrap();
                if quit {
                    break;
                }
            }
        }
    });

    event_loop.await?;
    ui_event_loop.await?;
    restore_terminal()?;

    Ok(())
}
