use pyrite_app_macros::generate_system_function_handlers;
use std::any::TypeId;

use crate::resource::{FromResourceBank, Res, ResMut, ResourceBank};

#[derive(Debug)]
pub enum ResourceDependency {
    Res(TypeId),
    ResMut(TypeId),
}

type SystemParameterTarget<'rb, P> = <P as SystemParameter>::Target<'rb>;

pub trait SystemParameter {
    type Target<'rb>: SystemParameter;

    fn from_resource_bank(resource_bank: &ResourceBank) -> Self::Target<'_>;
    fn dependency() -> ResourceDependency;
}

impl<R> SystemParameter for Res<'_, R>
where
    R: FromResourceBank + 'static,
{
    type Target<'rb> = Res<'rb, R>;

    fn from_resource_bank(resource_bank: &ResourceBank) -> Self::Target<'_> {
        R::from_resource_bank(resource_bank)
    }

    fn dependency() -> ResourceDependency {
        ResourceDependency::Res(TypeId::of::<R>())
    }
}

impl<R> SystemParameter for ResMut<'_, R>
where
    R: FromResourceBank + 'static,
{
    type Target<'rb> = ResMut<'rb, R>;

    fn from_resource_bank(resource_bank: &ResourceBank) -> Self::Target<'_> {
        R::from_resource_bank_mut(resource_bank)
    }

    fn dependency() -> ResourceDependency {
        ResourceDependency::ResMut(TypeId::of::<R>())
    }
}

pub(crate) type BoxedSystem = Box<dyn System>;

pub(crate) trait System: 'static + Send + Sync {
    fn run(&mut self, resource_bank: &ResourceBank);
    fn name(&self) -> &'static str;
    fn dependencies(&self) -> Vec<ResourceDependency>;
}

pub trait SystemFunctionHandler<M>: 'static + Send + Sync {
    fn handle(&mut self, resource_bank: &ResourceBank);
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
    fn dependencies() -> Vec<ResourceDependency>;
}

pub(crate) struct SystemFunction<M: Send + Sync, F: SystemFunctionHandler<M> + Send + Sync> {
    f: F,
    _marker: std::marker::PhantomData<fn(M) -> ()>,
}

impl<M: Send + Sync, F: SystemFunctionHandler<M>> SystemFunction<M, F> {
    fn new(f: F) -> Self {
        Self {
            f,
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn new_boxed(f: F) -> Box<Self> {
        Box::new(Self::new(f))
    }
}

impl<M: 'static + Send + Sync, F: SystemFunctionHandler<M>> System for SystemFunction<M, F> {
    fn run(&mut self, resource_bank: &ResourceBank) {
        self.f.handle(resource_bank);
    }

    fn name(&self) -> &'static str {
        F::name()
    }
    fn dependencies(&self) -> Vec<ResourceDependency> {
        F::dependencies()
    }
}

macro_rules! impl_system_function_handler {
    ($($param:ident),*) => {
        impl<F, $($param: SystemParameter),*> SystemFunctionHandler<fn($($param),*) -> ()> for F
        where
            F: FnMut($($param),*) + FnMut($(SystemParameterTarget<$param>),*) + 'static + Send + Sync,
        {
            fn handle(&mut self, _resource_bank: &ResourceBank) {
                // Function needs to be generified again since rust can't infer the type correctly.
                fn call<F, $($param),*>(mut f: F, $($param: $param),*)
                where
                    F: FnMut($($param),*),
                {
                    (f)($($param),*);
                }
                call(self, $($param::from_resource_bank(_resource_bank)),*);
            }

            fn dependencies() -> Vec<ResourceDependency> {
                vec![$($param::dependency()),*]
            }
        }
    };
}

generate_system_function_handlers!(impl_system_function_handler, 16);
