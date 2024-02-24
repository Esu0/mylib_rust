mod in_the_bush;

use std::{array, marker::PhantomData};

use rand::{rngs::ThreadRng, Rng};

trait Game {
    type Action;
    type GlobalState;
    type PlayerState;
    type Player: Copy;
    type PlayerStateSet: PlayerStateSet<Player = Self::Player, State = Self::PlayerState>;

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        player_state_set: &mut Self::PlayerStateSet,
        player: Self::Player,
        action: Self::Action,
    ) -> Option<Self::Player>;

    fn render(
        &self,
        global_state: &Self::GlobalState,
        players_state: &Self::PlayerStateSet,
        player: Self::Player,
    );
    fn render_result(
        &self,
        global_state: &Self::GlobalState,
        players_state: &Self::PlayerStateSet,
        player: Self::Player,
    );

    fn players(&self) -> impl IntoIterator<Item = Self::Player>;

    fn initial_state(&self) -> (Self::GlobalState, Self::PlayerStateSet, Self::Player);
}

trait PlayerStateSet {
    type Player;
    type State;
    fn get(&self, player: Self::Player) -> &Self::State;
}

struct NoPlayerState<P>(PhantomData<P>);
impl<P> Default for NoPlayerState<P> {
    fn default() -> Self {
        NoPlayerState(PhantomData)
    }
}

impl<P> PlayerStateSet for NoPlayerState<P> {
    type Player = P;
    type State = ();
    fn get(&self, _player: Self::Player) -> &Self::State {
        &()
    }
}

trait Strategy {
    type Game: Game;
    fn next_action(
        &mut self,
        game: &Self::Game,
        global_state: &<Self::Game as Game>::GlobalState,
        player_state: &<Self::Game as Game>::PlayerState,
    ) -> <Self::Game as Game>::Action;
}

trait StrategySet {
    type Game: Game;

    fn get_mut(
        &mut self,
        player: <Self::Game as Game>::Player,
    ) -> &mut dyn Strategy<Game = Self::Game>;
}

#[derive(Clone, Copy)]
struct Player<const N: u8> {
    index: u8,
}

impl<const N: u8> Player<N> {
    fn next(self) -> Self {
        if self.index >= N {
            unsafe {
                std::hint::unreachable_unchecked();
            }
        }
        Self {
            index: (self.index + 1) % N,
        }
    }

    fn new(index: u8) -> Self {
        if index >= N {
            panic!("Player index out of range");
        }
        Self { index }
    }

    unsafe fn new_unchecked(index: u8) -> Self {
        Self { index }
    }
}

impl<T: CompleteInformationGame> Game for T {
    type Action = T::Action;
    type GlobalState = T::GlobalState;
    type Player = T::Player;
    type PlayerState = ();
    type PlayerStateSet = NoPlayerState<T::Player>;

    fn initial_state(&self) -> (Self::GlobalState, Self::PlayerStateSet, Self::Player) {
        let (gs, p) = self.initial_state();
        (gs, NoPlayerState::default(), p)
    }

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        _player_state_set: &mut Self::PlayerStateSet,
        player: Self::Player,
        action: Self::Action,
    ) -> Option<Self::Player> {
        self.next(global_state, player, action)
    }

    fn players(&self) -> impl IntoIterator<Item = Self::Player> {
        self.players()
    }

    fn render(
        &self,
        global_state: &Self::GlobalState,
        _players_state: &Self::PlayerStateSet,
        player: Self::Player,
    ) {
        self.render(global_state, player);
    }

    fn render_result(
        &self,
        global_state: &Self::GlobalState,
        _players_state: &Self::PlayerStateSet,
        player: Self::Player,
    ) {
        self.render_result(global_state, player);
    }
}

trait CompleteInformationGame {
    type Action;
    type GlobalState;
    type Player: Copy;

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        player: Self::Player,
        action: Self::Action,
    ) -> Option<Self::Player>;

    fn render(&self, global_state: &Self::GlobalState, player: Self::Player);
    fn render_result(&self, global_state: &Self::GlobalState, player: Self::Player);

    fn players(&self) -> impl IntoIterator<Item = Self::Player>;

    fn initial_state(&self) -> (Self::GlobalState, Self::Player);
}

fn simulate_game<G: Game>(game: G, mut strategy: impl StrategySet<Game = G>) {
    let (mut gs, mut pss, mut p) = game.initial_state();
    game.render(&gs, &pss, p);
    let mut next_action = strategy.get_mut(p).next_action(&game, &gs, pss.get(p));
    while let Some(next_player) = game.next(&mut gs, &mut pss, p, next_action) {
        game.render(&gs, &pss, next_player);
        next_action = strategy
            .get_mut(next_player)
            .next_action(&game, &gs, pss.get(next_player));
        p = next_player;
    }
    game.render(&gs, &pss, p);
    game.render_result(&gs, &pss, p);
}

mod mygame {
    use super::*;
    struct MyGame;

