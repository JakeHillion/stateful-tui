use log::{trace, warn};

use super::Context;
use crate::{Event, Props, State};

use std::any::Any;
use std::sync::{Arc, Mutex, Weak};

pub struct StateStore {
    index: usize,
    state: Vec<Arc<dyn Any + Send + Sync>>,
    last_drawn_state: Option<Vec<Arc<dyn Any + Send + Sync>>>,
}

impl StateStore {
    pub(super) fn new() -> StateStore {
        StateStore {
            index: 0,
            state: Vec::new(),
            last_drawn_state: None,
        }
    }

    pub(super) fn reset_index(&mut self) {
        self.index = 0
    }

    pub(super) fn drawn(&mut self) {
        self.last_drawn_state = Some(self.state.clone())
    }

    pub(super) fn use_state<P: Props, S, F>(
        &mut self,
        ctx: Weak<Mutex<Context<P>>>,
        initial: F,
    ) -> (Arc<S>, Box<dyn Fn(S) + Send + Sync + 'static>)
    where
        S: State,
        F: FnOnce() -> S,
    {
        let i = self.index;
        self.index += 1;

        if self.state.len() < i + 1 {
            trace!("use_state hit for the first time");
            self.state.push(Arc::new(initial()));
        } else {
            trace!("use_state hit for the nth time")
        }

        let state = self
            .state
            .get(i)
            .expect("self.state.len() and self.index desynced");

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
            ctx.state.state.push(new_state.clone());
            // remove the ith element and replace it with the last element
            let last_state = ctx.state.state.swap_remove(i);

            if new_state.as_ref()
                != last_state
                    .as_ref()
                    .downcast_ref()
                    .expect("self.state.len() and self.index desynced")
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
}
