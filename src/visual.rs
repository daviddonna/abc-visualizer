extern crate sdl2;

use self::sdl2::Sdl;
use self::sdl2::rect::Rect;
use self::sdl2::render::{Renderer, Texture};
use self::sdl2::pixels::{PixelFormatEnum as Format, Color};

use state::State;
use coords::Coords;

pub struct Visual<'a> {
    renderer: Renderer<'a>,
    texture: Option<Texture>,
    min: Coords,
    max: Coords,
    width: u32,
    height: u32,
}

impl<'a> Visual<'a> {
    pub fn new(context: &'a Sdl, min: Coords, max: Coords) -> Visual<'a> {
        let width = (max.x - min.x) as u32;
        let height = (max.y - min.y) as u32;

        let video_subsystem = context.video().unwrap();
        let window = video_subsystem.window("Bees!", width, height)
                                    .position_centered()
                                    .build()
                                    .unwrap();
        let renderer = window.renderer().build().unwrap();

        Visual {
            renderer: renderer,
            texture: None,
            min: min,
            max: max,
            width: width,
            height: height,
        }
    }

    pub fn make_texture(&mut self, state: &State, min_color: [f64; 3], max_color: [f64; 3]) {
        let mut texture = self.renderer
                              .create_texture_streaming(Format::RGB24, self.width, self.height)
                              .unwrap();

        let mut min_fitness = (state.fitness)(self.min.clone());
        let mut max_fitness = (state.fitness)(self.max.clone());

        for x in self.min.x..(self.max.x + 1) {
            for y in self.min.y..(self.max.y + 1) {
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
                   for y in 0..self.height {
                       for x in 0..self.width {
                           let fitness = (state.fitness)(Coords {
                               x: self.min.x + x as i32,
                               y: self.min.y + y as i32,
                           });
                           let offset = (y * (pitch as u32) + x * 3) as usize;

                           for i in 0..3 {
                               let ratio = (fitness - min_fitness) / (max_fitness - min_fitness);
                               let c = (min_color[i] +
                                        ((max_color[i] - min_color[i]) *
                                         ratio)) as u8;
                               buffer[offset + i] = c;
                           }
                       }
                   }
               })
               .unwrap();

        self.texture = Some(texture);
    }

    pub fn draw_bee(&mut self, location: Coords) {
        self.renderer.fill_rect(Rect::new(location.x - 4, location.y - 4, 8, 8)).unwrap();
    }

    pub fn draw_hive(&mut self, location: Coords) {
        self.renderer.set_draw_color(Color::RGB(200, 155, 50));
        self.renderer.fill_rect(Rect::new(location.x - 20, location.y - 30, 40, 50)).unwrap();
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
        self.renderer.copy(self.texture.as_ref().unwrap(),
                           None,
                           Some(Rect::new(0, 0, self.width, self.height)));
        self.renderer.set_draw_color(Color::RGB(255, 255, 50));

        let (upper_left, _) = state.corners();

        for maybe in state.bees.iter() {
            if let Some(bee) = maybe.as_ref() {
                self.draw_bee(bee.location - upper_left);
            }
        }

        self.draw_hive(state.hive_location - upper_left);

        if let Some(&(ref coords, _)) = state.best.as_ref() {
            self.draw_best(coords.clone() - upper_left);
        }

        self.renderer.present();
    }
}
