use std::{io::Write, thread::sleep, time::{self, UNIX_EPOCH}};
use crossterm::{*, event::*, style::{Color, Attribute, Stylize}};

mod minesweeper;
mod rand;
use minesweeper::*;
use rand::*;

fn main() {
    srand(time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u64);
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Show, cursor::SetCursorStyle::SteadyBlock, event::EnableMouseCapture).unwrap();

    let mut game = Game::new(9, 9, 10);

    let mspc = ((1.0 / 10.0) * 1_000_000.0) as u128;
    loop {
        let started_at = time::SystemTime::now();
        if event::poll(time::Duration::from_nanos(1)).unwrap() {
            if let Ok(key) = event::read() {
                match key {
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        .. 
                    }) => {
                        terminal::disable_raw_mode().unwrap();
                        execute!(stdout, terminal::LeaveAlternateScreen, cursor::SetCursorStyle::DefaultUserShape, event::DisableMouseCapture).unwrap();
                        std::process::exit(0)
                    },
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('w') | KeyCode::Up,
                        kind: KeyEventKind::Press,
                        ..
                    }) => game.selected[1] = game.selected[1].saturating_sub(1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('s') | KeyCode::Down,
                        kind: KeyEventKind::Press,
                        ..
                    }) => game.selected[1] = (game.selected[1] + 1).min(game.size[1] - 1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('a') | KeyCode::Left,
                        kind: KeyEventKind::Press,
                        ..
                    }) => game.selected[0] = game.selected[0].saturating_sub(1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d') | KeyCode::Right,
                        kind: KeyEventKind::Press,
                        ..
                    }) => game.selected[0] = (game.selected[0] + 1).min(game.size[0] - 1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('j'),
                        kind: KeyEventKind::Press,
                        ..
                    }) => game.open(),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('k'),
                        kind: KeyEventKind::Press,
                        ..
                    }) => game.flag(),
                    Event::Key(KeyEvent {
                        code: KeyCode::Backspace,
                        kind: KeyEventKind::Press,
                        ..
                    }) => game = Game::new(9, 9, 10),
                    Event::Mouse(MouseEvent {
                        kind: MouseEventKind::Down(MouseButton::Left),
                        column,
                        row,
                        ..
                    }) => game.selected = vec![
                        (column as usize / 2).min(game.size[0] - 1),
                        (row.saturating_sub(1) as usize).min(game.size[1] - 1),
                    ],
                    Event::Resize(..) => execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap(),
                    _ => ()
                }
            }
        }
        render(&mut stdout, &game);
        let elapsed = started_at.elapsed().unwrap().as_micros();
        sleep(time::Duration::from_micros(mspc.checked_sub(elapsed).unwrap_or(0) as u64));
    }
}

fn render(stdout: &mut std::io::Stdout, game: &Game) {
    render_board(stdout, game);
    match game.state {
        GameState::Normal => {
            queue!(stdout,
                cursor::MoveTo(0, 0),
                style::PrintStyledContent(
                    "\u{f01f5}"
                        .with(Color::Yellow)
                        .attribute(Attribute::Bold)
                ),
            ).unwrap();
        },
        GameState::Dead => {
            queue!(stdout,
                cursor::MoveTo(0, 0),
                style::PrintStyledContent(
                    "\u{f069b}"
                        .with(Color::Yellow)
                        .attribute(Attribute::Bold)
                ),
            ).unwrap();
        },
        GameState::Won => {
            queue!(stdout,
                cursor::MoveTo(0, 0),
                style::PrintStyledContent(
                    "\u{f04e0}"
                        .with(Color::Yellow)
                        .attribute(Attribute::Bold)
                ),
            ).unwrap();
        },
    }
    queue!(stdout, cursor::MoveTo(game.selected[0] as u16 * 2, game.selected[1] as u16 + 1)).unwrap();
    stdout.flush().unwrap();
}

fn render_board(stdout: &mut std::io::Stdout, game: &Game) {
    for y in 0..game.size[1] {
        for x in 0..game.size[0] {
            queue!(stdout,
                cursor::MoveTo((x * 2) as u16, y as u16 + 1),
                style::PrintStyledContent(
                    match game.get(&[x, y]).unwrap() {
                        Cell { flagged: true, .. } => "\u{f024}".to_string(),
                        Cell { typ: CellType::Mine, opened: true, .. } => "\u{f0dda}".to_string(),
                        Cell { typ: CellType::Number(n), opened: true, .. } => n.to_string(),
                        Cell { opened: false, .. } => "?".to_string()
                    }.with(match game.get(&[x, y]).unwrap() {
                        Cell { typ: CellType::Number(0), opened: true, .. } => Color::Black,
                        Cell { typ: CellType::Mine, opened: true, .. } => Color::White,
                        Cell { flagged: true, .. } => Color::DarkRed,
                        Cell { opened: true, .. } => Color::DarkBlue,
                        Cell { opened: false, .. } => Color::DarkGrey,
                    })
                )
            ).unwrap();
        }
    }
}
