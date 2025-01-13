use super::app_db::{AppDB, Cell};
use super::handler;
use crate::core::{DataFusionSession, LocalDataFusionSession};
use crate::tui::handler::Handler;
use crate::tui::message::{CellsMessage, Message};
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
    if let Some(cell_id) = app_db.cells.get_current_cell_id() {
        if let Some(cell) = app_db.cells.get_cell(&cell_id) {
            let test = format!("{:?} : {:?} : {:?}", &cell.state, &cell.code, &cell.result);
            frame.render_widget(Paragraph::new(test), frame.area());
        }
    }
}

pub async fn start() -> Result<()> {
    install_panic_hook();
    let mut terminal = init_terminal()?;
    let mut app_db = AppDB::default();

    // test code
    let mut cell1 = Cell::new();
    cell1.code = Some("select now();".to_string());
    app_db.cells.switch_cell(cell1.id);
    app_db.cells.add(cell1);
    // test code

    let (sender, receiver) = mpsc::channel::<Vec<Message>>();
    let sender_from_ue = sender.clone();

    let (df_sender, df_receiver) = mpsc::channel::<(Uuid, String)>();
    let handler = Handler::new(sender.clone(), df_sender);

    let _df_loop = tokio::spawn(async move {
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
            match msgs[..] {
                [Message::Quit] => {
                    break;
                }
                _ => {
                    for msg in msgs {
                        handler.handle(&mut app_db, msg).unwrap();
                    }
                    terminal.draw(|f| view(&app_db, f)).unwrap();
                }
            }
        }
    });

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
