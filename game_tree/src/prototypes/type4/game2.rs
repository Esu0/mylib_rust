use rand::Rng;

use super::{game::Player, Game, NoPlayerState, Strategy};

struct Game2 {
    containers: u32,
    chips_min: u32,
    chips_max: u32,
}

struct GlobalState {
    containers: Box<[u32]>,
    empty_containers: u32,
    winner: Option<Player>,
}

#[derive(Clone, Copy)]
struct Action {
    container: usize,
    chips: u32,
}

impl Game for Game2 {
    type Action = Action;
    type GlobalState = GlobalState;
    type Player = Player;
    type PlayerState = ();
    type PlayerStateSet = NoPlayerState<Player>;

    fn initial_state(&self) -> (Self::GlobalState, Self::PlayerStateSet, Self::Player) {
        let mut rng = rand::thread_rng();
        let containers: Box<[u32]> = (0..self.containers)
            .map(|_| rng.gen_range(self.chips_min..=self.chips_max))
            .collect();
        (
            GlobalState {
                empty_containers: containers.iter().filter(|&&x| x == 0).count() as u32,
                containers,
                winner: None,
            },
            Default::default(),
            Player::First,
        )
    }

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        _player_state_set: &mut Self::PlayerStateSet,
        player: Self::Player,
        action: Self::Action,
    ) -> Option<Self::Player> {
        if let Some(container) = global_state.containers.get_mut(action.container) {
            if *container != 0 {
                *container = container.saturating_sub(action.chips.max(1));
                if *container == 0 {
                    global_state.empty_containers += 1;
                    if global_state.empty_containers >= self.containers {
                        global_state.winner = Some(player);
                        return None;
                    }
                }
                return Some(player.other());
            }
        }
        global_state.winner = Some(player.other());
        None
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
        for &container in &*global_state.containers {
            print!("{:>3}", container);
        }
        println!();
        println!("{player}'s turn");
        println!();
    }

    fn render_result(
        &self,
        global_state: &Self::GlobalState,
        _players_state: &Self::PlayerStateSet,
        _player: Self::Player,
    ) {
        println!("{} won!", global_state.winner.unwrap());
    }
}

struct SimpleStrategy;

impl Strategy for SimpleStrategy {
    type Game = Game2;

    fn next_action(
        &mut self,
        _game: &Self::Game,
        global_state: &<Self::Game as Game>::GlobalState,
        _player_state: &<Self::Game as Game>::PlayerState,
    ) -> <Self::Game as Game>::Action {
        let (container, &chips) = global_state
            .containers
            .iter()
            .enumerate()
            .find(|&(_, &container)| container != 0)
            .unwrap();
        Action {
            container,
            chips: chips.min(3),
        }
    }
}

struct InputStrategy;

impl Strategy for InputStrategy {
    type Game = Game2;

    fn next_action(
        &mut self,
        _game: &Self::Game,
        global_state: &<Self::Game as Game>::GlobalState,
        _player_state: &<Self::Game as Game>::PlayerState,
    ) -> <Self::Game as Game>::Action {
        println!(
            "info: Xor sum {}",
            global_state.containers.iter().fold(0u32, |acc, &x| acc ^ x)
        );
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).unwrap();
        let mut iter = s.split_whitespace().map(|x| x.parse().unwrap());
        let container = iter.next().unwrap();
        let chips = iter.next().unwrap();
        Action {
            container: container as usize,
            chips,
        }
    }
}

struct RandomStrategy;

impl Strategy for RandomStrategy {
    type Game = Game2;

    fn next_action(
        &mut self,
        _game: &Self::Game,
        global_state: &<Self::Game as Game>::GlobalState,
        _player_state: &<Self::Game as Game>::PlayerState,
    ) -> <Self::Game as Game>::Action {
        let mut rng = rand::thread_rng();
        let (container, chips) = loop {
            let a = rng.gen_range(0..global_state.containers.len());
            let chips = global_state.containers[a];
            if chips != 0 {
                break (a, chips);
            }
        };
        let chips = rng.gen_range(1..=chips);
        Action { container, chips }
    }
}
#[cfg(test)]
#[test]
fn test_game() {
    let game = Game2 {
        containers: 5,
        chips_min: 1,
        chips_max: 10,
    };

    super::simulate_game(game, (InputStrategy, RandomStrategy));
}
