#![allow(dead_code)]

use rand::{rngs::ThreadRng, Rng};

trait Game {
    type Action;

    fn next(&mut self, action: Self::Action) -> bool;
    fn render(&self);
    fn render_result(&self);
}

trait Strategy {
    type Game: Game;
    fn next_action(&mut self, game: &Self::Game) -> <Self::Game as Game>::Action;
}

fn simulate_game<G: Game>(game: &mut G, mut strategy: impl Strategy<Game = G>) {
    game.render();
    let mut next_action = strategy.next_action(game);
    while !game.next(next_action) {
        game.render();
        next_action = strategy.next_action(game);
    }
    game.render_result();
}

struct MyGame {
    state: MyGameState,
    leftover_moves: u8,
    score: u32,
}

#[derive(Clone)]
struct MyGameState {
    position: MyGamePosition,
    board: [[u8; 5]; 5],
}

impl MyGameState {
    fn render(&self) {
        println!("+---+---+---+---+---+");
        for (i, row) in self.board.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if i == self.position.1 && j == self.position.0 {
                    print!("| X ");
                } else if *cell == 0 {
                    print!("|   ");
                } else {
                    print!("| {} ", cell);
                }
            }
            println!("|");
            println!("+---+---+---+---+---+");
        }
    }

    fn action(&mut self, action: MyGameAction) -> u32 {
        use MyGameAction::*;
        match action {
            Up => self.position.1 = self.position.1.saturating_sub(1),
            Down => self.position.1 = (self.position.1 + 1).min(4),
            Left => self.position.0 = self.position.0.saturating_sub(1),
            Right => self.position.0 = (self.position.0 + 1).min(4),
        };
        std::mem::take(&mut self.board[self.position.1][self.position.0]) as u32
    }

    fn predict_score(&self, action: MyGameAction) -> u32 {
        let mut state = self.clone();
        state.action(action)
    }
}

#[derive(Clone, Copy)]
struct MyGamePosition(usize, usize);

#[derive(Clone, Copy)]
enum MyGameAction {
    Up,
    Down,
    Left,
    Right,
}

impl Game for MyGame {
    type Action = MyGameAction;

    fn next(&mut self, action: Self::Action) -> bool {
        if self.leftover_moves == 0 {
            true
        } else {
            self.leftover_moves -= 1;
            self.score += self.state.action(action);
            false
        }
    }

    fn render(&self) {
        self.state.render();
        println!("Leftover moves: {}", self.leftover_moves);
        println!("Score: {}", self.score);
    }

    fn render_result(&self) {
        println!("Game over! Your score is {}", self.score);
    }
}

impl MyGame {
    fn new() -> Self {
        Self {
            state: MyGameState {
                position: MyGamePosition(0, 0),
                board: [
                    [0, 2, 3, 4, 5],
                    [6, 7, 8, 9, 0],
                    [1, 2, 3, 4, 5],
                    [6, 7, 8, 9, 0],
                    [1, 2, 3, 4, 5],
                ],
            },
            leftover_moves: 10,
            score: 0,
        }
    }
}
struct MyGameGreedyStrategy;

impl Strategy for MyGameGreedyStrategy {
    type Game = MyGame;

    fn next_action(&mut self, game: &Self::Game) -> <Self::Game as Game>::Action {
        let mut best_action = MyGameAction::Up;
        let mut best_score = game.state.predict_score(best_action);
        for action in [MyGameAction::Down, MyGameAction::Left, MyGameAction::Right] {
            let score = game.state.predict_score(action);
            if score > best_score {
                best_score = score;
                best_action = action;
            }
        }
        best_action
    }
}

struct MyGameRandomStrategy {
    rng: ThreadRng,
}

impl Default for MyGameRandomStrategy {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}

impl Strategy for MyGameRandomStrategy {
    type Game = MyGame;

    fn next_action(&mut self, _game: &Self::Game) -> <Self::Game as Game>::Action {
        use MyGameAction::*;
        match self.rng.gen_range(0..4) {
            0 => Up,
            1 => Down,
            2 => Left,
            3 => Right,
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_game() {
    let mut game = MyGame::new();
    simulate_game(&mut game, MyGameGreedyStrategy);
    println!();
    println!();
    game = MyGame::new();
    simulate_game(&mut game, MyGameRandomStrategy::default());
}