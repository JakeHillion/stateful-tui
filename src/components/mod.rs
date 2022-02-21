use log::{debug, info};

mod border;

pub use border::border;

use crate::Error;

use std::io::Write;
use std::ops::Range;

use crossterm::{cursor, style, QueueableCommand};

pub trait Drawable {
    fn draw(&self, t: &mut dyn Write, x: Range<u16>, y: Range<u16>) -> Result<(), Error>;
}

pub struct Fragment;

impl Drawable for Fragment {
    fn draw(&self, t: &mut dyn Write, x: Range<u16>, y: Range<u16>) -> Result<(), Error> {
        debug!("drew a fragment");

        for row in y {
            t.queue(cursor::MoveTo(x.start, row))?;
            t.queue(style::Print(" ".repeat((x.end - x.start) as usize)))?;
        }

        Ok(())
    }
}

pub struct Span(pub String);

impl Drawable for Span {
    fn draw(&self, t: &mut dyn Write, x: Range<u16>, y: Range<u16>) -> Result<(), Error> {
        debug!("drew a span(`{}`)", self.0);

        t.queue(cursor::MoveTo(x.start, y.start))?;
        t.queue(style::Print(self.0.as_str()))?;
        t.queue(cursor::MoveToNextLine(1))?;

        Ok(())
    }
}

pub struct Overlay;

impl Drawable for Overlay {
    fn draw(&self, _t: &mut dyn Write, _x: Range<u16>, _y: Range<u16>) -> Result<(), Error> {
        info!("todo: overlay");
        Ok(())
    }
}
