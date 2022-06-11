use super::Canvas;

use std::io::Write;
use std::rc::Rc;

/**
 * A whole TUI canvas using crossterm as the backend
 *
 * This canvas has a fixed size for the duration of the render.
 */
pub struct CrosstermCanvas {
    terminal: Rc<dyn Write>,

    x_min: usize,
    x_max: usize,

    y_min: usize,
    y_max: usize,
}

impl CrosstermCanvas {
    fn new() -> Self {
        todo!()
    }
}

impl Canvas for CrosstermCanvas {
    fn width(&self) -> usize {
        self.x_max - self.x_min
    }

    fn height(&self) -> usize {
        self.y_max - self.y_min
    }

    fn split_width(&self, x: usize) -> (Box<dyn Canvas>, Box<dyn Canvas>) {
        let x = x.min(self.width());

        (
            Box::new(CrosstermCanvas {
                terminal: self.terminal.clone(),

                x_min: self.x_min,
                x_max: x,

                y_min: self.y_min,
                y_max: self.y_max,
            }),
            Box::new(CrosstermCanvas {
                terminal: self.terminal.clone(),

                x_min: x,
                x_max: self.x_max,

                y_min: self.y_min,
                y_max: self.y_max,
            }),
        )
    }

    fn split_height(&self, y: usize) -> (Box<dyn Canvas>, Box<dyn Canvas>) {
        let y = y.min(self.height());

        (
            Box::new(CrosstermCanvas {
                terminal: self.terminal.clone(),

                x_min: self.x_min,
                x_max: self.x_max,

                y_min: self.y_min,
                y_max: y,
            }),
            Box::new(CrosstermCanvas {
                terminal: self.terminal.clone(),

                x_min: self.x_min,
                x_max: self.x_max,

                y_min: y,
                y_max: self.y_max,
            }),
        )
    }

    fn shrink(&self, top: usize, bottom: usize, left: usize, right: usize) -> Box<dyn Canvas> {
        let x_min = (self.x_min + left).max(self.x_max);
        let x_max = (self.x_max - right).min(x_min);

        let y_min = (self.y_min + top).max(self.y_max);
        let y_max = (self.y_max - bottom).min(y_min);

        Box::new(CrosstermCanvas {
            terminal: self.terminal.clone(),

            x_min,
            x_max,

            y_min,
            y_max,
        })
    }
}
