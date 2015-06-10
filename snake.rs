/* Copyright (C) 2015 by Alexandru Cojocaru */

/* This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>. */


/* I'm still learning both Rust and Piston, please don't judge me :) */

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use graphics::*;
use opengl_graphics::{ GlGraphics, OpenGL };
//use glutin_window::GlutinWindow as Window;
//use piston::window::WindowSettings;
use piston::event::*;
use piston::input::keyboard::Key;
use rand::{thread_rng, Rng};

// width and height must never change
const BOARD_WIDTH: i8 = 15;
const BOARD_HEIGHT: i8 = 15;
const TILE_SIZE: f64 = 50.0;
const UPDATE_TIME: f64 = 0.15;

#[derive(PartialEq, Copy, Clone)]
enum State {
    Playing,
    Paused,
    GameOver,
}

struct Snake {
    tail: Vec<(i8,i8)>,
    headi: usize,
    last_pressed: Key,
}

impl Snake {
    fn new(tail: Vec<(i8, i8)>, key: Key) -> Snake {
        Snake {
            tail: tail,
            headi: 0,
            last_pressed: key,
        }
    }
    
    fn render(&self, t: math::Matrix2d, gfx: &mut GlGraphics) {
        for p in self.tail.iter() {
            rectangle(color::hex("8ba673"),
                      rectangle::square(p.0 as f64 * TILE_SIZE, p.1 as f64 * TILE_SIZE, TILE_SIZE),
                      t, gfx
            );
        }
    }

    fn key_press(&mut self, k: Key) {
        use piston::input::keyboard::Key::*;
        // Don't allow a player to kill themselves by doing a 180
        match k {
            Right if self.last_pressed != Left => {},
            Left if self.last_pressed != Right => {},
            Up if self.last_pressed != Down => {},
            Down if self.last_pressed != Up => {}
            _ => return
        }

        self.last_pressed = k;
    }

    fn mv(g: &mut Game, dtxy: (i8, i8)) {
        let mut xy = (g.snake.tail[g.snake.headi].0 + dtxy.0, g.snake.tail[g.snake.headi].1 + dtxy.1);
        if xy.0 >= BOARD_WIDTH {
            xy.0 = 0;
        }
        if xy.0 < 0 {
            xy.0 = BOARD_WIDTH-1;
        }
        if xy.1 >= BOARD_HEIGHT {
            xy.1 = 0;
        }
        if xy.1 < 0 {
            xy.1 = BOARD_HEIGHT-1;
        }

        if g.walls.collides(xy) || g.snake.collides(xy) {
            g.state = State::GameOver;
            println!("### Game Over ###\nScore: {}\nPress R to restart\nPress Esc to quit", g.score);
            return;
        }
        for i in 0..g.food.len() {
            if g.food[i].xy == xy {
                let f = g.food.swap_remove(i);
                g.score += f.score;
                let xy = g.snake.tail[g.snake.headi];
                g.snake.tail.push(xy);
                g.update_time -= 0.002;
                break;
            }
        }
        g.snake.tail.pop();
        g.snake.tail.insert(0, xy);
    }

    fn update(g: &mut Game) {
        use piston::input::keyboard::Key::*;
        Snake::mv(g, match g.snake.last_pressed {
            Right =>  (1, 0),
            Down => (0, 1),
            Left => (-1, 0),
            Up => (0, -1),
            _ => panic!("only UP/DOWN/LEFT/UP arrows allowed"),
        })
    }

    fn collides(&self, xy: (i8,i8)) -> bool {
        self.tail.iter().any(|t| *t == xy)
    }
}

#[derive(PartialEq)]
enum FoodType {
    Apple,
    Candy,
}

struct Food {
    food_type: FoodType,
    xy: (i8,i8),
    score: u32,
    life_time: u32, 
    lived_time: u32,
}

impl Food {
    fn new(t: FoodType, xy: (i8,i8), s: u32, lt: u32, probability: f64) -> Option<Food> {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0, 100.0) < probability {
            Some(Food {
                    food_type: t,
                    xy: xy,
                    score: s,
                    life_time: lt,
                    lived_time: 0
            })
        } else {
            None
        }
    }

    fn genxy(g: &Game) -> (i8,i8) {
        loop {
            let mut rng = rand::thread_rng();
            let xy = (rng.gen_range(0,BOARD_WIDTH),
                      rng.gen_range(0,BOARD_HEIGHT));

            if !(g.snake.tail.iter().any(|t| *t == xy) ||
                 g.food.iter().any(|f| f.xy == xy) ||
                 g.walls.iter().any(|w| *w == xy) ||
                 g.invisible_walls.iter().any(|w| *w == xy)) {
                return xy;
            }
        }
    }

    fn update(g: &mut Game) {
        if !g.food.iter().any(|f| f.food_type == FoodType::Apple) {
            if let Some(f) = Food::new(FoodType::Apple, Food::genxy(g), 10, 45, 100.0) {
                g.food.push(f)
            }            
        } 

        if !g.food.iter().any(|f| f.food_type == FoodType::Candy) {
            if let Some(f) = Food::new(FoodType::Candy, Food::genxy(g), 50, 15, 1.0) {
                g.food.push(f)
            }
        }
        
        for i in 0..g.food.len() {
            g.food[i].lived_time += 1;
            if g.food[i].lived_time > g.food[i].life_time {
                g.food.swap_remove(i);
                break;
            }
        }
    }

    fn render(&self, t: math::Matrix2d, gfx: &mut GlGraphics) {
        if self.life_time - self.lived_time < 6 && self.lived_time % 2 == 0 {
            return
        }

        let color = match self.food_type {
            FoodType::Apple => color::hex("b83e3e"),
            FoodType::Candy => color::hex("b19d46"),
        };

        rectangle(color, rectangle::square(self.xy.0 as f64 * TILE_SIZE, self.xy.1 as f64 * TILE_SIZE, TILE_SIZE), t, gfx);
    }
}

