use super::Context;
use crate::{Component, Props};

use std::any::Any;
use std::sync::{Arc, Mutex};

pub struct ChildStore {
    index: usize,
    store: Vec<(ChildIdentifier, Arc<Mutex<dyn Any + Send + Sync>>)>,
}

#[derive(Debug, PartialEq)]
pub struct ChildIdentifier {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
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

    pub(super) fn add_child<P: Props>(
        &mut self,
        id: ChildIdentifier,
        c: Box<dyn Component<P>>,
        p: P,
    ) -> Arc<Mutex<Context<P>>> {
        let i = self.index;
        self.index += 1;

        todo!()
    }
}
