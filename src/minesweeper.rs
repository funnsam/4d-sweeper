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
    pub cells: Vec<Cell>,
    pub size: Vec<usize>,

    pub selected: Vec<usize>,
    pub state: GameState,
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        let mut a = Self {
            cells: vec![Cell {
                typ: CellType::Number(0),
                flagged: false,
                opened: false,
            }; width * height],

            size: vec![width, height],

            selected: vec![0; 2],
            state: GameState::Normal
        };
        a.scramble(mines);
        a.update();
        a
    }

    pub fn get_mut(&mut self, at: &[usize]) -> Option<&mut Cell> {
        self.get(at).map(|i| unsafe {
            #[allow(mutable_transmutes)]
            core::mem::transmute(i)
        })
    }

    pub fn get(&self, at: &[usize]) -> Option<&Cell> {
        assert_eq!(at.len(), self.size.len());

        let mut index = 0;
        for (i, s) in at.iter().rev().zip(self.size.iter().rev()) {
            if s <= i { return None; }

            index *= *s;
            index += i;
        }

        Some(&self.cells[index])
    }

    pub fn open(&mut self) {
        self._open(&self.selected.clone());
        if matches!(self.state, GameState::Normal) && self.cells.iter().fold(true, |a, c| a & (c.opened || matches!(c.typ, CellType::Mine))) {
            self.on_win()
        }
    }

    fn _open(&mut self, a: &[usize]) {
        let c = self.get_mut(a).unwrap();
        if !c.flagged && !c.opened {
            if matches!(c.typ, CellType::Mine) {
                self.on_lose();
                return;
            }
            c.opened = true;

            if matches!(c.typ, CellType::Number(0)) {
                for i in neighbours(a, &self.size) {
                    self._open(&i);
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
        for i in self.cells.iter_mut() {
            i.opened = true;
            i.flagged = false;
        }
    }

    pub fn flag(&mut self) {
        self.get_mut(&self.selected.clone()).unwrap().flagged ^= true;
    }

    fn scramble(&mut self, mines: usize) {
        for _ in 0..mines {
            let mut okay = false;
            while !okay {
                let x = rand() % self.size[0] as u64;
                let y = rand() % self.size[1] as u64;
                let x = x as usize;
                let y = y as usize;
                if !matches!(self.get(&[x, y]).unwrap().typ, CellType::Mine) {
                    self.get_mut(&[x, y]).unwrap().typ = CellType::Mine;
                    okay = true
                }
            }
        }
    }

    fn update(&mut self) {
        for i in cells(&self.size.clone()) {
            if matches!(self.get(&i).unwrap().typ, CellType::Mine) { continue }

            let mut mines = 0;
            for i in neighbours(&i, &self.size) {
                mines += matches!(self.get(&i).unwrap().typ, CellType::Mine) as usize;
            }

            self.get_mut(&i).unwrap().typ = CellType::Number(mines);
        }
    }
}

fn neighbours(at: &[usize], dim: &[usize]) -> impl Iterator<Item = Vec<usize>> {
    fn make(mut v: Vec<core::ops::Range<usize>>) -> Box<dyn Iterator<Item = Vec<usize>>> {
        if v.len() == 1 {
            Box::new(v.swap_remove(0).map(|i| vec![i]))
        } else {
            Box::new(v.swap_remove(0).flat_map(move |i| make(v.clone()).map(move |mut k| {
                k.insert(0, i);
                k
            })))
        }
    }

    let ranges = at.iter().enumerate().map(|(i, p)| if *p > 0 {
        p - 1..(p + 2).min(dim[i])
    } else {
        0..2
    }).collect::<Vec<core::ops::Range<usize>>>();

    make(ranges)
}

fn cells<'a>(dim: &'a [usize]) -> impl Iterator<Item = Vec<usize>> + 'a {
    fn make<'a>(v: &'a [usize]) -> Box<dyn Iterator<Item = Vec<usize>> + 'a> {
        if v.len() == 1 {
            Box::new((0..v[0]).map(|i| vec![i]))
        } else {
            Box::new((0..v[0]).flat_map(|i| make(&v[1..]).map(move |mut j| {
                j.insert(0, i);
                j
            })))
        }
    }

    make(dim)
}
