use image;
use rayon::prelude::*;
use std::{
    f64::consts::TAU,
    ops::{Add, AddAssign, Mul, Sub},
};

type Precision = f64;
fn main() {
    let width: usize = 400;
    let height: usize = 400;
    let plane_x: Precision = 0.;
    let plane_y: Precision = 0.;
    let plane_w: Precision = 400.;
    let plane_h: Precision = 400.;

    let magnet_radius: Precision = 50.;

    let mut img_buf = image::ImageBuffer::new(width as u32, height as u32);
    let grid_size: usize = width;

    let magnets_count: usize = 3;
    let mut magnets: Vec<Magnet> = Vec::with_capacity(magnets_count);

    for i in 0..magnets_count {
        let x = width as Precision / 2.0
            + magnet_radius * ((i as Precision * TAU) / (magnets_count as Precision)).cos();
        let y = height as Precision / 2.0
            + magnet_radius * ((i as Precision * TAU) / (magnets_count as Precision)).sin();
        magnets.push(Magnet::new(Vec2::new(x, y)))
    }

    magnets[0].color = Color::Yellow;
    magnets[1].color = Color::Red;
    magnets[2].color = Color::Blue;
    // magnets[3].color = Color::Green;

    // let pixels = vec![vec![Color::Black; width]; height];
    let initial_ball_count: usize = (width * height).div_euclid(grid_size.pow(2));
    let mut balls: Vec<Ball> = Vec::with_capacity(initial_ball_count);
    // let mut unfinished_balls: Vec<&Ball> = balls.iter().filter(|b| !b.finished).collect();

    let tile_height = height.div_euclid(grid_size);
    let tile_width = width.div_euclid(grid_size);


    for window_y in 0..grid_size {
        let y = plane_y + (window_y as Precision) / (grid_size as Precision) * plane_h;
        for window_x in 0..grid_size {
            let x = plane_x + (window_x as Precision) / (grid_size as Precision) * plane_w;

            balls.push(Ball {
                pos: Vec2::new(x, y),
                vel: Vec2::new(0., 0.),
                init_pos: Vec2::new(x, y),
                finished: false,
                final_color: None,
            })
        }
    }


    println!("Hello, world!");


    balls.par_iter_mut().for_each(|ball| {
        let mut iters = 0u32;
        while !ball.finished {
            iters += 1;
            if iters == 500_000 {
                // println!("Ball stagnant");
                break;
            }
            // if ball.finished {
            //     // unfinished_balls.
            //     println!("Finished ball");
            //     continue;
            // }

            let mut force = Vec2::zero();
            let mut min_dist_sq: Precision = Precision::INFINITY;
            let mut min_color: Option<Color> = None;
            for m in magnets.iter() {
                let sub_force = m.pos - ball.pos;
                let mag_sq = sub_force.mag_sq();

                if mag_sq < min_dist_sq {
                    min_dist_sq = mag_sq;
                    min_color = Some(m.color);
                }

                force += sub_force.normalize().mul(1.0 * m.f / mag_sq).limit(5.);
            }

            if min_dist_sq < 100.0 {
                if ball.vel.mag_sq() < 16.0 {
                    ball.finished = true;
                    // println!("Ball finished");
                    ball.final_color = min_color;
                    continue;
                }
            }
            ball.vel += force;
            ball.vel = ball.vel.limit(1000.0).mul(0.99999999999);
            ball.pos += ball.vel
        }
    });

    println!("Finished");

    for colored_ball in balls.iter().filter(|&b| b.finished) {
        // let y = plane_y + (window_y as Precision) / (grid_size as Precision) * plane_h;
        // for window_x in 0..grid_size {
        // let x = plane_x + (window_x as Precision) / (grid_size as Precision) * plane_w;
        let window_x = (colored_ball.init_pos.x - plane_x) / plane_w * (width as Precision);
        let window_y = (colored_ball.init_pos.y - plane_y) / plane_h * (height as Precision);
        let pixel = img_buf.get_pixel_mut(window_x.round() as u32, window_y.round() as u32);

        let image::Rgb(data) = *pixel;
        *pixel = image::Rgb(colored_ball.final_color.unwrap().to_rgb_u8());
    }

    // println!("{:?}"/, finished_balls);
    img_buf.save("fractal.png").unwrap()
}

#[derive(Clone, Copy, Debug)]
enum Color {
    Yellow,
    Red,
    Green,
    Blue,
    Black,
}

impl Color {
    fn to_string(&self) -> String {
        match self {
            Color::Red => "red".to_string(),
            Color::Green => "green".to_string(),
            Color::Blue => "blue".to_string(),
            Color::Black => "transparent".to_string(),
            Color::Yellow => "yellow".to_string(),
        }
    }

    fn to_rgb_u8(&self) -> [u8; 3] {
        match self {
            Color::Yellow => [255, 255, 0],
            Color::Red => [255, 0, 0],
            Color::Green => [0, 255, 0],
            Color::Blue => [0, 0, 255],
            Color::Black => [0, 0, 0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Vec2 {
    x: Precision,
    y: Precision,
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<Precision> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Precision) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Vec2 {
    fn new(x: Precision, y: Precision) -> Self {
        Self { x, y }
    }

    fn zero() -> Self {
        Self {
            x: 0.0000001,
            y: 0.0000001,
        }
    }

    fn mag_sq(&self) -> Precision {
        self.x.powi(2) + self.y.powi(2)
    }

    fn mag(&self) -> Precision {
        self.mag_sq().sqrt()
    }

    fn normalize(&self) -> Self {
        let mag = self.mag();
        Self {
            x: self.x / mag,
            y: self.y / mag,
        }
    }

    fn limit(&self, limit: Precision) -> Self {
        let ratio = limit / self.mag();
        if ratio < 1.0 {
            return Self {
                x: self.x * ratio,
                y: self.y * ratio,
            };
        }
        self.clone()
    }
}

#[test]
fn vec2_limit() {
    let mut ten = Vec2::new(10., 0.);
    ten = ten.limit(5.);
    let five = Vec2::new(5., 0.);
    assert_eq!(ten, five)
}

#[test]
fn vec2_normalize() {
    let mut ten = Vec2::new(10., 0.);
    ten = ten.normalize();
    let one = Vec2::new(1., 0.);
    assert_eq!(ten, one)
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

struct Magnet {
    pos: Vec2,
    f: Precision,
    color: Color,
}

impl Magnet {
    fn new(pos: Vec2) -> Self {
        Self {
            pos,
            f: 100.0,
            color: Color::Black,
        }
    }
}

#[derive(Debug)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
    init_pos: Vec2,
    finished: bool,
    final_color: Option<Color>,
}
