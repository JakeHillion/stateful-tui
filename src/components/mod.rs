use log::{debug, info};

mod border;
mod fragment;
mod span;

pub use border::border;
pub use fragment::fragment;
pub use span::span;

use crate::{Component, Error};

use std::io::Write;
use std::ops::Range;

use crossterm::{cursor, style, QueueableCommand};

pub trait Drawable: Send + Sync + 'static {
    fn draw(
        &self,
        t: &mut dyn Write,
        // c: &mut dyn Iterator<Item = &dyn Drawable>,
        x: Range<u16>,
        y: Range<u16>,
    ) -> Result<(), Error>;
}
