use std::cell::RefCell;

use arrayvec::ArrayVec;
use rand::Rng;

struct InTheBush {
    player_num: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PersonTile {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    X1 = 9,
    X2 = 10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Suspect {
    One,
    Two,
    Three,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// person tileを引いて「第一発見者」を決定するフェーズ
    Setup,
    /// 「第一発見者」が見るタイルを決定するフェーズ
    FirstOnTheScene,
    /// プレイヤーが犯人を推理するフェーズ
    Declare,
}

struct Board {
    suspect: [PersonTile; 3],
    dead: PersonTile,
    other: ArrayVec<PersonTile, 5>,
}

struct PartialBoard {
    suspect: [Option<PersonTile>; 3],
    other: ArrayVec<Option<PersonTile>, 5>,
}

struct GlobalState {
    round: u8,
    phase: Phase,
    detective_chips: ArrayVec<Suspect, 5>,
    points: ArrayVec<i8, 5>,
    unseen_marker: Option<Suspect>,
    first_marker: Option<Player>,
}

struct PlayerState {
    board: PartialBoard,
}

impl PartialBoard {
    fn unknown(game: &InTheBush) -> Self {
        Self {
            suspect: [None, None, None],
            other: (0..game.player_num).map(|_| None).collect(),
        }
    }
}

struct PlayerStateSet {
    board: Option<Board>,
    playerstates: ArrayVec<PlayerState, 5>,
}

impl PlayerStateSet {
    fn initial(game: &InTheBush) -> Self {
        Self {
            board: None,
            playerstates: (0..game.player_num).map(|_| PlayerState {
                board: PartialBoard::unknown(game),
            }).collect(),
        }
    }
}

impl super::PlayerStateSet for PlayerStateSet {
    type Player = Player;
    type State = PlayerState;

    fn get(&self, player: Self::Player) -> &Self::State {
        &self.playerstates[player.0 as usize]
    }
}

#[derive(Clone, Copy)]
struct Player(u8);

impl Player {
    fn random(game: &InTheBush) -> Self {
        RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            Player(rng.gen_range(0..game.player_num))
        })
    }

    fn next(self, game: &InTheBush) -> Self {
        Player((self.0 + 1) % game.player_num)
    }
}

thread_local! {
    static RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

impl super::Game for InTheBush {
    type Action = Option<Suspect>;
    type GlobalState = GlobalState;
    type Player = Player;
    type PlayerState = PlayerState;
    type PlayerStateSet = PlayerStateSet;

    fn initial_state(&self) -> (Self::GlobalState, Self::PlayerStateSet, Self::Player) {
        let global_state = GlobalState {
            round: 0,
            phase: Phase::Setup,
            detective_chips: ArrayVec::new_const(),
            points: (0..self.player_num).map(|_| 0).collect(),
            unseen_marker: None,
            first_marker: None,
        };

        let player_state_set = PlayerStateSet::initial(self);
        let player = Player(0);
        (global_state, player_state_set, player)
    }

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        player_state_set: &mut Self::PlayerStateSet,
        player: Self::Player,
        action: Self::Action,
    ) -> Option<Self::Player> {
        let phase = global_state.phase;
        match phase {
            Phase::Setup => {
                let first = Player::random(self);
                global_state.first_marker = Some(first);
                global_state.phase = Phase::FirstOnTheScene;
                // TODO: 盤面の抽選を行う
                Some(first)
            }
            Phase::FirstOnTheScene => {
                let Some(decision) = action else {
                    return Some(player);
                };
                todo!()
            }
            Phase::Declare => {
                todo!()
            }
        }
    }

    fn players(&self) -> impl IntoIterator<Item = Self::Player> {
        (1..=self.player_num).map(Player)
    }

    fn render(
        &self,
        global_state: &Self::GlobalState,
        players_state: &Self::PlayerStateSet,
        player: Self::Player,
    ) {
        todo!()
    }

    fn render_result(
        &self,
        global_state: &Self::GlobalState,
        players_state: &Self::PlayerStateSet,
        player: Self::Player,
    ) {
        todo!()
    }
}
