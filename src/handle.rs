use std::any::Any;
use std::time::Duration;

use crossbeam_channel::Sender;

use crate::callback::{BoxedCallback, Callback};

/// This means the wheel has been dropped
#[derive(Clone, Copy, Debug)]
pub struct DanglingAddHandle;

/// Allow adding tasks while wheel is running.
/// These tasks will be added to the wheel in the next tick.
#[derive(Clone, Debug)]
pub struct AddHandle(pub(crate) Sender<(BoxedCallback, Duration, Box<dyn Any + Send>)>);

impl AddHandle {
    /// Add tasks to after a certain interval
    #[inline]
    pub fn add<C>(&self, cb: C, dur: Duration) -> Result<(), DanglingAddHandle>
    where
        C: Callback<()> + Send + 'static,
    {
        self.0
            .send((cb.boxed(), dur, Box::new(())))
            .map_err(|_| DanglingAddHandle)
    }

    // like `add` but with context
    #[inline]
    pub fn add_with_ctx<C, T>(&self, cb: C, dur: Duration, ctx: T) -> Result<(), DanglingAddHandle>
    where
        C: Callback<(T,)> + Send + 'static,
        T: Send + 'static,
    {
        self.0
            .send((cb.boxed(), dur, Box::new(ctx)))
            .map_err(|_| DanglingAddHandle)
    }
}
