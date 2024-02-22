use std::fmt;

use super::{Game, NoPlayerState, Strategy};

struct TwoPlayerGameExample {
    start_number: u32,
    max_decrement: u32,
}

struct GlobalState {
    number: u32,
    won: Option<Player>,
}

#[derive(Clone, Copy)]
enum Player {
    First,
    Second,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::First => write!(f, "Player1"),
            Player::Second => write!(f, "Player2"),
        }
    }
}

impl Player {
    fn other(self) -> Self {
        match self {
            Player::First => Player::Second,
            Player::Second => Player::First,
        }
    }
}

impl Game for TwoPlayerGameExample {
    type Action = u32;
    type GlobalState = GlobalState;
    type Player = Player;
    type PlayerState = ();
    type PlayerStateSet = NoPlayerState<Player>;

    fn initial_state(&self) -> (Self::GlobalState, Self::PlayerStateSet, Self::Player) {
        (
            GlobalState {
                number: self.start_number,
                won: None,
            },
            NoPlayerState::default(),
            Player::First,
        )
    }

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        _player_state_set: &mut Self::PlayerStateSet,
        player: Self::Player,
        mut action: Self::Action,
    ) -> Option<Self::Player> {
        action = action.min(self.max_decrement);
        action = action.max(1);
        global_state.number = global_state.number.saturating_sub(action);
        if global_state.number == 0 {
            global_state.won = Some(player);
            None
        } else {
            Some(player.other())
        }
    }

    fn players(&self) -> impl IntoIterator<Item = Self::Player> {
        [Player::First, Player::Second]
    }

    fn render(
        &self,
        global_state: &Self::GlobalState,
        _players_state: &Self::PlayerStateSet,
        player: Self::Player,
    ) {
        println!("Number: {}", global_state.number);
        println!("{}'s turn", player);
    }

    fn render_result(
        &self,
        _global_state: &Self::GlobalState,
        _players_state: &Self::PlayerStateSet,
        player: Self::Player,
    ) {
        println!("{} won!", player);
    }
}

struct Strategy1;

impl Strategy for Strategy1 {
    type Game = TwoPlayerGameExample;

    fn next_action(
        &mut self,
        game: &Self::Game,
        _global_state: &<Self::Game as Game>::GlobalState,
        _player_state: &<Self::Game as Game>::PlayerState,
    ) -> <Self::Game as Game>::Action {
        game.max_decrement
    }
}

struct Strategy2;

impl Strategy for Strategy2 {
    type Game = TwoPlayerGameExample;

    fn next_action(
        &mut self,
        _game: &Self::Game,
        _global_state: &<Self::Game as Game>::GlobalState,
        _player_state: &<Self::Game as Game>::PlayerState,
    ) -> <Self::Game as Game>::Action {
        1
    }
}
