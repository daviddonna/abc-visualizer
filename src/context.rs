extern crate abc;
extern crate rand;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use self::abc::{Context, Candidate};
use self::rand::{thread_rng, Rng};

use coords::Coords;
use spmc::Queue;

pub struct NewBee {
    pub id: usize,
    pub coords: Coords,
}

pub struct Ctx {
    id: Mutex<usize>,
    new_bees: Mutex<Sender<NewBee>>,
    queue: Arc<Queue<f64>>,
    min: Coords,
    max: Coords,
}

impl Ctx {
    pub fn new(queue: Arc<Queue<f64>>,
               new_bees: Sender<NewBee>,
               min: Coords,
               max: Coords)
               -> Ctx {
        Ctx {
            id: Mutex::new(0),
            new_bees: Mutex::new(new_bees),
            queue: queue,
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
                     })
                     .expect("failed to send new bee");
        drop(new_bee_guard);

        self.queue.wait_for(id)
    }

    fn explore(&self, field: &[Candidate<Coords>], n: usize) -> Coords {
        Coords {
            x: field[n].solution.x + thread_rng().gen_range(-20, 21),
            y: field[n].solution.y + thread_rng().gen_range(-20, 21),
        }
        .clamp(&self.min, &self.max)
    }
}
