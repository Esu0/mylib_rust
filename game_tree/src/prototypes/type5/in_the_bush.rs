use std::cell::RefCell;

use arrayvec::ArrayVec;
use rand::{seq::SliceRandom, Rng};

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

impl PersonTile {
    fn all(player_num: u8) -> ArrayVec<Self, 9> {
        if player_num == 3 {
            [
                Self::Two,
                Self::Three,
                Self::Four,
                Self::Five,
                Self::Six,
                Self::Seven,
                Self::Eight,
            ]
            .into_iter()
            .collect()
        } else if player_num == 4 {
            [
                Self::Two,
                Self::Three,
                Self::Four,
                Self::Five,
                Self::Six,
                Self::Seven,
                Self::Eight,
                Self::X1,
            ]
            .into_iter()
            .collect()
        } else if player_num == 5 {
            ArrayVec::from([
                Self::Two,
                Self::Three,
                Self::Four,
                Self::Five,
                Self::Six,
                Self::Seven,
                Self::Eight,
                Self::X1,
                Self::X2,
            ])
        } else {
            panic!("invalid player_num: {}", player_num);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Suspect {
    One,
    Two,
    Three,
}

impl Suspect {
    fn other(self) -> [Self; 2] {
        match self {
            Self::One => [Self::Two, Self::Three],
            Self::Two => [Self::One, Self::Three],
            Self::Three => [Self::One, Self::Two],
        }
    }
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

impl Board {
    fn draw(game: &InTheBush) -> Self {
        let mut tiles = PersonTile::all(game.player_num);
        RNG.with_borrow_mut(|rng| {
            tiles.shuffle(rng);
        });
        let mut iter = tiles.into_iter();
        let suspect = [
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ];
        let dead = iter.next().unwrap();
        let other = iter.collect();
        Self {
            suspect,
            dead,
            other,
        }
    }

    fn get_tile_suspect(&self, suspect: Suspect) -> PersonTile {
        match suspect {
            Suspect::One => self.suspect[0],
            Suspect::Two => self.suspect[1],
            Suspect::Three => self.suspect[2],
        }
    }
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

    fn set_tile_suspect(&mut self, suspect: Suspect, tile: PersonTile) {
        match suspect {
            Suspect::One => self.suspect[0] = Some(tile),
            Suspect::Two => self.suspect[1] = Some(tile),
            Suspect::Three => self.suspect[2] = Some(tile),
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
            playerstates: (0..game.player_num)
                .map(|_| PlayerState {
                    board: PartialBoard::unknown(game),
                })
                .collect(),
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
                player_state_set.board = Some(Board::draw(self));
                // TODO: 次の人にタイルを渡す(前の人のタイルを見る)操作を実装する
                Some(first)
            }
            Phase::FirstOnTheScene => {
                let Some(decision) = action else {
                    return Some(player);
                };
                global_state.unseen_marker = Some(decision);
                let board = player_state_set.board.as_ref().unwrap();
                for suspect in decision.other() {
                    player_state_set.playerstates[player.0 as usize]
                        .board
                        .set_tile_suspect(suspect, board.get_tile_suspect(suspect));
                }
                global_state.phase = Phase::Declare;
                Some(player.next(self))
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
