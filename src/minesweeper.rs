use crate::rand::rand;

pub enum GameState {
    Normal, Dead, Won
}

pub struct Game {
    pub dists: Vec<Vec<u8>>,
    pub opened: Vec<Vec<bool>>,

    pub selected: (u16, u16),
    pub state: GameState
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let mut a = Self {
            dists : vec![vec![0; width]; height],
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
        if matches!(self.state, GameState::Normal) && self.opened.iter().flatten().zip(self.dists.iter().flatten()).fold(true, |a, (b, c)| a & (*b || c & 0b1111 == 10)) {
            self.state = GameState::Won;
        }
    }
    fn _open(&mut self, a: (u16, u16)) {
        let d = self.dists[a.1 as usize][a.0 as usize];
        if d & 0b1_0000 == 0 && !self.opened[a.1 as usize][a.0 as usize] {
            if d == 10 {
                self.on_lose();
                return;
            }
            self.opened[a.1 as usize][a.0 as usize] = true;
            if d == 0 {
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
    fn on_lose(&mut self) {
        self.state = GameState::Dead;
        for y in self.opened.iter_mut() {
            for x in y.iter_mut() {
                *x = true
            }
        }
        for y in self.dists.iter_mut() {
            for x in y.iter_mut() {
                *x &= 0b1111
            }
        }
    }
    pub fn flag(&mut self) {
        self.dists[self.selected.1 as usize][self.selected.0 as usize] ^= 0b1_0000
    }
    fn scramble(&mut self, mines: usize) {
        for _ in 0..mines {
            let mut okay = false;
            while !okay {
                let x = rand() % self.dists[0].len() as u64;
                let y = rand() % self.dists.len() as u64;
                let x = x as usize;
                let y = y as usize;
                if self.dists[y][x] != 10 {
                    self.dists[y][x] = 10;
                    okay = true
                }
            }
        }
    }
    fn update(&mut self) {
        for i in 0..self.dists.len() {
            for j in 0..self.dists[0].len() {
                if self.dists[i][j] == 10 { continue; }
                let mut mines = 0;
                for k in i as i32 - 1 ..= i as i32 + 1 {
                    for l in j as i32 - 1 ..= j as i32 + 1 {
                        let ke = self.dists.get(k as usize).unwrap_or(&vec![]).clone();
                        if *ke.get(l as usize).unwrap_or(&0) & 0b1111 == 10 {
                            mines += 1;
                        }
                    }
                }
                self.dists[i][j] = mines | (self.dists[i][j] & 0b1_0000)
            }
        }
    }
}
