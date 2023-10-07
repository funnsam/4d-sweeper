use std::{io, time, thread};

use tui::{backend::{CrosstermBackend, Backend}, *, widgets::{*, canvas::*}, layout::*, style::*};
use crossterm::{
    event::{*, self},
    execute,
    terminal::*,
};

mod deassem;
mod emu;
mod args;
use crate::emu::emulate;
use crate::args::*;
use std::process::exit;

pub struct EmuState {
    disassem: String,
    disassem_scroll: usize,
    vertical: bool,
    number_out: u8,

    pc  : u16,
    sp  : u8,
    regs: [u8; 3],
    fr  : u8,
    mem : [u8; 0b1111_1111_1111],
    rom : [u32; 0b1111_1111_1111_1111],
    status: EmuStatus
}

#[derive(Debug, Clone, PartialEq)]
pub enum EmuStatus {
    Starting, Normal, Halted, Waiting(u8), Stalled(u8)
}

static mut STATE: EmuState = EmuState {
    disassem: String::new(),
    disassem_scroll: 0,
    vertical: false,
    number_out: 1,

    pc      : 0,
    sp      : 0,
    regs    : [0; 3],
    fr      : 0,
    mem     : [0; 0b1111_1111_1111],
    rom     : [0; 0b1111_1111_1111_1111],
    status  : EmuStatus::Starting
};

fn main() -> Result<(), Box<io::Error>> {
    let arg = match Args::parse() {
        ParseResult::Ok(a) => a,
        ParseResult::Err(err) => {
            println!("\x1b[1;31mError: {err}\x1b[0m");
            exit(-1);
        },
        ParseResult::Help(msg) => {
            println!("{msg}");
            exit(0);
        }
    };

    let prog = match std::fs::read(&arg.file) {
        Ok(a) => a,
        Err(a) => {
            println!("\x1b[1;31mError: Couldn't read \"{}\": {}\x1b[0m", &arg.file, a);
            exit(-1);
        }
    };

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    unsafe {
        STATE.disassem = deassem::deassemble(&prog);
        let mut a = prog.chunks(3).map(|a| u32::from_le_bytes([a[0], a[1], a[2], 0])).collect::<Vec<u32>>();
        a.extend(std::iter::repeat(0).take(0b1111_1111_1111_1111 - a.len()));
        STATE.rom = a.try_into().unwrap();
    }

    std::thread::spawn(move || loop {
        let started_at = time::SystemTime::now();
        if event::poll(std::time::Duration::from_nanos(1)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key {
                    KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                        .. 
                    } => {
                        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
                        std::process::exit(0)
                    },
                    KeyEvent {
                        code: KeyCode::Insert,
                        modifiers: KeyModifiers::NONE,
                        .. 
                    } => unsafe { STATE.disassem_scroll = STATE.disassem_scroll.checked_sub(1).unwrap_or(0); },
                    KeyEvent {
                        code: KeyCode::Delete,
                        modifiers: KeyModifiers::NONE,
                        .. 
                    } => unsafe { STATE.disassem_scroll += 1 },
                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::ALT,
                        ..
                    } => unsafe { STATE.vertical ^= true },
                    _ => ()
                }
            }
        }

        terminal.draw(|f| {
            unsafe {
                render(f, &STATE)
            }
        }).unwrap();
        let elapsed = started_at.elapsed().unwrap().as_millis();
        thread::sleep(time::Duration::from_millis(60_u128.checked_sub(elapsed).unwrap_or(0) as u64));
    });

    let mspc = ((1.0 / arg.speed) * 1_000_000.0) as u128;
    loop {
        let started_at = time::SystemTime::now();
        unsafe {
            emulate(&mut STATE);
        }
        let elapsed = started_at.elapsed().unwrap().as_micros();
        thread::sleep(time::Duration::from_micros(mspc.checked_sub(elapsed).unwrap_or(0) as u64));
    }
}

fn render<B: Backend>(f: &mut Frame<B>, state: &EmuState) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(match state.vertical {
            true  => Direction::Vertical,
            false => Direction::Horizontal,
        })
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30)
            ].as_ref()
        )
        .split(size);

    let deassembly = List::new(
            state.disassem.lines()
            .skip(state.disassem_scroll)
            .take(chunks[0].height as usize)
            .map(|a| ListItem::new(a)).collect::<Vec<ListItem>>()
        ).block(
            Block::default()
                .title("Deassembly")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
        ).highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut s = ListState::default();
    s.select(Some(state.pc as usize - state.disassem_scroll));
    f.render_stateful_widget(deassembly, chunks[0], &mut s);

    let display = Canvas::default()
        .block(
            Block::default()
                .title("Display")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
            )
        .x_bounds([-(chunks[1].width  as f64) / 2.0, chunks[1].width  as f64 / 2.0])
        .y_bounds([-(chunks[1].height as f64) / 2.0, chunks[1].height as f64 / 2.0])
        .paint(|ctx| {
            ctx.print(0.0, 0.0, format!("{}", state.number_out))
        });
<<<<<<< Updated upstream
    f.render_widget(display, chunks[1])
=======
    f.render_widget(display, chunks[1]);

    let stat = List::new(
        [
            ListItem::new(format!("R1: {}", state.regs[0])),
            ListItem::new(format!("R2: {}", state.regs[1])),
            ListItem::new(format!("R3: {}", state.regs[2])),
            ListItem::new(format!("PC: {}", state.pc)),
            ListItem::new(format!("SP: {}", state.sp)),
            ListItem::new(format!("Status: {:?}", state.status)),
            ListItem::new(format!("Flags:\n    Z: {:01b}\n    C: {:01b}", state.fr & 1, state.fr >> 1)),
        ]
    )
        .block(
            Block::default()
                .title("Emulator State")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
        );
    f.render_widget(stat, chunks[2]);
>>>>>>> Stashed changes
}
