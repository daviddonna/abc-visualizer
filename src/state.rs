use coords::Coords;

pub type FitnessFunction = Fn(Coords) -> f64;

#[derive(Debug)]
pub enum Activity {
    Seeking,
    Gathering(f64),
    Returning(f64),
}

#[derive(Debug)]
pub struct Bee {
    pub id: usize,
    pub location: Coords,
    pub assignment: Coords,
    pub activity: Activity,
}

pub struct Field {
    min: Coords,
    max: Coords,
}

impl Field {
    pub fn new(min: Coords, max: Coords) -> Field {
        Field {
            min: min,
            max: max,
        }
    }
}

pub struct State {
    pub fitness: Box<FitnessFunction>,
    pub hive_location: Coords,
    pub bees: Vec<Option<Bee>>,
    pub best: Option<(Coords, f64)>,
    pub field: Field,
}

impl State {
    pub fn new(min: Coords, max: Coords, threads: usize, fitness: Box<FitnessFunction>) -> State {
        let field = Field::new(min, max);
        let hive_coords = Coords::random(&field.min, &field.max);
        State {
            fitness: fitness,
            field: field,
            hive_location: hive_coords,
            bees: (0..threads).map(|_| None).collect(),
            best: None,
        }
    }

    pub fn corners(&self) -> (Coords, Coords) {
        (self.field.min.clone(), self.field.max.clone())
    }

    pub fn add_bee(&mut self, id: usize, coords: Coords) {
        let bee = Bee {
            id: id,
            location: self.hive_location.clone(),
            assignment: coords,
            activity: Activity::Seeking,
        };
        let index = self.bees
                        .iter()
                        .position(|b| b.is_none())
                        .expect("no available bee slot");
        self.bees[index] = Some(bee);
    }
}
