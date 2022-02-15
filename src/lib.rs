mod context;
pub mod draw;
mod error;
mod tui;

pub use context::Context;
pub use draw::Drawable;
pub use error::Error;
pub use tui::{Event, Tui};

use std::any::Any;

/// The type of a component
pub trait Component<T: Props>: Send + Sync + 'static {
    fn render(&self, c: &mut Context<T>, props: &T) -> Box<dyn Drawable>;
}

impl<F, T: Props> Component<T> for F
where
    F: Fn(&mut Context<T>, &T) -> Box<dyn Drawable> + Send + Sync + 'static,
{
    fn render(&self, c: &mut Context<T>, props: &T) -> Box<dyn Drawable> {
        self(c, props)
    }
}

pub trait Props: PartialEq + Send + Sync + 'static {}
impl<T> Props for T where T: PartialEq + Send + Sync + 'static {}

pub trait State: Any + PartialEq + Send + Sync + 'static {}
impl<T> State for T where T: Any + PartialEq + Send + Sync + 'static {}

pub trait EffectArgs: Any + PartialEq + Send + 'static {}
impl<T> EffectArgs for T where T: Any + PartialEq + Send + 'static {}
