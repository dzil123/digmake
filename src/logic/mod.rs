enum State {
    Handshake,
    State,
    Login,
    Play,
    Closed,
}

impl Default for State {
    fn default() -> Self {
        Self::Handshake
    }
}
