#![allow(dead_code)]
#![allow(unused_variables)]

use rand::Rng;
use std::io::{self};
use std::time::{Instant};
use rand::seq::SliceRandom;
use std::collections::VecDeque;

struct Game {
    size: u8,
    done: bool,
    score: u8,
    health: i8,
    starting_idx: u8,
    body_list: VecDeque<u8>, // im assuming we are going to be storing indexes
    snake_head_board: u128,
    snake_body_board: u128,
    food_board: u128,
    first_column: u128,
    last_column: u128,
    first_row: u128,
    msb_board: u128,
}

impl Game {
    fn new(size: u8) -> Game {

        // let starting_idx: u8 = ((size.pow(2) as f32) / 2.0).ceil() as u8;
        let starting_idx: u8 = ((size.pow(2)) as f32 / 2.0) as u8;
        let snake_head_board: u128 = 1u128 << starting_idx;


        let (mut first_column, mut last_column, mut first_row) = (0u128, 0u128, 0u128);

        for i in 0..size {
            first_column |= 0b1u128 << (size * i + size - 1);
            last_column  |= 0b1u128 << (size * i);
            first_row    |= 0b1u128 << (size * size - i - 1);
            // last_row     |= 0b1u128 << (size - i - 1);
        }

        // we need to exlude the last bits because in 11x11, it only goes up to 128; dont want to put food past 121 
        let mut msb_board = 0b0u128;
        for i in 121..128 {
            msb_board |= 0b1u128 << i;
        }

        let body_list: VecDeque<u8> = VecDeque::from(vec![starting_idx]);


        let mut game: Game = Game {size, 
            done: false,
            score: 0,
            health: 100,
            starting_idx,
            body_list: body_list,
            snake_head_board, 
            snake_body_board: 0b0,
            food_board: 0,
            first_column,
            last_column,
            first_row,
            msb_board,
        };

        game.get_food();
        return game;
    }

    fn step(&mut self, action: u8) {
        let mut new_head = self.snake_head_board;

        // left, right, up, down
        match action {
            0 => new_head <<= 0b1,
            1 => new_head >>= 0b1,
            2 => new_head <<= self.size,
            _ => new_head >>= self.size,
        }
        
        // check to see if it is faster to cast or to change the list to u32
        // appending the previous head index to the list 
        let idx: u8 = new_head.trailing_zeros() as u8;
        self.body_list.push_back(idx);

        // adding the old snake position to be part of the body 
        self.snake_body_board |= self.snake_head_board;

        // setting the new/old head 
        let old_head = self.snake_head_board;
        self.snake_head_board = new_head;

        // if snake ate apple
        if self.snake_head_board & self.food_board != 0 {
            self.get_food();
            self.health = 100;
            self.score += 1;
        }
        // snake did not eat the apple
        else {
            self.health -= 1;
            let idx_to_remove: u8 = self.body_list.pop_front().unwrap_or(0);
            self.snake_body_board ^= 0b1u128 << idx_to_remove;
        }

        // terminal states 
        
        // running into itself
        if new_head & self.snake_body_board != 0 {
            self.done = true;
        }
        // going left
        else if old_head & self.first_column != 0 && action == 0 {
            self.done = true;
        }
        // going right 
        else if old_head & self.last_column != 0 && action == 1 {
            self.done = true;
        }
        // going up ; there are some positions where if we go up, it doesnt equal 0
        else if old_head & self.first_row != 0 && action == 2 {
            self.done = true;
        }
        // going down ; if we are on the last row, we will always be 0 if we shift right 
        else if new_head == 0 {
            self.done = true;
        }
        else if self.health <= 0 {
            self.done = true;
        }
    }


    fn get_food(&mut self) {
        // creating board for all open spaces 
        let mut all_boards = !(self.snake_body_board | self.snake_head_board | self.msb_board);
        let mut food_choices: Vec<u32> = vec![];
        // xor away each open index until we have no more open spaces
        while all_boards != 0 {
            let index = all_boards.trailing_zeros();
            food_choices.push(index);
            all_boards ^= 0b1u128 << index;
        }

        let mut rng = rand::thread_rng();
        let random_index = food_choices.choose(&mut rng).unwrap_or(&0);

        self.food_board = 0b1u128 << random_index;
    }


    fn reset(&mut self) {
        self.snake_head_board = 0b1u128 << self.starting_idx;
        self.get_food();
        self.score = 0;
        self.done = false;
        self.health = 100;

        self.snake_body_board = 0b0u128;
        self.body_list.clear();
        self.body_list.push_back(self.starting_idx);
    }


    fn print_board(&self) {
        // bottom right is lsb ; top left is msb 
        for i in (0..self.size).rev() {
            for j in (0..self.size).rev() {
                let idx = 0b1u128 << (i * self.size + j);
                if self.snake_head_board & idx != 0 {
                    print!("H ");
                }
                else if self.food_board & idx != 0 {
                    print!("F ");
                }
                else if self.snake_body_board & idx != 0 {
                    print!("B ");
                }
                else {
                    print!("| ");
                }
            }
            println!();
        }
    }


    fn print_individual_board(&self, board: u128) {
        for i in (0..self.size).rev() {
            for j in (0..self.size).rev() {
                let idx = 0b1u128 << (i * self.size + j);
                    if board & idx != 0 {
                        print!("1 ");
                    }
                    else {
                        print!("0 ")
                    }
                }
                println!();
            }
        }        
}


fn play_game() {
    let mut game: Game = Game::new(11);
    game.print_board();
    let mut input: String = String::new();

    loop {
        io::stdin().read_line(&mut input).expect("failed to read from stdin");
        let action = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Enter a valid number");
                return;
            }
        };

        input.clear();
        game.step(action);
        game.print_board();

        if game.done {
            println!("DEAD");

            game.reset();
            game.print_board();

        }
    }
}


fn run_test() {
    let mut game: Game = Game::new(11);
    let num_games = 10_000;

    let mut rng = rand::thread_rng();
    let mut action: u8;

    let start = Instant::now();

    for _ in 0..num_games {
        game.reset();
        while !game.done {
            action = rng.gen_range(0..=3);
            game.step(action);
        }
    }

    let end = Instant::now();
    let elapsed_time = end - start;

    println!("elapsed time {:?}", elapsed_time.as_secs_f64());
}


fn main() {
    run_test();
}
