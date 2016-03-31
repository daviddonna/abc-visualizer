extern crate sdl2;

use self::sdl2::Sdl;
use self::sdl2::rect::Rect;
use self::sdl2::render::{Renderer, Texture};
use self::sdl2::pixels::{PixelFormatEnum as Format, Color};

use state::State;
use coords::Coords;

const MIN_COLOR: [f64; 3] = [5.0_f64, 20.0, 9.0];
const MAX_COLOR: [f64; 3] = [50.0_f64, 200.0, 90.0];

pub struct Visual<'a> {
    renderer: Renderer<'a>,
    texture: Texture,
}

impl<'a> Visual<'a> {
    pub fn new(context: &'a Sdl, state: &State) -> Visual<'a> {
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem.window("Bees!", state.width(), state.height())
                                    .position_centered()
                                    .build()
                                    .unwrap();
        let renderer = window.renderer().build().unwrap();
        let texture = Visual::make_texture(&renderer, state);
        Visual {
            renderer: renderer,
            texture: texture,
        }
    }

    fn make_texture(renderer: &Renderer, state: &State) -> Texture {
        let width = state.width();
        let height = state.height();

        let mut texture = renderer.create_texture_streaming(Format::RGB24, width, height)
                                  .unwrap();

        let (min, max) = state.corners();
        let mut min_fitness = (state.fitness)(min);
        let mut max_fitness = (state.fitness)(min);

        for x in min.x..(max.x + 1) {
            for y in min.y..(max.y + 1) {
                let fitness = (state.fitness)(Coords { x: x, y: y });

                if fitness < min_fitness {
                    min_fitness = fitness;
                }
                if fitness > max_fitness {
                    max_fitness = fitness;
                }
            }
        }

        // Create a red-green gradient
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                   for y in 0..height {
                       for x in 0..width {
                           let fitness = (state.fitness)(Coords {
                               x: min.x + x as i32,
                               y: min.y + y as i32,
                           });
                           let offset = (y * (pitch as u32) + x * 3) as usize;

                           for i in 0..3 {
                               let ratio = (fitness - min_fitness) / (max_fitness - min_fitness);
                               let c = (MIN_COLOR[i] +
                                        ((MAX_COLOR[i] - MIN_COLOR[i]) *
                                         ratio)) as u8;
                               buffer[offset + i] = c;
                           }
                       }
                   }
               })
               .unwrap();
        texture
    }

    pub fn draw_bee(&mut self, location: Coords) {
        self.renderer.fill_rect(Rect::new(location.x - 4, location.y - 4, 8, 8)).unwrap();
    }

    pub fn draw_hive(&mut self, location: Coords) {
        self.renderer.set_draw_color(Color::RGB(200, 155, 50));
        self.renderer.fill_rect(Rect::new(location.x - 20, location.y - 30, 40, 50)).unwrap();
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.fill_rect(Rect::new(location.x - 5, location.y - 5, 10, 10)).unwrap();
    }

    pub fn draw_best(&mut self, location: Coords) {
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.fill_rect(Rect::new(location.x - 12, location.y - 2, 10, 4)).unwrap();
        self.renderer.fill_rect(Rect::new(location.x + 2, location.y - 2, 10, 4)).unwrap();
        self.renderer.fill_rect(Rect::new(location.x - 2, location.y - 12, 4, 10)).unwrap();
        self.renderer.fill_rect(Rect::new(location.x - 2, location.y + 2, 4, 10)).unwrap();
    }

    pub fn render(&mut self, state: &State) {
        self.renderer.clear();
        self.renderer.copy(&self.texture,
                           None,
                           Some(Rect::new(0, 0, state.width(), state.height())));

        let (upper_left, _) = state.corners();

        self.draw_hive(state.hive_location - upper_left);

        self.renderer.set_draw_color(Color::RGB(255, 255, 50));
        for maybe in state.bees.iter() {
            if let Some(bee) = maybe.as_ref() {
                self.draw_bee(bee.location - upper_left);
            }
        }

        if let Some(&(ref coords, _)) = state.best.as_ref() {
            self.draw_best(coords.clone() - upper_left);
        }

        self.renderer.present();
    }
}
