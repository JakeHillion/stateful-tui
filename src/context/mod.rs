use log::{debug, warn};

mod children;
mod effects;
mod state;

pub use children::ChildIdentifier;
use children::ChildStore;
use effects::EffectStore;
use state::StateStore;

use crate::Event;
use crate::{Component, Drawable, EffectArgs, Error, Props, State};

use std::future::Future;
use std::io;
use std::ops::Range;
use std::sync::{mpsc::SyncSender, Arc, Mutex, Weak};

pub struct Context<T: Props> {
    me: Weak<Mutex<Context<T>>>,
    events: SyncSender<Event>,

    component: Arc<dyn Component<T>>,

    props: PropsInternal<T>,
    last_drawn_props: Option<PropsInternal<T>>,

    state: StateStore,
    effects: EffectStore,
    children: ChildStore,
}

#[derive(PartialEq)]
struct PropsInternal<T: Props> {
    x_range: Range<u16>,
    y_range: Range<u16>,

    props: Arc<T>,
}

impl<T: Props> Clone for PropsInternal<T> {
    fn clone(&self) -> Self {
        PropsInternal {
            x_range: self.x_range.clone(),
            y_range: self.y_range.clone(),
            props: self.props.clone(),
        }
    }
}

impl<T: Props> Context<T> {
    /// Create a new context with the initial props
    pub fn new(
        root: Box<dyn Component<T>>,
        props: T,
        events: SyncSender<Event>,
    ) -> Arc<Mutex<Context<T>>> {
        Arc::new_cyclic(|me| {
            Mutex::new(Context {
                me: me.clone(),
                events,

                component: Arc::from(root),

                props: PropsInternal {
                    x_range: (0..0),
                    y_range: (0..0),
                    props: Arc::new(props),
                },
                last_drawn_props: None,

                state: StateStore::new(),
                effects: EffectStore::new(),
                children: ChildStore::new(),
            })
        })
    }

    /// Update the props of the root component externally
    pub fn update_props(&mut self, new_props: T) {
        self.props.props = Arc::new(new_props);

        if self.props_are_changed() {
            if let Err(e) = self.events.send(Event::Redraw) {
                warn!("attempted to send event to closed channel: {}", e);
            }
        }
    }

    /// Update the size of the component
    pub fn update_size(&mut self, x_range: Range<u16>, y_range: Range<u16>) {
        debug!("component resized: ({:?}, {:?})", &x_range, &y_range);

        self.props.x_range = x_range;
        self.props.y_range = y_range;

        if self.size_is_changed() {
            if let Err(e) = self.events.send(Event::Redraw) {
                warn!("attempted to send event to closed channel: {}", e);
            }
        }
    }

    /// Add a piece of state to this component
    pub fn use_state<S, F>(
        &mut self,
        initial: F,
    ) -> (Arc<S>, Box<dyn Fn(S) + Send + Sync + 'static>)
    where
        S: State,
        F: FnOnce() -> S,
    {
        self.state.use_state(self.me.clone(), initial)
    }

    /// Add an effect to be triggered when certain values change
    pub fn use_effect<A, F>(&mut self, effect: Box<dyn FnOnce(&A) -> F>, args: A)
    where
        A: EffectArgs,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.effects.use_effect(&mut self.events, effect, args)
    }

    /// Add a child component under this one
    pub fn add_child<P: Props>(
        &mut self,
        id: ChildIdentifier,
        c: Box<dyn Component<P>>,
        p: P,
    ) -> Arc<Mutex<Context<P>>> {
        debug!("adding child: {:?}", &id);

        self.children.add_child(&self.events, id, c, p)
    }

    pub fn draw(&mut self) -> Box<dyn Drawable> {
        // reset indices
        self.state.reset_index();
        self.effects.reset_index();
        self.children.reset_index();

        // clone references
        let component = &self.component.clone();
        let props = &self.props.props.clone();

        // draw
        let drawable = component.render(self, props.as_ref());

        // store props and state for comparison
        self.last_drawn_props = Some(self.props.clone());
        self.state.drawn();
        self.children.drawn();

        // return drawable
        drawable
    }

    pub fn render(&mut self, terminal: &mut dyn io::Write) -> Result<(), Error> {
        let drawable = self.draw();

        drawable.draw(
            terminal,
            self.props.x_range.clone(),
            self.props.y_range.clone(),
        )
    }

    fn props_are_changed(&self) -> bool {
        if let Some(last_props) = &self.last_drawn_props {
            self.props.props != last_props.props
        } else {
            true
        }
    }

    fn size_is_changed(&self) -> bool {
        if let Some(last_props) = &self.last_drawn_props {
            self.props.x_range != last_props.x_range || self.props.y_range != last_props.y_range
        } else {
            true
        }
    }
}

#[macro_export]
macro_rules! add_child {
    ($ctx:ident, $component:expr, $props:expr) => {
        ($ctx).add_child(
            $crate::context::ChildIdentifier {
                line: std::line!(),
                column: std::column!(),
            },
            $component,
            $props,
        )
    };
}
