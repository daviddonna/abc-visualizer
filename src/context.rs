extern crate abc;
extern crate rand;

use std::sync::{Arc, Mutex, Condvar, MutexGuard};
use std::sync::mpsc::Sender;

use self::abc::{Context, Candidate};
use self::rand::{thread_rng, Rng};

use coords::Coords;

pub struct NewBee {
    pub id: usize,
    pub coords: Coords,
}

pub struct Fitness {
    pub id: usize,
    pub fitness: f64,
}

pub struct FitnessMover {
    fitness: Mutex<Option<Fitness>>,
    produced: Condvar,
    consumed: Condvar,
}

impl FitnessMover {
    pub fn new() -> FitnessMover {
        FitnessMover {
            fitness: Mutex::new(None),
            produced: Condvar::new(),
            consumed: Condvar::new(),
        }
    }

    pub fn send(&self, id: usize, fitness: f64) {
        let mut guard = self.fitness.lock().unwrap();
        while guard.is_some() {
            guard = self.consumed.wait(guard).unwrap();
        }
        *guard = Some(Fitness {
            id: id,
            fitness: fitness,
        });
        self.produced.notify_all()
    }

    fn _wait_for(&self, mut guard: MutexGuard<Option<Fitness>>, id: usize) -> f64 {
        match *guard {
            Some(Fitness {id: id2, fitness}) if id2 == id => {
                *guard = None;
                self.consumed.notify_one();
                fitness
            }
            _ => self._wait_for(self.produced.wait(guard).unwrap(), id),
        }
    }

    pub fn wait_for(&self, id: usize) -> f64 {
        let guard = self.fitness.lock().unwrap();
        self._wait_for(guard, id)
    }
}

pub struct Ctx {
    id: Mutex<usize>,
    new_bees: Mutex<Sender<NewBee>>,
    fitness_mover: Arc<FitnessMover>,
    min: Coords,
    max: Coords,
}

impl Ctx {
    pub fn new(mover: Arc<FitnessMover>,
               new_bees: Sender<NewBee>,
               min: Coords,
               max: Coords)
               -> Ctx {
        Ctx {
            id: Mutex::new(0),
            new_bees: Mutex::new(new_bees),
            fitness_mover: mover,
            min: min,
            max: max,
        }
    }
}

impl Context for Ctx {
    type Solution = Coords;

    fn make(&self) -> Coords {
        Coords::random(&self.min, &self.max)
    }

    fn evaluate_fitness(&self, solution: &Coords) -> f64 {
        let id = {
            let mut id_guard = self.id.lock().unwrap();
            *id_guard += 1;
            *id_guard
        };

        let new_bee_guard = self.new_bees.lock().unwrap();
        new_bee_guard.send(NewBee {
            id: id,
            coords: solution.clone(),
        }).expect("failed to send new bee");
        drop(new_bee_guard);

        let f = self.fitness_mover.wait_for(id);
        f
    }

    fn explore(&self, field: &[Candidate<Coords>], n: usize) -> Coords {
        Coords {
            x: field[n].solution.x + thread_rng().gen_range(-20, 21),
            y: field[n].solution.y + thread_rng().gen_range(-20, 21),
        }
        .clamp(&self.min, &self.max)
    }
}
