use log::trace;

use super::Context;
use crate::{Component, Event, Props};

use std::any::Any;
use std::sync::{mpsc::SyncSender, Arc, Mutex};

#[derive(Debug, PartialEq, Clone)]
pub struct ChildIdentifier {
    pub line: u32,
    pub column: u32,
}

pub struct ChildStore {
    index: usize,
    store: Vec<(ChildIdentifier, Arc<dyn Any + Send + Sync>)>,
}

impl ChildStore {
    pub(super) fn new() -> ChildStore {
        ChildStore {
            index: 0,
            store: Vec::new(),
        }
    }

    pub(super) fn reset_index(&mut self) {
        self.index = 0
    }

    /// Drop the remaining children that weren't seen this render
    pub(super) fn drawn(&mut self) {
        self.store.truncate(self.index + 1)
    }

    pub(super) fn add_child<P: Props>(
        &mut self,
        events: &SyncSender<Event>,
        id: ChildIdentifier,
        c: Box<dyn Component<P>>,
        p: P,
    ) -> Arc<Mutex<Context<P>>> {
        self.update_or_insert(events, id, c, p)
    }

    /// Insert a new child, reordering the store appropriately
    fn update_or_insert<P: Props>(
        &mut self,
        ev: &SyncSender<Event>,
        id: ChildIdentifier,
        c: Box<dyn Component<P>>,
        p: P,
    ) -> Arc<Mutex<Context<P>>> {
        let index = self.index;
        self.index += 1;

        // case 1: first time with this many components
        if self.store.len() < index + 1 {
            trace!("new child component added last");
            // on the first setup for this component, all children will be added as new
            let ctx = Context::new(c, p, ev.clone());
            self.store.push((id, ctx.clone()));
            return ctx;
        } else {
            trace!("new child component not last")
        };

        // case 2: already there out of order
        let found = self.store.as_slice()[index..]
            .iter()
            .enumerate()
            .find(|(_, (existing_id, _))| existing_id == &id)
            .map(|(i, (_, ctx))| (i, ctx.clone()));

        if let Some((i, ctx)) = found {
            trace!(
                "child component already exists, moving from {} to {}",
                i,
                index
            );
            self.store.swap(index, i);
            return ctx
                .downcast::<Mutex<Context<P>>>()
                .expect("child type desynced");
        }

        // case 3: new child
        trace!("new child component is new and not last, swapping in");
        let ctx = Context::new(c, p, ev.clone());

        self.store.push((id, ctx.clone()));
        let pushed = self.store.len();
        self.store.swap(index, pushed);

        ctx
    }
}
