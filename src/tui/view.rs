use crate::tui::app_db::{AppDB, CellState, ConfirmDialogButton, Mode};
use datafusion::arrow::error::ArrowError;
use datafusion::arrow::util::display::{ArrayFormatter, FormatOptions};
use indoc::indoc;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::{widgets, Frame};

fn render_status_line(app_db: &AppDB, frame: &mut Frame, rect: Rect) {
    let style = Style::new().bg(Color::Gray);
    let mode_str = match app_db.mode {
        Mode::Navigate => "🚀 NAVI",
        Mode::EditCell => "✏️ EDIT",
    };
    let cell_no = app_db
        .cells
        .current_cell_index
        .map(|i| format!("{}/{}", i + 1, app_db.cells.cells.len()));

    let cell_state =
        app_db
            .cells
            .current_cell_id
            .map(|id| match app_db.cells.get_cell(&id).unwrap().state {
                CellState::Clean => "Not Executed",
                CellState::Running => "Running",
                CellState::Finished => "Finished",
                CellState::Failed => "Failed",
            });

    let mut parts = vec![mode_str.to_string()];
    if let Some(val) = cell_no {
        parts.push(val);
    }
    if let Some(val) = cell_state {
        parts.push(val.to_string());
    }

    let mut status = parts.join(" • ");
    status.insert(0, ' ');

    frame.render_widget(
        Paragraph::new(status).style(style).wrap(Wrap::default()),
        rect,
    )
}

fn centered_area(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

fn render_popup(app_db: &AppDB, frame: &mut Frame) {
    if let Some(popup) = &app_db.popup {
        let pad = 1u16;

        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::all())
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::new(pad, pad, pad, pad));

        let text = popup.body.clone();
        let area = centered_area(frame.area(), text.len() as u16 + 10, 7);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(6), Constraint::Length(1)])
            .split(block.inner(area));

        let buttons_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .split(layout[1]);

        let text_widget = Paragraph::new(popup.body.clone()).alignment(Alignment::Center);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        frame.render_widget(text_widget, layout[0]);

        let active = Style::from((Color::White, Color::DarkGray));
        let not_active = Style::from((Color::Black, Color::Gray));

        let btn_yes = Paragraph::new("Yes").alignment(Alignment::Center).style(
            if popup.active_button == ConfirmDialogButton::Yes {
                active
            } else {
                not_active
            },
        );
        let btn_no = Paragraph::new("No").alignment(Alignment::Center).style(
            if popup.active_button == ConfirmDialogButton::No {
                active
            } else {
                not_active
            },
        );

        frame.render_widget(btn_yes, buttons_layout[1]);
        frame.render_widget(btn_no, buttons_layout[3]);
    }
}

fn render_help(frame: &mut Frame) {
    let help = indoc! {"
            n        - create new cell
            d        - delete selected cell
            ↑, k     - select previous cell
            ↓, j     - select next cell
            ←, h, ↵  - edit selected cell
            e        - execute selected cell
            q        - quit
            ?, F1    - show this help
            "};

    let height = help.lines().count() + 2;
    let width = help
        .lines()
        .map(|l| l.len())
        .max()
        .map(|l| l + 4)
        .unwrap_or_default();
    let area = centered_area(frame.area(), width as u16, height as u16);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::new(1u16, 1u16, 0, 0));
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(help).block(block), area);
}

fn render_table(app_db: &AppDB, frame: &mut Frame, area: Rect) {
    if let Some(ref result) = app_db
        .cells
        .get_current_cell_id()
        .and_then(|id| app_db.cells.get_cell(&id))
        .and_then(|cell| cell.result.as_ref())
    {
        frame.render_widget(Clear, area);

        if result.is_empty() {
            frame.render_widget(
                Paragraph::new("SQL statement did not return any data"),
                area,
            );
            return;
        }

        let batch = result.first().unwrap();

        let header = batch
            .schema()
            .fields
            .iter()
            .map(|f| f.name().clone())
            .map(widgets::Cell::from)
            .collect::<widgets::Row>()
            .height(1)
            .style(Style::from((Color::Black, Color::Gray)).add_modifier(Modifier::BOLD));

        let mut rows: Vec<widgets::Row> = vec![];

        for batch in result.iter() {
            let formatters = batch
                .columns()
                .iter()
                .map(|c| ArrayFormatter::try_new(c.as_ref(), &FormatOptions::default()))
                .collect::<anyhow::Result<Vec<_>, ArrowError>>()
                .unwrap();

            for i in 0..batch.num_rows() {
                let mut cells = Vec::new();
                for formatter in &formatters {
                    cells.push(formatter.value(i).to_string());
                }
                let bg = if i % 2 == 0 {
                    Color::White
                } else {
                    Color::Gray
                };
                let table_row = widgets::Row::new(cells).style(Style::new().bg(bg));
                rows.push(table_row);
            }
        }

        let table = widgets::Table::default()
            .header(header)
            .rows(rows)
            .row_highlight_style(Style::from(Color::Red));
        frame.render_widget(table, area);
    }
}

pub fn render(app_db: &AppDB, frame: &mut Frame) {
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
                    let text = indoc! {"
                        You can write and execute SQL in the DataFusion dialect.
                        
                        Official reference: https://datafusion.apache.org/user-guide/sql/index.html

                        To execute cell, press <Alt/Option + Enter>
                        You also may press <Esc> to back to the Navigation mode, and then press <e>
                    "};

                    frame.render_widget(Paragraph::new(text), cell_layout[1]);
                }
                CellState::Running => {
                    let area = centered_area(cell_layout[1], 30, 1);
                    frame.render_widget(Paragraph::new("Running 🏃‍➡️🏃‍♂️‍➡️🏃‍♀️‍➡️ "), area);
                }
                CellState::Finished => {
                    render_table(app_db, frame, cell_layout[1]);
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
        render_help(frame);
    }

    if app_db.popup.is_some() {
        render_popup(app_db, frame);
    }
}
