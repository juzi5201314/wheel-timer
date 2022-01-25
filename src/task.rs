use crate::callback::BoxedCallback;
use crate::wheel::MoveTo;
use std::any::Any;
use std::fmt::{Debug, Formatter};

pub struct Task {
    pub cb: BoxedCallback,
    pub round: usize,
    pub move_to: MoveTo,
    pub ctx: Box<dyn Any + Send>,
    pub(crate) tick: usize,
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("task")
            .field("round", &self.round)
            .field("move_to", &self.move_to)
            .finish()
    }
}
