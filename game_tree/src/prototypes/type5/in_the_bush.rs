use std::{cell::RefCell, fmt::{self, Debug, Display}};

use arrayvec::ArrayVec;
use rand::{seq::SliceRandom, Rng};

use super::{Strategy, StrategySet};

struct InTheBush {
    player_num: u8,
    round_num: u8,
}

impl InTheBush {
    const fn new(player: u8, round: u8) -> Self {
        Self {
            player_num: player,
            round_num: round,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PersonTile {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    X1 = 0,
    X2 = 1,
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

    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Three,
            _ => panic!("index must be up to 2")
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

    fn culprit(&self) -> (Suspect, PersonTile) {
        let (i, tile) = if self.suspect.contains(&PersonTile::Five) {
            self.suspect.iter().copied().enumerate().min_by_key(|&(_, tile)| tile).unwrap()
        } else {
            self.suspect.iter().copied().enumerate().max_by_key(|&(_, tile)| tile).unwrap()
        };
        (Suspect::from_index(i), tile)
    }
}

impl From<PersonTile> for char {
    fn from(value: PersonTile) -> Self {
        use PersonTile::*;
        match value {
            Two => '2',
            Three => '3',
            Four => '4',
            Five => '5',
            Six => '6',
            Seven => '7',
            Eight => '8',
            X1 | X2 => 'X',
        }
    }
}

struct PartialBoard {
    suspect: [Option<PersonTile>; 3],
    other: ArrayVec<Option<PersonTile>, 5>,
}

impl Display for PartialBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn to_char(s: Option<PersonTile>) -> char {
            s.map_or('?', From::from)
        }
        writeln!(f, "suspects:")?;
        writeln!(f, "+---+---+---+")?;
        write!(f, "|")?;
        for s in self.suspect {
            write!(f, " {} |", to_char(s))?;
        }
        writeln!(f)?;
        writeln!(f, "+---+---+---+")?;
        writeln!(f, "dead: ?")?;
        writeln!(f)?;
        for (i, &s) in self.other.iter().enumerate() {
            writeln!(f, "player{} have {}", i + 1, to_char(s))?;
        }
        Ok(())
    }
}
struct GlobalState {
    round: u8,
    phase: Phase,
    detective_chips: ArrayVec<Suspect, 5>,
    points: ArrayVec<i8, 5>,
    unseen_marker: Option<Suspect>,
    first_marker: Option<Player>,
}

impl GlobalState {
    fn calculate_points(&mut self, board: &Board) {
        let (culprit, _) = board.culprit();
        for s in culprit.other() {
            let mut count = 0;
            let mut last = None;
            for (i, _) in self.detective_chips.iter().enumerate().filter(|(_, x)| **x == s) {
                count += 1;
                last = Some(i);
            }
            if let Some(last) = last {
                let player_num = self.points.len() as u8;
                self.points[self.first_marker.unwrap().nth(last, player_num).index()] -= count;
            }
        }
    }
}

impl Display for GlobalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "round {}", self.round)?;
        if let Some(first) = self.first_marker {
            writeln!(f, "{first} has first marker")?;
        }
        writeln!(f)?;
        writeln!(f, "suspects:")?;
        use Suspect::*;
        match self.unseen_marker {
            Some(One) => writeln!(f, "  v          ")?,
            Some(Two) => writeln!(f, "      v      ")?,
            Some(Three) => writeln!(f, "          v  ")?,
            None => (),
        };
        writeln!(f, "+---+---+---+")?;
        writeln!(f, "| ? | ? | ? |")?;
        for &s in &self.detective_chips {
            match s {
                One => writeln!(f, "| * |   |   |"),
                Two => writeln!(f, "|   | * |   |"),
                Three => writeln!(f, "|   |   | * |"),
            }?;
        }
        writeln!(f, "+---+---+---+")?;
        writeln!(f, "dead: ?")?;
        writeln!(f)?;
        writeln!(f, "points:")?;
        for (i, &point) in self.points.iter().enumerate() {
            writeln!(f, "\t {} has {point} point(s)", Player::from(i))?;
        }
        Ok(())
    }
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

