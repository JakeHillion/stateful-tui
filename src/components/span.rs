use log::debug;

use super::Drawable;
use crate::{Context, Error};

use std::io::Write;
use std::ops::Range;

use crossterm::{cursor, style, QueueableCommand};

type SpanProps = String;

pub fn span(_c: &mut Context<SpanProps>, content: &SpanProps) -> Box<dyn Drawable> {
    Box::new(DrawSpan {
        content: content.to_string(),
    })
}

struct DrawSpan {
    content: String,
}

impl Drawable for DrawSpan {
    fn draw(&self, t: &mut dyn Write, x: Range<u16>, y: Range<u16>) -> Result<(), Error> {
        debug!("drew a span(`{}`)", self.content);

        t.queue(cursor::MoveTo(x.start, y.start))?;
        t.queue(style::Print(self.content.as_str()))?;
        t.queue(cursor::MoveToNextLine(1))?;

        Ok(())
    }
}
