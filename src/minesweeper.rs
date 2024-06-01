use crate::rand::rand;

pub enum GameState {
    Normal, Dead, Won
}

#[derive(Clone)]
pub struct Cell {
    pub typ: CellType,
    pub flagged: bool,
    pub opened: bool,
}

#[derive(Clone)]
pub enum CellType {
    Number(usize),
    Mine,
}

pub struct Game {
    pub cells: Vec<Vec<Cell>>,
    pub selected: (u16, u16),
    pub state: GameState,
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let mut a = Self {
            cells: vec![vec![Cell {
                typ: CellType::Number(0),
                flagged: false,
                opened: false,
            }; width]; height],

            selected: (0, 0),
            state: GameState::Normal
        };
        a.scramble(mines);
        a.update();
        a
    }
    pub fn open(&mut self) {
        self._open(self.selected);
        if matches!(self.state, GameState::Normal) && self.cells.iter().flatten().fold(true, |a, c| a & (c.opened || matches!(c.typ, CellType::Mine))) {
            self.on_win()
        }
    }
    fn _open(&mut self, a: (u16, u16)) {
        let c = &mut self.cells[a.1 as usize][a.0 as usize];
        if !c.flagged && !c.opened {
            if matches!(c.typ, CellType::Mine) {
                self.on_lose();
                return;
            }
            c.opened = true;
            if matches!(c.typ, CellType::Number(0)) {
                for y in a.1 as i16 - 1 ..= a.1 as i16 + 1 {
                    if y.is_negative() || y as usize >= self.cells.len() { continue; }
                    for x in a.0 as i16 - 1 ..= a.0 as i16 + 1 {
                        if x.is_negative() || x as usize >= self.cells[0].len() { continue; }
                        self._open((x as u16, y as u16));
                    }
                }
            }
        }
    }

    fn on_win(&mut self) {
        self.state = GameState::Won;
        self.reveal_all();
    }

    fn on_lose(&mut self) {
        self.state = GameState::Dead;
        self.reveal_all();
    }

    fn reveal_all(&mut self) {
        for y in self.cells.iter_mut() {
            for x in y.iter_mut() {
                x.opened = true;
                x.flagged = false;
            }
        }
    }

    pub fn flag(&mut self) {
        self.cells[self.selected.1 as usize][self.selected.0 as usize].flagged ^= true;
    }

    fn scramble(&mut self, mines: usize) {
        for _ in 0..mines {
            let mut okay = false;
            while !okay {
                let x = rand() % self.cells[0].len() as u64;
                let y = rand() % self.cells.len() as u64;
                let x = x as usize;
                let y = y as usize;
                if !matches!(self.cells[y][x].typ, CellType::Mine) {
                    self.cells[y][x].typ = CellType::Mine;
                    okay = true
                }
            }
        }
    }

    fn update(&mut self) {
        for i in 0..self.cells.len() {
            for j in 0..self.cells[0].len() {
                if matches!(self.cells[i][j].typ, CellType::Mine) { continue }
                let mut mines = 0;
                for k in i as i32 - 1 ..= i as i32 + 1 {
                    for l in j as i32 - 1 ..= j as i32 + 1 {
                        let ke = self.cells.get(k as usize).unwrap_or(&vec![]).clone();
                        if matches!(*ke.get(l as usize).map_or(&CellType::Number(0), |i| &i.typ), CellType::Mine) {
                            mines += 1;
                        }
                    }
                }

                self.cells[i][j].typ = CellType::Number(mines);
            }
        }
    }
}
