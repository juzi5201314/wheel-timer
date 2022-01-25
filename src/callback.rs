use crate::behave::Behave;
use std::any::Any;

pub type BoxedCallback = Box<dyn Callback<Box<dyn Any + Send>> + Send>;

pub trait Callback<Ctx> {
    fn call(&self, context: &mut Ctx) -> Behave;

    fn boxed(self) -> BoxedCallback
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl<F, B> Callback<()> for F
where
    F: Fn() -> B + Send + 'static,
    B: Into<Behave>,
{
    fn call(&self, _: &mut ()) -> Behave {
        self().into()
    }

    fn boxed(self) -> BoxedCallback {
        struct A(Box<dyn Callback<()> + Send + 'static>);

        impl Callback<Box<dyn Any + Send>> for A {
            fn call(&self, _: &mut Box<dyn Any + Send>) -> Behave {
                self.0.call(&mut ())
            }
        }

        Box::new(A(Box::new(self)))
    }
}

impl<F, B, C> Callback<(C,)> for F
where
    F: Fn(&mut C) -> B + Send + 'static,
    B: Into<Behave>,
    C: 'static,
{
    fn call(&self, (ref mut ctx,): &mut (C,)) -> Behave {
        self(ctx).into()
    }

    fn boxed(self) -> BoxedCallback {
        struct A<C>(Box<dyn Callback<(C,)> + Send + 'static>);

        impl<C> Callback<Box<dyn Any + Send>> for A<C>
        where
            C: 'static,
        {
            fn call(&self, a: &mut Box<dyn Any + Send>) -> Behave {
                self.0
                    .call(unsafe { &mut *(a.downcast_mut::<C>().unwrap() as *mut C as *mut (C,)) })
            }
        }

        Box::new(A::<C>(Box::new(self)))
    }
}
