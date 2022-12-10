#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    NextPassage,
    PreviousPassage,
    RestartPassage,
}
