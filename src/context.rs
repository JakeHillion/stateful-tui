use log::{debug, trace, warn};

use crate::Event;
use crate::{Component, EffectArgs, Error, Props, State};

use std::any::Any;
use std::future::Future;
use std::io;
use std::ops::Range;
use std::sync::{mpsc::SyncSender, Arc, Mutex, Weak};

pub struct Context<T: Props> {
    me: Weak<Mutex<Context<T>>>,
    events: SyncSender<Event<T>>,

    component: Arc<dyn Component<T>>,

    props: PropsInternal<T>,
    last_drawn_props: Option<PropsInternal<T>>,

    state_index: usize,
    state: Vec<Arc<dyn Any + Send + Sync>>,
    last_drawn_state: Option<Vec<Arc<dyn Any + Send + Sync>>>,

    effect_index: usize,
    effect_args: Vec<Box<dyn Any + Send>>,
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
        events: SyncSender<Event<T>>,
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

                state_index: 0,
                state: Vec::new(),
                last_drawn_state: None,

                effect_index: 0,
                effect_args: Vec::new(),
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
        let i = self.state_index;
        self.state_index += 1;

        if self.state.len() < i + 1 {
            trace!("use_state hit for the first time");
            self.state.push(Arc::new(initial()));
        } else {
            trace!("use_state hit for the nth time")
        }

        let state = self
            .state
            .get(i)
            .expect("self.state.len() and self.state_index desynced");

        let ctx = self.me.clone();

        let set_state = Box::new(move |new_state: S| {
            let ctx = if let Some(ctx) = ctx.upgrade() {
                ctx
            } else {
                warn!("set_state call outlived component");
                return;
            };

            let mut ctx = ctx.lock().unwrap();

            let new_state = Arc::new(new_state);
            // grow the vector by one and add the new state to the back
            ctx.state.push(new_state.clone());
            // remove the ith element and replace it with the last element
            let last_state = ctx.state.swap_remove(i);

            if new_state.as_ref()
                != last_state
                    .as_ref()
                    .downcast_ref()
                    .expect("self.state.len() and self.state_index desynced")
            {
                if let Err(e) = ctx.events.send(Event::Redraw) {
                    warn!("attempted to send event to closed channel: {}", e);
                }
            }
        });

        (
            state
                .clone()
                .downcast()
                .expect("use_state called in same position with different type"),
            set_state,
        )
    }

    /// Add an effect to be triggered when certain values change
    pub fn use_effect<A, F>(&mut self, effect: Box<dyn FnOnce(&A) -> F>, args: A)
    where
        A: EffectArgs,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        let i = self.effect_index;
        self.effect_index += 1;

        let call_args = if self.effect_args.len() < i + 1 {
            trace!("use_effect hit for the first time");

            self.effect_args.push(Box::new(args));
            Some(&self.effect_args[i])
        } else {
            trace!("use_effect hit for the nth time");
            let last_args: &A = self
                .effect_args
                .get(i)
                .expect("self.effect_args.len() and self.effect_index desynced")
                .downcast_ref()
                .expect("self.effect_args.len() and self.effect_index desynced");

            if &args != last_args {
                self.effect_args[i] = Box::new(args);
                Some(&self.effect_args[i])
            } else {
                None
            }
        };

        if let Some(a) = call_args {
            debug!("use_effect chose to call");

            let fut = effect(
                a.downcast_ref()
                    .expect("self.effect_args.len() and self.effect_index desynced"),
            );

            if let Err(e) = self.events.send(Event::NewEffect(Box::pin(fut))) {
                warn!("attempted to send event to closed channel: {}", e);
            }
        }
    }

    pub fn draw(&mut self, terminal: &mut dyn io::Write) -> Result<(), Error> {
        self.state_index = 0;
        self.effect_index = 0;

        let component = &self.component.clone();
        let props = &self.props.props.clone();

        let drawable = component.render(self, props.as_ref());
        drawable.draw(
            terminal,
            self.props.x_range.clone(),
            self.props.y_range.clone(),
        )?;

        self.last_drawn_props = Some(self.props.clone());
        self.last_drawn_state = Some(self.state.clone());
        Ok(())
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
