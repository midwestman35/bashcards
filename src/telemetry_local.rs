#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocalStats {
    pub attempts: usize,
    pub completions: usize,
}
