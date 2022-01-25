use std::time::Duration;

/// Behavior after task completion
#[derive(Debug, Copy, Clone)]
pub enum Behave {
    /// cancel the task, which is the default behavior.
    Cancel,
    /// change the time interval and continue the task.
    Change(Duration),
    /// repeat the task.
    Repeat,
}

impl From<()> for Behave {
    fn from(_: ()) -> Self {
        Behave::Cancel
    }
}

impl From<Duration> for Behave {
    fn from(dur: Duration) -> Self {
        Behave::Change(dur)
    }
}
