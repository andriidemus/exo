use super::app_db::{AppDB, CellState, Mode};
use super::handler;
use crate::core::{DataFusionSession, LocalDataFusionSession};
use crate::tui::handler::Handler;
use crate::tui::message::{CellsMessage, Message};
use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
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

fn render_status_line(app_db: &AppDB, frame: &mut Frame, rect: Rect) {
    let style = Style::new().bg(Color::Gray);
    let mode_str = match app_db.mode {
        Mode::Navigate => "NAVI",
        Mode::EditCell => "EDIT",
    };
    let cell_no = app_db
        .cells
        .current_cell_index
        .map(|i| format!("{}/{}", i + 1, app_db.cells.cells.len()))
        .unwrap_or_default();

    let cell_state = app_db
        .cells
        .current_cell_id
        .map(|id| match app_db.cells.get_cell(&id).unwrap().state {
            CellState::Clean => "Not Executed",
            CellState::Running => "Running",
            CellState::Finished => "Finished",
            CellState::Failed => "Failed",
        })
        .unwrap_or_default();

    let status = format!(" {} • {} • {}", mode_str, cell_no, cell_state);

    frame.render_widget(
        Paragraph::new(status).style(style).wrap(Wrap::default()),
        rect,
    )
}

fn view(app_db: &AppDB, frame: &mut Frame) {
    let mut show_help = app_db.show_help;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    render_status_line(app_db, frame, layout[1]);

    if let Some(cell_id) = app_db.cells.get_current_cell_id() {
        let cell_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        if let Some(cell) = app_db.cells.get_cell(&cell_id) {
            frame.render_widget(&app_db.cells.editor, cell_layout[0]);

            match cell.state {
                CellState::Clean => {
                    frame.render_widget(
                        Paragraph::new("Press <ctrl+e> to execute current cell"),
                        cell_layout[1],
                    );
                }
                CellState::Running => {
                    frame.render_widget(Paragraph::new(""), cell_layout[1]);
                }
                CellState::Finished => {
                    let result = format!("{:?}", &cell.result);
                    frame.render_widget(Paragraph::new(result), cell_layout[1]);
                }
                CellState::Failed => {
                    frame.render_widget(
                        Paragraph::new(cell.error.clone().unwrap_or(String::new()))
                            .style(Style::new().fg(Color::Red))
                            .wrap(Wrap::default()),
                        cell_layout[1],
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
