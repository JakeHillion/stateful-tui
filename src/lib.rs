mod canvas;
mod context;

pub use canvas::Canvas;
pub use context::Context;

use std::fmt::Debug;
use std::hash::Hash;

/**
 * Marker trait implementation for all Props
 *
 * Props for a Component must satisfy these requirements. The most important is
 * PartialEq, to avoid redrawing a component when the Props are the same as
 * before.
 */
pub trait Props: PartialEq + Send + Sync + 'static {}
impl<T> Props for T where T: PartialEq + Send + Sync + 'static {}

/**
 * Marker trait implementation for all Bodies
 *
 * Body is a special case of props. This makes little difference unless using
 * the syntactic sugar, where the body is placed in the body of the component.
 */
pub trait Body: Props {}
impl<T> Body for T where T: Props {}

/**
 * Marker trait implementation for all Keys
 *
 * A Key references a child to avoid expensive redrawing when the child is the
 * same.
 */
pub trait Key: Hash + Eq + Clone + Debug + 'static {}
impl<T> Key for T where T: Hash + Eq + Clone + Debug + 'static {}

/**
 * Abstract objects which know how to be drawn to the terminal
 */
pub trait Drawable {
    fn draw(&self, canvas: &mut dyn Canvas);
}

/**
 * Trait implementation for all Components
 *
 * A component is a function that takes props and/or children and produces a
 * description of what should be drawn to the TUI.
 */
pub trait Component<P: Props, B: Body, K> {
    fn render(&self, c: &mut Context<P, B, K>, props: &P, children: &B) -> Box<dyn Drawable>;
}

/**
 * Component implementation for purely functional components
 *
 * Functions which directly implement the render method of components are
 * acceptable as components directly.
 */
impl<F, P: Props, B: Body, K> Component<P, B, K> for F
where
    F: Fn(&mut Context<P, B, K>, &P, &B) -> Box<dyn Drawable> + Send + Sync + 'static,
{
    fn render(&self, c: &mut Context<P, B, K>, props: &P, children: &B) -> Box<dyn Drawable> {
        self(c, props, children)
    }
}
