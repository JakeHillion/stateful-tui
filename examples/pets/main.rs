use log::info;

use stateful_tui::{add_child, components, Context, Drawable, Tui};

fn main() {
    env_logger::init();

    Tui::spawn_with_props(Box::new(my_pets_tui), ()).unwrap()
}

fn my_pets_tui(c: &mut Context<()>, _: &()) -> Box<dyn Drawable> {
    let (count, set_count) = c.use_state(|| 0);
    c.use_effect(
        Box::new(|_| async move {
            info!("setting count to 104");
            set_count(104)
        }),
        count,
    );

    let border = add_child!(c, Box::new(components::border), (true, true, true, true));
    let mut border = border.lock().unwrap();

    add_child!(
        border,
        Box::new(components::span),
        "nested span".to_string()
    );

    border.draw()

    // <border props=(true, true, true, true)>
    //   <span props="nested_span".to_string() />
    // </border>
}
