//! `wipe_all` wipes the holders still alive, and only them.
//! In its own test binary, as it wipes every holder of the process.

use melodium_common::executive::{register_wipe, wipe_all, wiped_count, Wipe};
use std::sync::{Arc, Mutex, Weak};

struct Holder(Mutex<Option<String>>);

impl Wipe for Holder {
    fn wipe(&self) -> bool {
        self.0.lock().unwrap().take().is_some()
    }
}

#[test]
fn alive_holders_are_wiped() {
    let alive = Arc::new(Holder(Mutex::new(Some("wipe-sentinel".to_string()))));
    let dropped = Arc::new(Holder(Mutex::new(Some("dropped".to_string()))));
    register_wipe(Arc::downgrade(&alive) as Weak<dyn Wipe>);
    register_wipe(Arc::downgrade(&dropped) as Weak<dyn Wipe>);
    drop(dropped);

    let count = wiped_count();
    assert_eq!(wipe_all(), 1);
    assert!(alive.0.lock().unwrap().is_none());
    // Holders count their own wipes, this one does not.
    assert_eq!(wiped_count(), count);
    // Registrations are consumed.
    assert_eq!(wipe_all(), 0);
}
