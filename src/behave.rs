use std::time::Duration;

/// Behavior after task completion
///
/// `Cancel`: cancel the task, which is the default behavior.
/// `Change`: change the time interval and continue the task.
/// `Repeat`: repeat the task.
#[derive(Debug, Copy, Clone)]
pub enum Behave {
    Cancel,
    Change(Duration),
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
