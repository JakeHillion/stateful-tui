use super::{Body, Component, Drawable, Key, Props};

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/**
 * Context type which wraps drawn components and holds their state
 *
 * This type is provided to public component consumers with the `use_state` and
 * `use_effect` methods.
 */
pub struct Context<P: Props, B: Body, K> {
    component: Rc<dyn Component<P, B, K>>,

    props: Rc<P>,
    last_props: Option<Rc<P>>,

    body: Rc<B>,
    last_body: Option<Rc<B>>,

    children: HashMap<K, ChildEntry>,
    live_children: HashSet<K>,

    last_drawn: Option<Rc<dyn Drawable>>,
}

impl<P: Props, B: Body, K: Key> Context<P, B, K> {
    pub(crate) fn new(
        component: Box<dyn Component<P, B, K>>,
        initial_props: P,
        initial_body: B,
    ) -> Self {
        Self {
            component: component.into(),

            props: initial_props.into(),
            last_props: None,

            body: initial_body.into(),
            last_body: None,

            children: HashMap::new(),
            live_children: HashSet::new(),

            last_drawn: None,
        }
    }

    /**
     * Apply a child component to this component
     *
     * This creates a child and the appropriate state to avoid rendering it
     * unless necessary. Key must always be unique. Children not visible in a
     * render pass will have their state dropped.
     */
    pub fn with_child<P1: Props, B1: Body, K1: Key>(
        &mut self,
        key: K,
        component: Box<dyn Component<P1, B1, K1>>,
        props: P1,
        body: B1,
    ) -> Rc<dyn Drawable> {
        self.live_children.insert(key.clone());

        let mut entry = self.children.entry(key);

        let child_ctx = if let Entry::Vacant(entry) = entry {
            let ctx = Context::new(component, props, body);
            let child_entry = entry.insert(ChildEntry::new(ctx));

            child_entry
                .get_context_mut::<P1, B1, K1>()
                .expect("child components with the same key must be the same type")
        } else if let Entry::Occupied(entry) = &mut entry {
            let ctx = entry
                .get_mut()
                .get_context_mut::<P1, B1, K1>()
                .expect("child components with the same key must be the same type");

            ctx.update_props(props);
            ctx.update_body(body);

            ctx
        } else {
            unreachable!("the entry was eiter full or not")
        };

        child_ctx.render().clone()
    }

    pub fn render(&mut self) -> &Rc<dyn Drawable> {
        if self.redraw_required() {
            self.last_drawn = None;
        }

        if self.last_drawn.is_none() {
            self.last_drawn = Some(self.draw().into());
        }

        if let Some(d) = &self.last_drawn {
            d.into()
        } else {
            unreachable!("last_drawn has content")
        }
    }

    fn draw(&mut self) -> Box<dyn Drawable> {
        // reset state monitors
        self.live_children.clear();

        // clone to avoid borrowing during the render
        let component = self.component.clone();
        let props = self.props.clone();
        let body = self.body.clone();

        // render
        let ret = component.render(self, props.as_ref(), body.as_ref());

        // clean up old state
        self.children.retain(|k, _| self.live_children.contains(k));

        return ret;
    }

    fn redraw_required(&self) -> bool {
        if let Some(last_props) = &self.last_props {
            if &self.props != last_props {
                return true;
            }
        } else {
            return true;
        };

        if let Some(last_body) = &self.last_body {
            if &self.body != last_body {
                return true;
            }
        } else {
            return true;
        };

        false
    }

    fn update_props(&mut self, props: P) {
        self.props = props.into();
    }

    fn update_body(&mut self, body: B) {
        self.body = body.into();
    }
}

/**
 * ChildEntry stores a child context within a context
 *
 * Dynamic typing is used because the children may have any manner of props,
 * meaning that the types (generics) of each component are different. We must
 * instead establish that this is correct at runtime.
 */
struct ChildEntry {
    context: Box<dyn std::any::Any>,
}

impl ChildEntry {
    fn new<P: Props, B: Body, K: Key>(context: Context<P, B, K>) -> Self {
        Self {
            context: Box::new(context),
        }
    }

    fn get_context_mut<P: Props, B: Body, K: Key>(&mut self) -> Option<&mut Context<P, B, K>> {
        self.context.downcast_mut()
    }
}
