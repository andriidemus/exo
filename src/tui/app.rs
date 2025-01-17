use super::app_db::{AppDB, CellState};
use super::handler;
use crate::core::{DataFusionSession, LocalDataFusionSession};
use crate::tui::handler::Handler;
use crate::tui::message::{CellsMessage, Message};
use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Frame, Terminal,
};
use std::sync::{mpsc, Arc, Mutex};
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

fn view(app_db: &AppDB, frame: &mut Frame) {
    let mut show_help = app_db.show_help;

    if let Some(cell_id) = app_db.cells.get_current_cell_id() {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());

        if let Some(cell) = app_db.cells.get_cell(&cell_id) {
            frame.render_widget(&app_db.cells.editor, layout[0]);

            match cell.state {
                CellState::Clean => {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("No result")
                        .border_style(Color::Black);
                    frame.render_widget(
                        Paragraph::new("Press <ctrl+e> to execute current cell").block(block),
                        layout[1],
                    );
                }
                CellState::Running => {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Running...")
                        .border_style(Color::Black);
                    frame.render_widget(Paragraph::new("").block(block), layout[1]);
                }
                CellState::Finished => {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Finished")
                        .border_style(Color::Black);
                    let result = format!("{:?}", &cell.result);
                    frame.render_widget(Paragraph::new(result).block(block), layout[1]);
                }
                CellState::Failed => {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Failed")
                        .border_style(Color::Black);
                    frame.render_widget(
                        Paragraph::new(cell.error.clone().unwrap_or(String::new()))
                            .block(block)
                            .wrap(Wrap::default()),
                        layout[1],
                    );
                }
            }
        }
    } else {
        show_help = true;
    }

    if show_help {
        let block = Block::default().borders(Borders::ALL);
        frame.render_widget(
            Paragraph::new("Press 'n' to create a cell").block(block),
            frame.area(),
        );
    }
}

pub async fn start() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut app_db = AppDB::default();

    let (sender, receiver) = mpsc::channel::<Vec<Message>>();
    let sender_from_ue = sender.clone();

    let (df_sender, df_receiver) = mpsc::channel::<(Uuid, String)>();
    let handler = Handler::new(df_sender);

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

    let event_loop = tokio::spawn(async move {
        terminal.draw(|f| view(&app_db, f)).unwrap();
        while let Ok(msgs) = receiver.recv() {
            for msg in msgs {
                handler.handle(&mut app_db, msg).unwrap();
            }
            if app_db.quit {
                return;
            }
            terminal.draw(|f| view(&app_db, f)).unwrap();
        }
    });

    let exit_flag = Arc::new(Mutex::new(false));
    let exit_flag_clone = exit_flag.clone();
    let ui_event_loop = tokio::spawn(async move {
        loop {
            if let Some(msg) = handler::user_event().unwrap() {
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
