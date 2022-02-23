use log::debug;

use super::Drawable;
use crate::{Context, Error};

use std::io::Write;
use std::ops::Range;

use crossterm::{cursor, style, QueueableCommand};

#[derive(PartialEq)]
pub struct FragmentProps {
    // children: Vec<Box<dyn Drawable>>,
}

pub fn fragment(_c: &mut Context<FragmentProps>, _props: &FragmentProps) -> Box<dyn Drawable> {
    Box::new(DrawFragment)
}

pub struct DrawFragment;

impl Drawable for DrawFragment {
    fn draw(
        &self,
        t: &mut dyn Write,
        // c: &mut dyn Iterator<Item = &dyn Drawable>,
        x: Range<u16>,
        y: Range<u16>,
    ) -> Result<(), Error> {
        debug!("drew a fragment");

        for row in y {
            t.queue(cursor::MoveTo(x.start, row))?;
            t.queue(style::Print(" ".repeat((x.end - x.start) as usize)))?;
        }
        // for child in c {
        //     child.draw(t, None, x.clone(), y.clone())?;
        // }

        Ok(())
    }
}
