extern crate abc;
extern crate sdl2;

#[macro_use]
extern crate log;
extern crate env_logger;

mod coords;
mod spmc;
mod context;
mod state;
mod logic;
mod visual;

use std::f32::consts::PI;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread::{spawn, sleep};
use std::time::Duration;

use sdl2::event::Event;
use abc::{HiveBuilder, Candidate};

use coords::Coords;
use context::{FitnessMover, Ctx, NewBee};
use state::State;
use logic::Logic;
use visual::Visual;

const FPS: u32 = 60;

fn main() {
    env_logger::init().unwrap();

    info!("starting");

    let sdl = sdl2::init().unwrap();
    let frame_time = 1000 / FPS;

    let threads = 40_usize;
    let workers = 20_usize;
    let min = Coords { x: -256, y: -256 };
    let max = Coords { x: 256, y: 256 };
    let fitness = Box::new(|Coords { x, y }| {
        let a = 10.0;
        let x = x as f32 / 50.0;
        let y = y as f32 / 50.0;
        ((2.0 * a) + x.powi(2) + y.powi(2) -
         a *
         ((2.0 * PI * x).cos() +
          (2.0 * PI * y).cos())) as f64
    });

    let (send_new_bees, receive_new_bees) = channel::<NewBee>();
    let (send_best, receive_best) = channel::<Candidate<Coords>>();

    let mut state = State::new(min, max, threads, fitness);
    let mover = Arc::new(FitnessMover::new());

    let logic = Logic::new(receive_new_bees, receive_best, &mover);
    let mut visual = Visual::new(&sdl, &state);

    let context = Ctx::new(mover.clone(), send_new_bees, min, max);
    spawn(move || {
        let mut hive = HiveBuilder::new(context, workers).set_threads(threads).build().unwrap();
        hive.set_sender(send_best);
        hive.run_forever().unwrap()
    });

    let mut timer = sdl.timer().unwrap();
    let mut events = sdl.event_pump().unwrap();
    let mut start_ticks;

    'running: loop {
        start_ticks = timer.ticks();
        if let Some(Event::Quit { .. }) = events.poll_event() {
            break 'running;
        }

        logic.tick(&mut state);
        visual.render(&state);
        let ticks_remaining = (start_ticks + frame_time) as i32 - timer.ticks() as i32;
        if ticks_remaining > 0 {
            sleep(Duration::from_millis(ticks_remaining as u64));
        }
    }
}
