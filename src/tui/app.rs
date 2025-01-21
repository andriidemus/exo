use super::state::State;
use super::view;
use crate::core::{DataFusionSession, LocalDataFusionSession};
use crate::tui::handler::Handler;
use crate::tui::message::{CellsMessage, Message};
use anyhow::Result;
use crossterm::event;
use crossterm::event::Event;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::{io::stdout, panic};
use uuid::Uuid;

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

fn user_event() -> Result<Option<Message>> {
    if event::poll(Duration::from_millis(10))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(Some(Message::KeyPressed(key)));
            }
        }
    }
    Ok(None)
}

pub async fn start() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut state = State::default();

    let (sender, receiver) = mpsc::channel::<Vec<Message>>();
    let sender_from_ue = sender.clone();

    let (df_sender, df_receiver) = mpsc::channel::<(Uuid, String)>();
    let handler = Handler::new(df_sender);

    // Processing all DataFusion operations async
    let df_loop = tokio::spawn(async move {
        let df = LocalDataFusionSession::new();

        while let Ok((uuid, expr)) = df_receiver.recv() {
            let messages = match df.sql(&expr).await {
                Ok(result) => vec![Message::Cells(CellsMessage::SetResult(uuid, result))],
                Err(err) => vec![Message::Cells(CellsMessage::SetError(
                    uuid,
                    err.to_string(),
                ))],
            };

            sender.send(messages).unwrap();
        }
    });

    // Main even(message) processing loop
    let event_loop = tokio::spawn(async move {
        // We need to draw UI on start
        terminal.draw(|f| view::render(&state, f)).unwrap();
        while let Ok(msgs) = receiver.recv() {
            for msg in msgs {
                handler.handle(&mut state, msg).unwrap();
            }
            if state.quit {
                return;
            }
            // In order to reduce CPU usage, we do not re-draw UI until a message has been received
            terminal.draw(|f| view::render(&state, f)).unwrap();
        }
    });

    let exit_flag = Arc::new(Mutex::new(false));
    let exit_flag_clone = exit_flag.clone();
    let ui_event_loop = tokio::spawn(async move {
        loop {
            if let Some(msg) = user_event().unwrap() {
                if sender_from_ue.send(vec![msg]).is_err() {
                    break;
                }
            } else if *exit_flag_clone.lock().unwrap() {
                break;
            }
        }
    });

    event_loop.await?;
    *exit_flag.lock().unwrap() = true;
    restore_terminal()?;
    df_loop.await?;
    ui_event_loop.await?;

    Ok(())
}