trait Collides {
    fn collides(&self, xy: (i8,i8)) -> bool;
}

impl Collides for Vec<Food> {
    fn collides(&self, xy: (i8,i8)) -> bool {
        self.iter().any(|f| f.xy == xy)
    }
}

impl Collides for Vec<(i8,i8)> {
    fn collides(&self, xy: (i8,i8)) -> bool {
        self.iter().any(|z| *z == xy)
    }
}

struct Level {
    snake: Snake,
    walls: Vec<(i8,i8)>,
    invisible_walls: Vec<(i8,i8)>,
}

fn level1() -> Level {
    let w = vec![
        (1,0),(2,0),(3,0),(4,0),(5,0),(6,0),(8,0),(9,0),(10,0),(11,0),(12,0),(13,0),
        (14,1),(14,2),(14,3),(14,4),(14,5),(14,6),(14,8),(14,9),(14,10),(14,11),(14,12),(14,13),
        (1,14),(2,14),(3,14),(4,14),(5,14),(6,14),(8,14),(9,14),(10,14),(11,14),(12,14),(13,14),
        (0,1),(0,2),(0,3),(0,4),(0,5),(0,6),(0,8),(0,9),(0,10),(0,11),(0,12),(0,13),
        (7,7),
    ];

    let iw = vec![(0,0),(7,0),(14,0),(14,7),(14,14),(7,14),(0,14),(0,7)];

    Level {
        snake: Snake::new(vec![(2,3), (2,2), (2,1)], Key::Down),
        walls: w,
        invisible_walls: iw,
    }
}

fn level2() -> Level {
    let w = vec![
        (2,2),(3,3),(4,4),(5,5),(7,7),(9,9),(10,10),(11,11),(12,12),
        (12,2),(11,3),(10,4),(9,5),(7,7),(5,9),(4,10),(3,11),(2,12),
        (0,7),(7,0),(14,7),(7,14),
    ];
    
    let iw = vec![];

    Level {
        snake: Snake::new(vec![(0,0), (1,0), (2,0)], Key::Down),
        walls: w,
        invisible_walls: iw,
    }
}

fn rand_level() -> Level {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0,2) {
        0 => level1(),
        1 => level2(),
        _ => panic!(""),
    }
}

    
struct Game {
    gfx: GlGraphics,
    snake: Snake,
    time: f64,
    update_time: f64,
    state: State,
    walls: Vec<(i8,i8)>,
    invisible_walls: Vec<(i8,i8)>,
    food: Vec<Food>,
    score: u32,
    last_key: Key,
}

impl Game {
    fn new() -> Game {
        
        let opengl = OpenGL::_3_2;
        let gl = GlGraphics::new(opengl);
        let l = rand_level();
        Game {gfx: gl,
              snake: l.snake,
              time: UPDATE_TIME,
              update_time: UPDATE_TIME,
              state: State::Playing,
              walls: l.walls,
              invisible_walls: l.invisible_walls,
              food: vec![],
              score: 0,
              last_key: Key::Unknown,
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        let t = Context::new_viewport(args.viewport()).transform;
        if self.state == State::GameOver {
            clear(color::hex("000000"), &mut self.gfx);
            return;
        }

        clear(color::hex("001122"), &mut self.gfx);

        for ref mut f in &self.food {
            f.render(t, &mut self.gfx);
        }

        self.snake.render(t, &mut self.gfx);

        for w in &self.walls {
            rectangle(color::hex("002951"),
                      rectangle::square(w.0 as f64 * TILE_SIZE, w.1 as f64 * TILE_SIZE, TILE_SIZE),
                      Context::new_viewport(args.viewport()).transform, &mut self.gfx);

        }

    }

    fn update(&mut self, args: &UpdateArgs) {
        match self.state {
            State::Paused | State::GameOver => return,
            _ => {},
        }

        self.time += args.dt;

        if self.time > self.update_time {
            self.time -= self.update_time;
            self.snake.key_press(self.last_key);
            Snake::update(self);
            Food::update(self);
        }
    }

    fn key_press(&mut self, key: Key) {
        match (key, self.state) {
            (Key::R, _) => {
                let l = rand_level();
                self.snake = l.snake;
                self.state = State::Playing;
                self.time = UPDATE_TIME;
                self.update_time = UPDATE_TIME;
                self.walls = l.walls;
                self.invisible_walls = l.invisible_walls;
                self.food = vec![];
                self.score = 0;
                self.last_key = Key::Unknown;
                return;
            },
            (Key::P, State::Playing) => {
                self.state = State::Paused;
            },
            (Key::P, State::Paused) => {
                self.state = State::Playing;
            },
            _ => {
                self.last_key = key;

            }
        };
    }
}

fn main() {
    use glutin_window::GlutinWindow as Window;
    use piston::window::WindowSettings;
    println!("R => Restart\nP => Pause\nEsc => Quit");
    
    let window = Window::new(
        WindowSettings::new("Snake - Piston",
                            [BOARD_WIDTH as u32 * TILE_SIZE as u32, BOARD_HEIGHT as u32 * TILE_SIZE as u32])
            .exit_on_esc(true));
    
    let mut game = Game::new();
    
    for e in window.events() {
        use piston::input::Button;
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            game.key_press(key);
        }

        if let Some(args) = e.update_args() {
            game.update(&args);
        }
    }
}
