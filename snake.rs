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
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use graphics::*;
use opengl_graphics::{ GlGraphics, OpenGL };
use glutin_window::GlutinWindow as Window;
use piston::window::WindowSettings;
use piston::event::*;
use piston::input::keyboard::Key;
use rand::{thread_rng, Rng};

// width and height must never change
const BOARD_WIDTH: u32 = 15;
const BOARD_HEIGHT: u32 = 15;
const TILE_SIZE: f64 = 50.0;
const UPDATE_TIME: f64 = 0.15;

#[derive(PartialEq, Copy, Clone)]
enum State {
    Playing,
    Paused,
    GameOver,
}

struct Snake {
    tail: Vec<(f64,f64)>,
    headi: usize,
    last_pressed: Key,
}

impl Snake {
    fn new(tail: Vec<(f64, f64)>, key: Key) -> Snake {
        Snake{tail: tail,
              headi: 0,
              last_pressed: key,}
    }
    
    fn index(&self, i: isize) -> usize {
        ((self.tail.len() as isize + i) as usize + self.headi) % self.tail.len()
    }

    fn render(&self, t: math::Matrix2d, gfx: &mut GlGraphics) {
        for p in self.tail.iter() {
            rectangle(color::hex("8ba673"),
                      rectangle::square(p.0*TILE_SIZE, p.1*TILE_SIZE, TILE_SIZE),
                      t, gfx);
        }
    }

    fn key_press(&mut self, k: Key) {
        use piston::input::keyboard::Key::*;
        if k == Right || k == Down || k == Left || k == Up {
            self.last_pressed = k;
        }
    }

