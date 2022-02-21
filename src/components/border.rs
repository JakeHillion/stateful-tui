use log::debug;

use super::Drawable;
use crate::{Context, Error};

use std::io::Write;
use std::ops::Range;

use crossterm::{cursor, style, QueueableCommand};

type BorderProps = (bool, bool, bool, bool);

pub fn border(_c: &mut Context<BorderProps>, props: &BorderProps) -> Box<dyn Drawable> {
    let (top, bottom, left, right) = props;

    Box::new(DrawBorder {
        top: *top,
        bottom: *bottom,
        left: *left,
        right: *right,
    })
}

struct DrawBorder {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Drawable for DrawBorder {
    fn draw(&self, t: &mut dyn Write, x: Range<u16>, y: Range<u16>) -> Result<(), Error> {
        debug!("drew a border");

        for row in y.clone() {
            t.queue(cursor::MoveTo(x.start, row))?;
            if row == y.start && self.top || row == y.end - 1 && self.bottom {
                t.queue(style::Print("-".repeat((x.end - x.start) as usize)))?;
            } else {
                if self.left {
                    t.queue(style::Print("|"))?;
                }
                if self.right {
                    t.queue(cursor::MoveTo(x.end - 1, row))?;
                    t.queue(style::Print("|"))?;
                }
            }
        }

        let child_x =
            x.start + if self.left { 1 } else { 0 }..x.end - if self.right { 1 } else { 0 };

        let child_y =
            y.start + if self.top { 1 } else { 0 }..y.end - if self.bottom { 1 } else { 0 };

        // TODO: draw actual child
        super::Fragment.draw(t, child_x, child_y)?;

        Ok(())
    }
}
