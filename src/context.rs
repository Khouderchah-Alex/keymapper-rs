use lazy_static::lazy_static;

use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct Context {
    pub title: String,
}

impl Context {
    pub fn current() -> Arc<Context> {
        CURRENT_CONTEXT.read().unwrap().clone()
    }
    pub fn make_current(self) {
        *CURRENT_CONTEXT.write().unwrap() = Arc::new(self);
    }
}

lazy_static! {
    static ref CURRENT_CONTEXT: RwLock<Arc<Context>> = Default::default();
}