    fn mv(g: &mut Game, dtxy: &(f64,f64)) {
        let mut xy = (g.snake.tail[g.snake.headi].0 + dtxy.0, g.snake.tail[g.snake.headi].1 + dtxy.1);
        if xy.0 >= BOARD_WIDTH as f64 {
            xy.0 = 0.0;
        }
        if xy.0 < 0.0 {
            xy.0 = (BOARD_WIDTH-1) as f64;
        }
        if xy.1 >= BOARD_HEIGHT as f64 {
            xy.1 = 0.0;
        }
        if xy.1 < 0.0 {
            xy.1 = (BOARD_HEIGHT-1) as f64;
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
        let headi = g.snake.index(-1);
        g.snake.headi = headi;
        g.snake.tail[headi] = xy;
    }

    fn update(g: &mut Game) {
        use piston::input::keyboard::Key::*;

        lazy_static! {
            static ref MOVING_REMAP: HashMap<Key, (f64, f64)> = {
                let mut m = HashMap::new();
                m.insert(Right, (1.0, 0.0));
                m.insert(Down, (0.0, 1.0));
                m.insert(Left, (-1.0, 0.0));
                m.insert(Up, (0.0, -1.0));
                m
            };
        }

        match MOVING_REMAP.get(&g.snake.last_pressed) {
            Some(x) => Snake::mv(g, x),
            None => panic!("only UP/DOWN/LEFT/UP arrows allowed")
        }
    }

    fn collides(&self, xy: (f64,f64)) -> bool {
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
    xy: (f64,f64),
    score: u32,
    life_time: u32, 
    lived_time: u32,
}

impl Food {
    fn new(t: FoodType, xy: (f64,f64), s: u32, lt: u32, probability: f64) -> Option<Food> {
        let mut rng = thread_rng();
        if rng.gen_range(0.0, 100.0) < probability {
            Some(Food{food_type: t,
                       xy: xy,
                       score: s,
                       life_time: lt,
                       lived_time: 0})
        } else {
            None
        }
    }

    fn genxy(g: &Game) -> (f64,f64) {
        loop {
            let mut rng = thread_rng();
            let xy_int = (rng.gen_range(0,BOARD_WIDTH),
                          rng.gen_range(0,BOARD_HEIGHT));
            let xy = (xy_int.0 as f64, xy_int.1 as f64);
            if !(g.snake.tail.iter().any(|t| *t == xy) || g.food.iter().any(|f| f.xy == xy) || g.walls.iter().any(|w| *w == xy) || g.invisible_walls.iter().any(|w| *w == xy)) {
                return xy;
            }
        }
    }

    fn update(g: &mut Game) {
        if !g.food.iter().any(|f| if f.food_type == FoodType::Apple { true } else { false } ) {
            if let Some(f) = Food::new(FoodType::Apple, Food::genxy(g), 10, 45, 100.0) {
                g.food.push(f)
            }            
        } 

        if !g.food.iter().any(|f| if f.food_type == FoodType::Candy { true } else { false } ) {
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
        match self.food_type {
            FoodType::Apple => {
                rectangle(color::hex("b83e3e"),
                          rectangle::square(self.xy.0*TILE_SIZE, self.xy.1*TILE_SIZE, TILE_SIZE),
                          t, gfx);
            },
            FoodType::Candy => {
                rectangle(color::hex("b19d46"),
                          rectangle::square(self.xy.0*TILE_SIZE, self.xy.1*TILE_SIZE, TILE_SIZE),
                          t, gfx);},
        }
    }
}

trait Collides {
    fn collides(&self, xy: (f64,f64)) -> bool;
}

impl Collides for Vec<Food> {
    fn collides(&self, xy: (f64,f64)) -> bool {
        self.iter().any(|f| f.xy == xy)
    }
}

impl Collides for Vec<(f64,f64)> {
    fn collides(&self, xy: (f64,f64)) -> bool {
        self.iter().any(|z| *z == xy)
    }
}

struct Level {
    snake: Snake,
    walls: Vec<(f64, f64)>,
    invisible_walls: Vec<(f64, f64)>,
}

fn level1() -> Level {
    let w = vec![
        (1.0,0.0),(2.0,0.0),(3.0,0.0),(4.0,0.0),(5.0,0.0),(6.0,0.0),(8.0,0.0),(9.0,0.0),(10.0,0.0),(11.0,0.0),(12.0,0.0),(13.0,0.0),
        (14.0,1.0),(14.0,2.0),(14.0,3.0),(14.0,4.0),(14.0,5.0),(14.0,6.0),(14.0,8.0),(14.0,9.0),(14.0,10.0),(14.0,11.0),(14.0,12.0),(14.0,13.0),
        (1.0,14.0),(2.0,14.0),(3.0,14.0),(4.0,14.0),(5.0,14.0),(6.0,14.0),(8.0,14.0),(9.0,14.0),(10.0,14.0),(11.0,14.0),(12.0,14.0),(13.0,14.0),
        (0.0,1.0),(0.0,2.0),(0.0,3.0),(0.0,4.0),(0.0,5.0),(0.0,6.0),(0.0,8.0),(0.0,9.0),(0.0,10.0),(0.0,11.0),(0.0,12.0),(0.0,13.0),
        (7.0,7.0),
        ];
    let iw = vec![(0.0,0.0),(7.0,0.0),(14.0,0.0),(14.0,7.0),(14.0,14.0),(7.0,14.0),(0.0,14.0),(0.0,7.0)];

    Level {
        snake: Snake::new(vec![(2.0,3.0), (2.0,2.0), (2.0,1.0)], Key::Down),
        walls: w,
        invisible_walls: iw,
    }
}

fn level2() -> Level {
    let w = vec![
        (2.0,2.0),(3.0,3.0),(4.0,4.0),(5.0,5.0),(7.0,7.0),(9.0,9.0),(10.0,10.0),(11.0,11.0),(12.0,12.0),
        (12.0,2.0),(11.0,3.0),(10.0,4.0),(9.0,5.0),(7.0,7.0),(5.0,9.0),(4.0,10.0),(3.0,11.0),(2.0,12.0),
        (0.0,7.0),(7.0,0.0),(14.0,7.0),(7.0,14.0),
        ];
    let iw = vec![];

    Level {
        snake: Snake::new(vec![(0.0,0.0), (1.0,0.0), (2.0,0.0)], Key::Down),
        walls: w,
        invisible_walls: iw,
    }
}

fn rand_level() -> Level {
    let mut rng = thread_rng();
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
    walls: Vec<(f64,f64)>,
    invisible_walls: Vec<(f64,f64)>,
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
                      rectangle::square(w.0*TILE_SIZE, w.1*TILE_SIZE, TILE_SIZE),
                      Context::new_viewport(args.viewport()).transform, &mut self.gfx);

        }

    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.state == State::Paused {return}
        if self.state == State::GameOver {
            return;
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
                return;},
            (Key::P, State::Playing) => {
                self.state = State::Paused;},
            (Key::P, State::Paused) => {
                self.state = State::Playing;},
            _ => {
                self.last_key = key;

            }
        };
    }
}

fn main() {
    lazy_static! {
        static ref PREV_MOVE_REMAP: HashMap<Key, Key> = {
            use piston::input::keyboard::Key::*;
            let mut m = HashMap::new();
            m.insert(Right, Left);
            m.insert(Down, Up);
            m.insert(Left, Right);
            m.insert(Up, Down);
            m
        };
    }

    println!("R => Restart\nP => Pause\nEsc => Quit");
    let window = Window::new(
        WindowSettings::new("Snake - Piston",
                            [BOARD_WIDTH * TILE_SIZE as u32, BOARD_HEIGHT * TILE_SIZE as u32])
            .exit_on_esc(true));
    let mut game = Game::new();
    for e in window.events() {
        use piston::input::Button;
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            // println!("now: {:?}, last: {:?}", key, game.last_key ); // debug

            if let Some(x) = PREV_MOVE_REMAP.get(&key) {
                if x != &game.last_key {
                    game.key_press(key);
                }
            } else {
                game.key_press(key);
            }

        }

        if let Some(args) = e.update_args() {
            game.update(&args);
        }
    }
}
