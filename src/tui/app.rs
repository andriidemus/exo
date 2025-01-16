use super::app_db::{AppDB, Cell};
use super::handler;
use crate::core::{DataFusionSession, LocalDataFusionSession};
use crate::tui::handler::Handler;
use crate::tui::message::{CellsMessage, Message};
use anyhow::Result;
use ratatui::layout::{Constraint, Direction, Layout};
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
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());
    if let Some(cell_id) = app_db.cells.get_current_cell_id() {
        if let Some(cell) = app_db.cells.get_cell(&cell_id) {
            frame.render_widget(&app_db.cells.editor, layout[0]);

            let result = format!("{:?}", &cell.result);
            frame.render_widget(Paragraph::new(result), layout[1]);
        }
    }
}

pub async fn start() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut app_db = AppDB::default();

    // test code
    let mut cell1 = Cell::new();
    let cell_id = cell1.id.clone();
    cell1.code = Some("select now();".to_string());
    app_db.cells.add(cell1);
    app_db.cells.switch_cell(cell_id);
    // test code

    let (sender, receiver) = mpsc::channel::<Vec<Message>>();
    let sender_from_ue = sender.clone();

    let (df_sender, df_receiver) = mpsc::channel::<(Uuid, String)>();
    let handler = Handler::new(sender.clone(), df_sender);

    let df_loop = tokio::spawn(async move {
        let df = LocalDataFusionSession::new();

        while let Ok((uuid, expr)) = df_receiver.recv() {
            let result = df.sql(&expr).await.unwrap();
            sender
                .send(vec![Message::Cells(CellsMessage::SetResult(uuid, result))])
                .unwrap();
        }
    });

    let event_loop = tokio::spawn(async move {
        terminal.draw(|f| view(&app_db, f)).unwrap();
        while let Ok(msgs) = receiver.recv() {
            for msg in msgs {
                handler.handle(&mut app_db, msg).unwrap();
            }
            if app_db.quit {
                break;
            }
            terminal.draw(|f| view(&app_db, f)).unwrap();
        }
    });

    let ui_event_loop = tokio::spawn(async move {
        loop {
            // TODO: fix immediate quitting
            if let Some(msg) = handler::user_event().unwrap() {
                sender_from_ue.send(vec![msg]).unwrap();
            }
        }
    });

    event_loop.await?;
    ui_event_loop.await?;
    df_loop.await?;
    restore_terminal()?;

    Ok(())
}
