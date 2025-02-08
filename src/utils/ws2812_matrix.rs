use crate::rgb::Rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

const WIDTH: usize = 8;
const HEIGHT: usize = 8;

pub struct Ws2812Matrix {
    leds: [Rgb; WIDTH * HEIGHT],
    orientation: Orientation,
}

impl Ws2812Matrix {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            leds: [Rgb::new(0, 0, 0); WIDTH * HEIGHT],
            orientation,
        }
    }
    pub fn fill(&mut self, c: Rgb) {
        for i in 0..(WIDTH * HEIGHT) {
            self.leds[i] = c;
        }
    }
    pub fn set(&mut self, xy: (usize, usize), c: Rgb) {
        self.leds[xy.0 + xy.1 * HEIGHT] = c;
    }
    pub fn iter(&self) -> Ws2812MatrixIterator {
        Ws2812MatrixIterator {
            leds: &self.leds,
            index: 0,
            orientation: self.orientation,
        }
    }
}

pub struct Ws2812MatrixIterator<'a> {
    leds: &'a [Rgb; WIDTH * HEIGHT],
    orientation: Orientation,
    index: usize,
}

impl<'a> Iterator for Ws2812MatrixIterator<'a> {
    type Item = Rgb;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < WIDTH * HEIGHT {
            let out = match self.orientation {
                Orientation::North => self.leds[self.index],
                Orientation::East => {
                    // XXX Not Working
                    let (x, y) = (self.index % WIDTH, self.index / WIDTH);
                    self.leds[y * WIDTH + (HEIGHT - 1 - x)]
                }
                Orientation::South => {
                    let (x, y) = (self.index % WIDTH, self.index / WIDTH);
                    self.leds[(WIDTH - x - 1) + (HEIGHT - y - 1) * WIDTH]
                }
                Orientation::West => {
                    // XXX Not Working
                    let (x, y) = (self.index % WIDTH, self.index / WIDTH);
                    self.leds[x + (HEIGHT - 1 - y) * WIDTH]
                }
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }
}
