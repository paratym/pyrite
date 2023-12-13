use pyrite_app_macros::generate_system_function_handlers;
use std::any::TypeId;

use crate::resource::{FromResourceBank, Res, ResMut, ResourceBank};

#[derive(Debug)]
pub enum ResourceDependency {
    Res(TypeId),
    ResMut(TypeId),
}

type SystemParamItem<'rb, P> = <P as SystemParam>::Item<'rb>;

pub trait SystemParam {
    type Item<'rb>: SystemParam;

    fn from_resource_bank(resource_bank: &ResourceBank) -> Self::Item<'_>;

    /// Used to validate that the system isn't using the same resource twice.
    fn dependency() -> ResourceDependency;

    // TODO: fn scheduling_dependencies() -> Vec<ScheduingDependencyType>;
}

// Generic system param over any generic resource from the resource bank.
impl<R> SystemParam for Res<'_, R>
where
    R: FromResourceBank + 'static,
{
    type Item<'rb> = Res<'rb, R>;

    fn from_resource_bank(resource_bank: &ResourceBank) -> Self::Item<'_> {
        R::from_resource_bank(resource_bank)
    }

    fn dependency() -> ResourceDependency {
        ResourceDependency::Res(TypeId::of::<R>())
    }
}

impl<R> SystemParam for ResMut<'_, R>
where
    R: FromResourceBank + 'static,
{
    type Item<'rb> = ResMut<'rb, R>;

    fn from_resource_bank(resource_bank: &ResourceBank) -> Self::Item<'_> {
        R::from_resource_bank_mut(resource_bank)
    }

    fn dependency() -> ResourceDependency {
        ResourceDependency::ResMut(TypeId::of::<R>())
    }
}

pub type BoxedSystem = Box<dyn System>;

pub trait System: Send {
    fn run(&mut self, resource_bank: &ResourceBank);
    fn name(&self) -> &'static str;
    fn dependencies(&self) -> Vec<ResourceDependency>;
}

pub trait SystemFunctionHandler<M>: Send {
    fn handle(&mut self, resource_bank: &ResourceBank);
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
    fn dependencies() -> Vec<ResourceDependency>;
}

pub struct SystemFunction<M, F: SystemFunctionHandler<M>> {
    f: F,
    _marker: std::marker::PhantomData<fn(M) -> ()>,
}

impl<M, F: SystemFunctionHandler<M>> SystemFunction<M, F> {
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

impl<M, F: SystemFunctionHandler<M>> System for SystemFunction<M, F> {
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
        impl<F, $($param: SystemParam),*> SystemFunctionHandler<fn($($param),*) -> ()> for F
        where
            F: FnMut($($param),*) + FnMut($(SystemParamItem<$param>),*) + Send,
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
