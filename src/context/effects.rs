use log::{debug, trace, warn};

use crate::{EffectArgs, Event};

use std::any::Any;
use std::sync::mpsc::SyncSender;

use futures::Future;

pub struct EffectStore {
    index: usize,
    last_args: Vec<Box<dyn Any + Send>>,
}

impl EffectStore {
    pub(super) fn new() -> EffectStore {
        EffectStore {
            index: 0,
            last_args: Vec::new(),
        }
    }

    pub(super) fn reset_index(&mut self) {
        self.index = 0
    }

    pub(super) fn use_effect<A, F>(
        &mut self,
        tx: &mut SyncSender<Event>,
        effect: Box<dyn FnOnce(&A) -> F>,
        args: A,
    ) where
        A: EffectArgs,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        let index = self.index;
        self.index += 1;

        let call_args = if self.last_args.len() < index + 1 {
            trace!("use_effect hit for the first time");

            self.last_args.push(Box::new(args));
            Some(&self.last_args[index])
        } else {
            trace!("use_effect hit for the nth time");
            let last_args: &A = self
                .last_args
                .get(index)
                .expect("self.effect_args.len() and self.effect_index desynced")
                .downcast_ref()
                .expect("self.effect_args.len() and self.effect_index desynced");

            if &args != last_args {
                self.last_args[index] = Box::new(args);
                Some(&self.last_args[index])
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

            if let Err(e) = tx.send(Event::NewEffect(Box::pin(fut))) {
                warn!("attempted to send event to closed channel: {}", e);
            }
        }
    }
}
