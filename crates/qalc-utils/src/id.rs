use lazy_static::lazy_static;
use std::sync::atomic::{AtomicUsize, Ordering};

lazy_static! {
  static ref ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

pub fn gen_id() -> usize {
  ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}
