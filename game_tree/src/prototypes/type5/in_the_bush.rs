use arrayvec::ArrayVec;

struct InTheBush {
    player_num: u8,
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
enum Suspect {
    One,
    Two,
    Three,
}

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

struct PlayerStateSet {
    board: Option<Board>,
    playerstates: ArrayVec<PlayerState, 5>,
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

impl super::Game for InTheBush {
    type Action = Suspect;
    type GlobalState = GlobalState;
    type Player = Player;
    type PlayerState = PlayerState;
    type PlayerStateSet = PlayerStateSet;

    fn initial_state(&self) -> (Self::GlobalState, Self::PlayerStateSet, Self::Player) {
        todo!()
    }

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        player_state_set: &mut Self::PlayerStateSet,
        player: Self::Player,
        action: Self::Action,
    ) -> Option<Self::Player> {
        todo!()
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
