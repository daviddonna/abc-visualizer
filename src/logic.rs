extern crate abc;

use std::sync::mpsc::Receiver;

use self::abc::Candidate;

use context::{NewBee, FitnessMover};
use coords::Coords;
use state::{State, Bee, Activity};

const BEE_GATHER_RATE: f64 = 0.6;
const BEE_SPEED: i32 = 5;

pub struct Logic<'a> {
    events: Receiver<NewBee>,
    best: Receiver<Candidate<Coords>>,
    mover: &'a FitnessMover,
}

impl<'a> Logic<'a> {
    pub fn new(events: Receiver<NewBee>,
               best: Receiver<Candidate<Coords>>,
               mover: &'a FitnessMover)
               -> Logic<'a> {
        Logic {
            events: events,
            best: best,
            mover: mover,
        }
    }

    fn move_towards(&self, bee: &mut Bee, target: &Coords) {
        let remaining = *target - bee.location;
        let distance = remaining.magnitude();
        if distance <= BEE_SPEED {
            bee.location = target.clone();
        } else {
            let travel = remaining / (remaining.magnitude() / BEE_SPEED);
            bee.location = bee.location + travel;
        }
    }

    pub fn tick(&self, state: &mut State) {
        if let Ok(NewBee { id, coords }) = self.events.try_recv() {
            state.add_bee(id, coords);
        }

        while let Ok(best) = self.best.try_recv() {
            info!("new best: {} ({},{})",
                  best.fitness,
                  best.solution.x,
                  best.solution.y);
            state.best = Some((best.solution, best.fitness));
        }

        // Split the borrow to use the fitness function while mutating bees.
        let bees = &mut state.bees;
        let get_fitness = &state.fitness;
        let hive_location = &state.hive_location;

        let delete = bees.iter_mut()
                         .enumerate()
                         .map(|(i, maybe)| {
                             maybe.as_mut().map_or(None, |bee: &mut Bee| {
                                 match bee.activity {
                                     Activity::Seeking => {
                                         let assignment = bee.assignment.clone();
                                         self.move_towards(bee, &assignment);
                                         if bee.location == bee.assignment {
                                             bee.location = bee.assignment;
                                             bee.activity = Activity::Gathering(0.0);
                                         }
                                         None
                                     }
                                     Activity::Gathering(n) => {
                                         let fitness = get_fitness(bee.location);
                                         let new_n = n + BEE_GATHER_RATE;
                                         if new_n >= fitness {
                                             bee.activity = Activity::Returning(fitness);
                                         } else {
                                             bee.activity = Activity::Gathering(new_n);
                                         }
                                         None
                                     }
                                     Activity::Returning(n) => {
                                         self.move_towards(bee, hive_location);
                                         if bee.location == *hive_location {
                                             self.mover.send(bee.id, n);
                                             Some(i)
                                         } else {
                                             None
                                         }
                                     }
                                 }
                             })
                         })
                         .filter(|x| x.is_some())
                         .collect::<Vec<_>>();

        for i in delete.iter() {
            bees[i.unwrap()] = None;
        }
    }
}
