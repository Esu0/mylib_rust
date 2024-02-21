trait Game {
    type Action;
    type State;

    fn next(&self, state: &mut Self::State, action: Self::Action) -> bool;
    fn render(&self, state: &Self::State);
    fn render_result(&self, state: &Self::State);
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

    fn render(&self, global_state: &Self::GlobalState, players_state: &Self::PlayersState);
}

trait PlayersState {
    type Player;
    type State;
    fn get_mut(&mut self, player: Self::Player) -> &mut Self::State;
}

impl<P: Clone, T: MultiPlayerGame<Player = P>> Game for T {
    type Action = <T as MultiPlayerGame>::Action;
    type State = (T::GlobalState, T::PlayersState, P);

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
        todo!()
    }

    fn render_result(&self, state: &Self::State) {
        todo!()
    }
}
trait Strategy {
    type Game: Game;
    fn next_action(
        &mut self,
        game: &Self::Game,
        state: &<Self::Game as Game>::State,
    ) -> <Self::Game as Game>::Action;
}

fn simulate_game<G: Game>(
    game: &mut G,
    initial_state: <G as Game>::State,
    mut strategy: impl Strategy<Game = G>,
) {
    game.render(&initial_state);
    let mut state = initial_state;
    let mut next_action = strategy.next_action(game, &state);
    while !game.next(&mut state, next_action) {
        game.render(&state);
        next_action = strategy.next_action(game, &state);
    }
    game.render_result(&state);
}
