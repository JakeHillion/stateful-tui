mod crossterm;

pub use crossterm::CrosstermCanvas;

/**
 * Canvas to draw on to represent the TUI
 *
 * This is a trait, allowing for multiple potential backends. Canvases may be
 * split any number of times to segment them as you see fit.
 */
pub trait Canvas {
    /**
     * Get the drawable width of the canvas
     */
    fn width(&self) -> usize;

    /**
     * Get the drawable height of the canvas
     */
    fn height(&self) -> usize;

    /**
     * Split the canvas into two parts widthwise
     *
     * Returns the left and right split halves. x should be less than the width
     * of the canvas.
     *
     */
    fn split_width(&self, x: usize) -> (Box<dyn Canvas>, Box<dyn Canvas>);

    /**
     * Split the canvas into two parts heightwise
     *
     * Returns the top and bottom split halves. y should be less than the height
     * of the canvas.
     *
     */
    fn split_height(&self, y: usize) -> (Box<dyn Canvas>, Box<dyn Canvas>);

    /**
     * Shrink the canvas in any or all of the four directions
     *
     * Returns the newly shrunk canvas.
     */
    fn shrink(&self, top: usize, bottom: usize, left: usize, right: usize) -> Box<dyn Canvas>;
}
