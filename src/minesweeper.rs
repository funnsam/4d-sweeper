use crate::rand::rand;

pub enum GameState {
    Normal, Dead, Won
}

#[derive(Clone)]
pub enum Cell {
    Number(usize),
    Mine,
    Flagged(usize),
    FlaggedMine
}

pub struct Game {
    pub dists: Vec<Vec<Cell>>,
    pub opened: Vec<Vec<bool>>,

    pub selected: (u16, u16),
    pub state: GameState
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let mut a = Self {
            dists : vec![vec![Cell::Number(0); width]; height],
            opened: vec![vec![false; width]; height],

            selected: (0, 0),
            state: GameState::Normal
        };
        a.scramble(mines);
        a.update();
        a
    }
    pub fn open(&mut self) {
        self._open(self.selected);
        if matches!(self.state, GameState::Normal) && self.opened.iter().flatten().zip(self.dists.iter().flatten()).fold(true, |a, (b, c)| a & (*b || matches!(c, Cell::Mine | Cell::FlaggedMine))) {
            self.on_win()
        }
    }
    fn _open(&mut self, a: (u16, u16)) {
        let d = &self.dists[a.1 as usize][a.0 as usize];
        if !matches!(d, Cell::Flagged(_)) && !self.opened[a.1 as usize][a.0 as usize] {
            if matches!(d, Cell::Mine) {
                self.on_lose();
                return;
            }
            self.opened[a.1 as usize][a.0 as usize] = true;
            if matches!(d, Cell::Number(0)) {
                for y in a.1 as i16 - 1 ..= a.1 as i16 + 1 {
                    if y.is_negative() || y as usize >= self.dists.len() { continue; }
                    for x in a.0 as i16 - 1 ..= a.0 as i16 + 1 {
                        if x.is_negative() || x as usize >= self.dists[0].len() { continue; }
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
        for y in self.opened.iter_mut() {
            for x in y.iter_mut() {
                *x = true
            }
        }
        for y in self.dists.iter_mut() {
            for x in y.iter_mut() {
                match x {
                    Cell::Flagged(n) => *x = Cell::Number(*n),
                    Cell::FlaggedMine => *x = Cell::Mine,
                    _ => (),
                }
            }
        }
    }
    pub fn flag(&mut self) {
        let c = &mut self.dists[self.selected.1 as usize][self.selected.0 as usize];
        match c {
            Cell::Number(n) => *c = Cell::Flagged(*n),
            Cell::Flagged(n) => *c = Cell::Number(*n),
            Cell::FlaggedMine => *c = Cell::Mine,
            Cell::Mine => *c = Cell::FlaggedMine,
        }
    }
    fn scramble(&mut self, mines: usize) {
        for _ in 0..mines {
            let mut okay = false;
            while !okay {
                let x = rand() % self.dists[0].len() as u64;
                let y = rand() % self.dists.len() as u64;
                let x = x as usize;
                let y = y as usize;
                if !matches!(self.dists[y][x], Cell::Mine) {
                    self.dists[y][x] = Cell::Mine;
                    okay = true
                }
            }
        }
    }
    fn update(&mut self) {
        for i in 0..self.dists.len() {
            for j in 0..self.dists[0].len() {
                if matches!(self.dists[i][j], Cell::Mine) { continue }
                let mut mines = 0;
                for k in i as i32 - 1 ..= i as i32 + 1 {
                    for l in j as i32 - 1 ..= j as i32 + 1 {
                        let ke = self.dists.get(k as usize).unwrap_or(&vec![]).clone();
                        if matches!(*ke.get(l as usize).unwrap_or(&Cell::Number(0)), Cell::Mine) {
                            mines += 1;
                        }
                    }
                }
                self.dists[i][j] = Cell::Number(mines)
            }
        }
    }
}