    impl Game for MyGame {
        type Action = MyGameAction;
        type GlobalState = MyGameState;
        type Player = ();
        type PlayerState = ();
        type PlayerStateSet = NoPlayerState<()>;

        fn initial_state(&self) -> (Self::GlobalState, Self::PlayerStateSet, Self::Player) {
            let mut rng = rand::thread_rng();
            let mut board = array::from_fn(|_| array::from_fn(|_| rng.gen_range(0u8..10)));
            board[2][2] = 0;
            let initial_state = MyGameState {
                leftover_moves: 10,
                score: 0,
                position: MyGamePosition(2, 2),
                board,
            };
            (initial_state, NoPlayerState::default(), ())
        }

        fn next(
            &self,
            global_state: &mut Self::GlobalState,
            _player_state_set: &mut Self::PlayerStateSet,
            _player: Self::Player,
            action: Self::Action,
        ) -> Option<Self::Player> {
            use MyGameAction::*;
            match action {
                Up => global_state.position.1 = global_state.position.1.saturating_sub(1),
                Down => global_state.position.1 = (global_state.position.1 + 1).min(4),
                Left => global_state.position.0 = global_state.position.0.saturating_sub(1),
                Right => global_state.position.0 = (global_state.position.0 + 1).min(4),
            };
            let addtional_score = std::mem::take(
                &mut global_state.board[global_state.position.1][global_state.position.0],
            ) as u32;
            global_state.score += addtional_score;
            global_state.leftover_moves -= 1;
            if global_state.leftover_moves == 0 {
                None
            } else {
                Some(())
            }
        }

        fn players(&self) -> impl IntoIterator<Item = Self::Player> {
            std::iter::once(())
        }

        fn render(
            &self,
            global_state: &Self::GlobalState,
            _players_state: &Self::PlayerStateSet,
            _player: Self::Player,
        ) {
            global_state.render();
        }

        fn render_result(
            &self,
            global_state: &Self::GlobalState,
            _players_state: &Self::PlayerStateSet,
            _player: Self::Player,
        ) {
            println!("Game over! Your score is {}", global_state.score);
        }
    }

    #[derive(Clone, Copy)]
    enum MyGameAction {
        Up,
        Down,
        Left,
        Right,
    }

    #[derive(Clone)]
    struct MyGameState {
        leftover_moves: u8,
        score: u32,
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
            println!("Leftover moves: {}", self.leftover_moves);
            println!("Score: {}", self.score);
        }
    }
    #[derive(Clone, Copy)]
    struct MyGamePosition(usize, usize);

    struct MyGameGreedyStrategy;

    impl Strategy for MyGameGreedyStrategy {
        type Game = MyGame;

        fn next_action(
            &mut self,
            game: &Self::Game,
            global_state: &<Self::Game as Game>::GlobalState,
            _player_state: &<Self::Game as Game>::PlayerState,
        ) -> <Self::Game as Game>::Action {
            let best = {
                use MyGameAction::*;
                [Up, Down, Left, Right]
                    .into_iter()
                    .map(|action| {
                        let mut state = global_state.clone();
                        game.next(&mut state, &mut NoPlayerState::default(), (), action);
                        (state.score, action)
                    })
                    .max_by_key(|x| x.0)
                    .unwrap_or_else(|| unreachable!())
            };
            best.1
        }
    }

    struct MyGameRandomStrategy(ThreadRng);

    impl MyGameRandomStrategy {
        fn new() -> Self {
            Self(rand::thread_rng())
        }
    }

    impl Strategy for MyGameRandomStrategy {
        type Game = MyGame;

        fn next_action(
            &mut self,
            _game: &Self::Game,
            _global_state: &<Self::Game as Game>::GlobalState,
            _player_state: &<Self::Game as Game>::PlayerState,
        ) -> <Self::Game as Game>::Action {
            use MyGameAction::*;
            let actions = [Up, Down, Left, Right];
            actions[self.0.gen_range(0usize..4)]
        }
    }
    impl<T: Strategy<Game = MyGame>> StrategySet for T {
        type Game = T::Game;

        fn get_mut(
            &mut self,
            _player: <Self::Game as Game>::Player,
        ) -> &mut dyn Strategy<Game = Self::Game> {
            self
        }
    }

    struct MyGameInputStrategy;

    impl Strategy for MyGameInputStrategy {
        type Game = MyGame;

        fn next_action(
            &mut self,
            _game: &Self::Game,
            _global_state: &<Self::Game as Game>::GlobalState,
            _player_state: &<Self::Game as Game>::PlayerState,
        ) -> <Self::Game as Game>::Action {
            use MyGameAction::*;
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                match input.trim() {
                    "w" => return Up,
                    "s" => return Down,
                    "a" => return Left,
                    "d" => return Right,
                    _ => println!("Invalid input!"),
                }
            }
        }
    }

    #[test]
    fn test_game() {
        simulate_game(MyGame, MyGameGreedyStrategy);
    }
}
