trait Game {
    type Action;
    type State;
    type GlobalState;
    type PlayerState;
    type Player;

    fn next(&self, state: &mut Self::State, action: Self::Action) -> bool;
    fn render(&self, state: &Self::State);
    fn render_result(&self, state: &Self::State);
    fn split_state<'a>(&self, state: &'a Self::State) -> (&'a Self::GlobalState, &'a Self::PlayerState);
}

trait MultiPlayerGame {
    /// グローバルに見える状態。全プレイヤーから同じものが見える。
    type GlobalState;
    type Player;
    /// 各プレイヤーから見える状態。
    /// 不完全情報ゲームのときに使う。
    /// 完全情報ゲームの場合は[`MultiPlayerGame::GlobalState`]のみを使用し、
    /// この型はユニット型にする。
    type PlayersState: PlayersState<Player = Self::Player>;
    type Action;

    fn next(
        &self,
        global_state: &mut Self::GlobalState,
        players_state: &mut Self::PlayersState,
        player: Self::Player,
        action: Self::Action,
    ) -> Option<Self::Player>;

    fn render(&self, global_state: &Self::GlobalState, players_state: &Self::PlayersState, player: Self::Player);
    fn render_result(&self, global_state: &Self::GlobalState, players_state: &Self::PlayersState, player: Self::Player);

    fn players(&self) -> impl IntoIterator<Item = Self::Player>;
}

trait PlayersState {
    type Player;
    type State;
    fn get(&self, player: Self::Player) -> &Self::State;
}

impl<P: Clone, T: MultiPlayerGame<Player = P>> Game for T {
    type Action = <T as MultiPlayerGame>::Action;
    type State = (T::GlobalState, T::PlayersState, P);
    type GlobalState = T::GlobalState;
    type PlayerState = <T::PlayersState as PlayersState>::State;
    type Player = P;

    fn next(&self, state: &mut Self::State, action: Self::Action) -> bool {
        let (global_state, players_state, player) = state;
        if let Some(next_player) = self.next(global_state, players_state, player.clone(), action) {
            *player = next_player;
            true
        } else {
            false
        }
    }

    fn render(&self, state: &Self::State) {
        for player in self.players() {
            self.render(&state.0, &state.1, player);
        }
    }

    fn render_result(&self, state: &Self::State) {
        for player in self.players() {
            self.render_result(&state.0, &state.1, player);
        }
    }

    fn split_state<'a>(&self, state: &'a Self::State) -> (&'a Self::GlobalState, &'a Self::PlayerState) {
        (&state.0, &state.1.get(state.2.clone()))
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
    type Strategy: Strategy;

    fn get_mut(&mut self, player: <<Self::Strategy as Strategy>::Game as Game>::Player) -> &mut Self::Strategy;
}

fn simulate_game<G: Game>(
    game: &mut G,
    initial_state: <G as Game>::State,
    mut strategy: impl Strategy<Game = G>,
) {
    game.render(&initial_state);
    let mut state = initial_state;
    let (global_state, player_state) = game.split_state(&state);
    let mut next_action = strategy.next_action(game, global_state, player_state);
    while !game.next(&mut state, next_action) {
        game.render(&state);
        let (global_state, player_state) = game.split_state(&state);
        next_action = strategy.next_action(game, global_state, player_state);
    }
    game.render_result(&state);
}