impl From<usize> for Player {
    fn from(value: usize) -> Self {
        Self(value as u8)
    }
}

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

    fn index(self) -> usize {
        self.0 as usize
    }

    fn nth(self, n: usize, player_num: u8) -> Self {
        Self(((self.index() + n) % player_num as usize) as u8)
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "player{}", self.0)
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
                let board = Board::draw(self);
                global_state.first_marker = Some(first);
                global_state.phase = Phase::FirstOnTheScene;

                for (i, player_board) in player_state_set.playerstates.iter_mut().map(|s| &mut s.board).enumerate() {
                    let next_i = (i + 1) % self.player_num as usize;
                    player_board.other[i] = Some(board.other[i]); // second check
                    player_board.other[next_i] = Some(board.other[next_i]); // first check
                }

                player_state_set.board = Some(board);
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
                Some(player)
            }
            Phase::Declare => {
                let Some(decision) = action else {
                    return Some(player);
                };
                global_state.detective_chips.push(decision);
                if global_state.detective_chips.len() as u8 == self.player_num {
                    global_state.calculate_points(player_state_set.board.as_ref().unwrap());
                    None
                } else {
                    let next_player = player.next(self);
                    let PlayerStateSet {board, playerstates} = player_state_set;
                    let partial_board = &mut playerstates[next_player.index()].board;
                    for suspect in decision.other() {
                        partial_board.set_tile_suspect(suspect, board.as_ref().unwrap().get_tile_suspect(suspect));
                    }
                    Some(next_player)
                }
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
        println!("*** {player}'s turn ***");
        match global_state.phase {
            Phase::Setup => {
                println!("Any actions...");
            }
            Phase::FirstOnTheScene => {
                println!("{global_state}");
            }
            Phase::Declare => {
                println!("{global_state}");
            }
        }
    }

    fn render_result(
        &self,
        global_state: &Self::GlobalState,
        players_state: &Self::PlayerStateSet,
        player: Self::Player,
    ) {
        println!("{global_state}");
    }
}

struct SimpleStrategy;

impl Strategy for SimpleStrategy {
    type Game = InTheBush;

    fn next_action(
        &mut self,
        game: &Self::Game,
        global_state: &<Self::Game as super::Game>::GlobalState,
        player_state: &<Self::Game as super::Game>::PlayerState,
    ) -> <Self::Game as super::Game>::Action {
        Some(Suspect::One)
    }
}

struct Print;

impl Strategy for Print {
    type Game = InTheBush;

    fn next_action(
        &mut self,
        game: &Self::Game,
        global_state: &<Self::Game as super::Game>::GlobalState,
        player_state: &<Self::Game as super::Game>::PlayerState,
    ) -> <Self::Game as super::Game>::Action {
        println!("---------------- my information ----------------");
        println!("{}", player_state.board);
        println!("------------------------------------------------");

        Some(Suspect::One)
    }
}
struct Strategies(Vec<Box<dyn Strategy<Game = InTheBush>>>);
impl StrategySet for Strategies {
    type Game = InTheBush;

    fn get_mut(
        &mut self,
        player: <Self::Game as super::Game>::Player,
    ) -> &mut dyn Strategy<Game = Self::Game> {
        &mut *self.0[player.index()]
    }
}
#[cfg(test)]
mod tests {
    use crate::prototypes::type5::simulate_game;

    use super::*;

    #[test]
    fn game_test() {
        let strategies: Vec<Box<dyn Strategy<Game = InTheBush>>> = vec![Box::new(Print), Box::new(SimpleStrategy), Box::new(SimpleStrategy)];
        simulate_game(InTheBush::new(3, 1), Strategies(strategies))
    }
}