use log::info;

use stateful_tui::{draw, Context, Drawable, Tui};

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
        count.clone(),
    );

    let (stateful_str, _) = c.use_state(|| "hello world!");

    Box::new(draw::Span(format!("{} count is: {}", stateful_str, count)))
}
