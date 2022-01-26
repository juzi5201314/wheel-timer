use std::any::Any;
use std::marker::PhantomData;

use crate::behave::Behave;

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
        struct A<CB>(CB);

        impl<CB> Callback<Box<dyn Any + Send>> for A<CB>
        where
            CB: Callback<()> + Send + 'static,
        {
            fn call(&self, _: &mut Box<dyn Any + Send>) -> Behave {
                self.0.call(&mut ())
            }
        }

        Box::new(A(self))
    }
}

impl<F, B, C> Callback<(C,)> for F
where
    F: Fn(&mut C) -> B + Send + 'static,
    B: Into<Behave>,
    C: Send + 'static,
{
    fn call(&self, (ref mut ctx,): &mut (C,)) -> Behave {
        self(ctx).into()
    }

    fn boxed(self) -> BoxedCallback {
        struct A<C, CB>(CB, PhantomData<C>);

        impl<C, CB> Callback<Box<dyn Any + Send>> for A<C, CB>
        where
            CB: Callback<(C,)> + Send + 'static,
            C: Send + 'static,
        {
            fn call(&self, a: &mut Box<dyn Any + Send>) -> Behave {
                self.0
                    .call(unsafe { &mut *(a.downcast_mut::<C>().unwrap() as *mut C as *mut (C,)) })
            }
        }

        Box::new(A::<C, _>(self, PhantomData::default()))
    }
}
