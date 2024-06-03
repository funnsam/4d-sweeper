use std::{io::Write, thread::sleep, time::{self, UNIX_EPOCH}};
use crossterm::{*, event::*, style::{Color, Attribute, Stylize}};

mod minesweeper;
mod rand;
use minesweeper::*;
use rand::*;

const NAV: &[[(KeyCode, KeyModifiers); 2]] = &[
    [(KeyCode::Char('a'), KeyModifiers::NONE), (KeyCode::Char('d'), KeyModifiers::NONE)],
    [(KeyCode::Char('w'), KeyModifiers::NONE), (KeyCode::Char('s'), KeyModifiers::NONE)],
    [(KeyCode::Char('A'), KeyModifiers::SHIFT), (KeyCode::Char('D'), KeyModifiers::SHIFT)],
    [(KeyCode::Char('W'), KeyModifiers::SHIFT), (KeyCode::Char('S'), KeyModifiers::SHIFT)],
    [(KeyCode::Char('a'), KeyModifiers::CONTROL), (KeyCode::Char('d'), KeyModifiers::CONTROL)],
    [(KeyCode::Char('w'), KeyModifiers::CONTROL), (KeyCode::Char('s'), KeyModifiers::CONTROL)],
    [(KeyCode::Char('a'), KeyModifiers::ALT), (KeyCode::Char('d'), KeyModifiers::ALT)],
    [(KeyCode::Char('w'), KeyModifiers::ALT), (KeyCode::Char('s'), KeyModifiers::ALT)],
];

fn new_game() -> Game {
    Game::new(vec![4, 4, 4, 4], 6)
}

fn main() {
    seed(UNIX_EPOCH.elapsed().unwrap().as_secs() as u64);
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide, event::EnableMouseCapture).unwrap();

    let mut game = new_game();

    loop {
        if let Ok(key) = event::read() {
            match key {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    terminal::disable_raw_mode().unwrap();
                    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show, event::DisableMouseCapture).unwrap();
                    std::process::exit(0)
                },

                // navigation
                Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    modifiers,
                    ..
                }) if NAV.iter().take(game.size.len()).any(|(_, i)| i[0] == (code, modifiers)) => {
                    let i = NAV.iter().enumerate().find(|(_, i)| i[0] == (code, modifiers)).unwrap().0;
                    game.selected[i] = game.selected[i].saturating_sub(1);
                },
                Event::Key(KeyEvent {
                    code,
                    kind: KeyEventKind::Press,
                    modifiers,
                    ..
                }) if NAV.iter().take(game.size.len()).any(|(_, i)| i[1] == (code, modifiers)) => {
                    let i = NAV.iter().enumerate().find(|(_, i)| i[1] == (code, modifiers)).unwrap().0;
                    game.selected[i] = (game.selected[i] + 1).min(game.size[1] - 1);
                },

                // flagging and opening
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
                }) => game = new_game(),
                // TODO: mouse nav
                // Event::Mouse(MouseEvent {
                //     kind: MouseEventKind::Down(MouseButton::Left),
                //     column,
                //     row,
                //     ..
                // }) => game.selected = vec![
                //     (column as usize / 2).min(game.size[0] - 1),
                //     (row.saturating_sub(1) as usize).min(game.size[1] - 1),
                // ],
                Event::Resize(..) => queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap(),
                _ => continue,
            }
        } else {
            continue
        }

        render(&mut stdout, &game);
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

    stdout.flush().unwrap();
}

fn render_board(stdout: &mut std::io::Stdout, game: &Game) {
    for c in cells(&game.size) {
        let at = coord_of(&c, game);

        #[cfg(feature = "auto_reduce")]
        let minus = neighbours(&c, &game.size).filter(|c| game.get(&c).unwrap().flagged).count();

        let mut t = format!("{:^1$}", match game.get(&c).unwrap() {
            Cell { flagged: true, .. } => "\u{f024}".to_string(),
            Cell { typ: CellType::Mine, opened: true, .. } => "\u{f0dda}".to_string(),
            #[cfg(feature = "auto_reduce")]
            Cell { typ: CellType::Number(n), opened: true, .. } => (*n as isize - minus as isize).to_string(),
            #[cfg(not(feature = "auto_reduce"))]
            Cell { typ: CellType::Number(n), opened: true, .. } => n.to_string(),
            Cell { opened: false, .. } => "?".to_string()
        }, max_width(game.size.len())).with(match game.get(&c).unwrap() {
            #[cfg(feature = "auto_reduce")]
            Cell { typ: CellType::Number(n), opened: true, .. } if n.abs_diff(minus) == 0 => Color::Black,
            #[cfg(feature = "auto_reduce")]
            Cell { typ: CellType::Number(n), opened: true, .. } if n.checked_sub(minus).is_none() => Color::Red,
            #[cfg(not(feature = "auto_reduce"))]
            Cell { typ: CellType::Number(0), opened: true, .. } => Color::Black,
            Cell { typ: CellType::Mine, opened: true, .. } => Color::White,
            Cell { flagged: true, .. } => Color::DarkRed,
            Cell { opened: true, .. } => Color::DarkBlue,
            Cell { opened: false, .. } => Color::DarkGrey,
        });

        if is_neighbour_of(&c, &game.selected) {
            t = t.underlined().underline_white();
        }

        if game.selected == c {
            t = t.reset().reverse();
        }

        queue!(stdout,
            cursor::MoveTo(at.0, at.1 + 1),
            style::PrintStyledContent(t)
        ).unwrap();
    }
}

fn max_width(d: usize) -> usize {
    3_usize.pow(d as u32).ilog10() as usize + 1
}

fn coord_of(a: &[usize], g: &Game) -> (u16, u16) {
    let mut x = 0;
    let mut y = 0;
    let mut xm = 1;
    let mut ym = 1;
    let mut p = false;

    for (i, d) in a.iter().enumerate() {
        let c = if p { &mut y } else { &mut x };
        let cm = if p { &mut ym } else { &mut xm };

        *c += d * (*cm + (i >> 1));
        *cm = g.size[i];

        p ^= true;
    }

    ((x * (max_width(a.len()) + 1)) as u16, y as u16)
}
