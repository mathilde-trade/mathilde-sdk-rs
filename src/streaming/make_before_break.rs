use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MakeBeforeBreakConfig {
    pub validation_window: Duration,
}

impl Default for MakeBeforeBreakConfig {
    fn default() -> Self {
        Self {
            validation_window: Duration::from_secs(2),
        }
    }
}
