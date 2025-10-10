use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fluxum::store::{
    DerivedStore, EmittingStore, Store, StoreCallback, StoreEffects, StoreKey, StoreRuntime, StoreRuntimeImpl,
    SubscriptionKey, mk_eq_fn,
};

fn with_quiet_panic<F, R>(f: F) -> std::thread::Result<R>
where
    F: FnOnce() -> R,
{
    // Temporarily silence the panic hook to avoid noisy output
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    // Restore previous hook
    let _ = std::panic::take_hook();
    std::panic::set_hook(prev);
    result
}

fn rc_counter() -> (Rc<RefCell<usize>>, StoreCallback) {
    let counter = Rc::new(RefCell::new(0usize));
    let counter_cb = counter.clone();
    let cb: StoreCallback = Rc::new(move |_store: StoreKey, _sub, _rt: &mut StoreEffects| {
        *counter_cb.borrow_mut() += 1;
    });
    (counter, cb)
}

#[test]
fn emitting_store_notifies_on_change_and_dedups_within_generation() {
    let mut rt = StoreRuntimeImpl::new();

    // i32 store with equality check so identical values don't notify
    let store = EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()));
    let key = rt.alloc_store(Box::new(store));

    let (counter, cb) = rc_counter();
    let _sub = rt.subscribe(key, cb);

    // Change once → one notification
    rt.set_value(key, Box::new(1i32));
    rt.drain_notifications();
    assert_eq!(*counter.borrow(), 1);

    // Set equal value → no notification
    rt.set_value(key, Box::new(1i32));
    rt.drain_notifications();
    assert_eq!(*counter.borrow(), 1);

    // Two changes in same generation before draining → still one notification
    rt.set_value(key, Box::new(2i32));
    rt.set_value(key, Box::new(3i32));
    rt.drain_notifications();
    assert_eq!(*counter.borrow(), 2);
}

#[test]
fn subscribe_enqueues_if_changed_in_current_generation() {
    let mut rt = StoreRuntimeImpl::new();

    let store = EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()));
    let key = rt.alloc_store(Box::new(store));

    // Change value before subscribing (gen 0)
    rt.set_value(key, Box::new(1i32));

    let (counter, cb) = rc_counter();
    let _sub = rt.subscribe(key, cb);

    // Subscription should be immediately enqueued since store changed this generation
    rt.drain_notifications();
    assert_eq!(*counter.borrow(), 1);
}

#[test]
fn unsubscribe_prevents_further_notifications() {
    let mut rt = StoreRuntimeImpl::new();

    let store = EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()));
    let key = rt.alloc_store(Box::new(store));

    let (counter, cb) = rc_counter();
    let sub = rt.subscribe(key, cb);

    // First change → one notification
    rt.set_value(key, Box::new(1i32));
    rt.drain_notifications();
    assert_eq!(*counter.borrow(), 1);

    // Unsubscribe and change again → no new notifications
    let removed = rt.unsubscribe(sub);
    assert!(removed);

    rt.set_value(key, Box::new(2i32));
    rt.drain_notifications();
    assert_eq!(*counter.borrow(), 1);
}

#[test]
fn free_store_cleans_subscriptions() {
    let mut rt = StoreRuntimeImpl::new();

    let store = EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()));
    let key = rt.alloc_store(Box::new(store));

    // Create a subscription and ensure freeing the store removes it
    let (_counter, cb) = rc_counter();
    let sub = rt.subscribe(key, cb);

    // Free the store; internal cleanup removes subscribers
    rt.free_store(key);

    // Unsubscribe should now return false (already cleaned up)
    let removed = rt.unsubscribe(sub);
    assert!(!removed);
}

#[test]
fn const_erased_panics_on_set_value() {
    // We can't construct ConstErased directly due to its private field,
    // so emulate the behavior with a local Store that panics on set_any.
    struct PanicWriteStore {
        value: Box<dyn std::any::Any>,
    }

    impl Store for PanicWriteStore {
        fn get_any(&self) -> &dyn std::any::Any { &*self.value }
        fn set_any(&mut self, _: Box<dyn std::any::Any>, _: &mut fluxum::store::SubSink) {
            panic!("attempt to write const store, this is a framework error (or you've been naughty)")
        }
        fn subscribe(&mut self, _: SubscriptionKey, _: u64) -> bool { false }
        fn unsubscribe(&mut self, _: SubscriptionKey) {}
        fn subscriptions(&self) -> Option<&[SubscriptionKey]> { None }
        fn dependencies(&self) -> Option<&[SubscriptionKey]> { None }
    }

    let mut rt = StoreRuntimeImpl::new();
    let key = rt.alloc_store(Box::new(PanicWriteStore { value: Box::new(0i32) }));
    // Attempting to set should panic; catch it and silence output
    let result = with_quiet_panic(|| {
        rt.set_value(key, Box::new(100i32));
    });
    assert!(result.is_err());
}

#[test]
fn derived_store_reacts_to_dependency() {
    let mut rt = StoreRuntimeImpl::new();

    // Base store
    let base = EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()));
    let base_key = rt.alloc_store(Box::new(base));

    // Prepare a place to store the derived store key accessible to callback
    let derived_key_cell: Rc<Cell<Option<StoreKey>>> = Rc::new(Cell::new(None));
    let cell_for_cb = derived_key_cell.clone();

    // Callback to run when base changes: set derived to a sentinel value
    let derive_cb: StoreCallback = Rc::new(move |_store, _sub, rt: &mut StoreEffects| {
        if let Some(k) = cell_for_cb.get() {
            rt.set_value(k, Box::new(999i32));
        }
    });

    // Create the derived store, subscribing to base
    let derived = DerivedStore::new(
        Box::new(0i32),
        Some(mk_eq_fn::<i32>()),
        derive_cb.clone(),
        [base_key],
        &mut rt,
    );

    // Allocate derived and publish its key to the callback
    let derived_key = rt.alloc_store(Box::new(derived));
    derived_key_cell.set(Some(derived_key));

    // Subscribe to derived to count notifications
    let (derived_counter, derived_notify_cb) = rc_counter();
    let _derived_sub = rt.subscribe(derived_key, derived_notify_cb);

    // Change base twice before draining; derived should compute once and notify once
    rt.set_value(base_key, Box::new(1i32));
    rt.set_value(base_key, Box::new(2i32));

    rt.drain_notifications();

    assert_eq!(*derived_counter.borrow(), 1);
}

#[test]
fn freeing_derived_cleans_dependency_subscriptions() {
    let mut rt = StoreRuntimeImpl::new();

    // Base store
    let base = EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()));
    let base_key = rt.alloc_store(Box::new(base));

    // Count how many times the derived's dependency callback fires
    let dep_counter = Rc::new(RefCell::new(0usize));
    let dep_counter_cb = dep_counter.clone();
    let dep_cb: StoreCallback = Rc::new(move |_store, _sub, _rt: &mut StoreEffects| {
        *dep_counter_cb.borrow_mut() += 1;
    });

    let derived = DerivedStore::new(
        Box::new(0i32),
        Some(mk_eq_fn::<i32>()),
        dep_cb.clone(),
        [base_key],
        &mut rt,
    );
    let derived_key = rt.alloc_store(Box::new(derived));

    // Change base: callback fires
    rt.set_value(base_key, Box::new(1i32));
    rt.drain_notifications();
    assert_eq!(*dep_counter.borrow(), 1);

    // Free derived; dependency subscription should be removed
    rt.free_store(derived_key);

    // Change base again: callback must not fire anymore
    rt.set_value(base_key, Box::new(2i32));
    rt.drain_notifications();
    assert_eq!(*dep_counter.borrow(), 1);
}

#[test]
fn set_value_on_freed_store_panics() {
    let mut rt = StoreRuntimeImpl::new();
    let key = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));
    rt.free_store(key);
    // Using a freed key should panic; catch and silence output
    let result = with_quiet_panic(|| {
        rt.set_value(key, Box::new(1i32));
    });
    assert!(result.is_err());
}

#[test]
fn subscribe_on_nonexistent_store_panics() {
    let mut rt = StoreRuntimeImpl::new();
    let key = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));
    rt.free_store(key);
    let (_c, cb) = rc_counter();
    // Subscribing to a freed key should panic; catch and silence output
    let result = with_quiet_panic(|| {
        let _ = rt.subscribe(key, cb);
    });
    assert!(result.is_err());
}

#[test]
fn unsubscribe_twice_returns_false_second_time() {
    let mut rt = StoreRuntimeImpl::new();
    let key = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));
    let (_c, cb) = rc_counter();
    let sub = rt.subscribe(key, cb);
    assert!(rt.unsubscribe(sub));
    assert!(!rt.unsubscribe(sub));
}

#[test]
fn subscribe_during_drain_gets_notified_next_cycle() {
    let mut rt = StoreRuntimeImpl::new();
    let key = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));

    // Local counter for first subscriber
    let c1 = Rc::new(RefCell::new(0usize));
    let c1_for_cb = c1.clone();
    let (c2, cb2_counter) = rc_counter();
    let subscribed = Rc::new(Cell::new(false));
    let subscribed_flag = subscribed.clone();
    let c2_cb = cb2_counter.clone();

    // First subscriber; during its callback, subscribe a second one.
    let cb1: StoreCallback = Rc::new(move |store, _sub, rt: &mut StoreEffects| {
        *c1_for_cb.borrow_mut() += 1;
        if !subscribed_flag.get() {
            subscribed_flag.set(true);
            let _ = rt.subscribe(store, c2_cb.clone());
        }
    });

    let _sub1 = rt.subscribe(key, cb1);

    rt.set_value(key, Box::new(1i32));
    rt.drain_notifications();

    // First subscriber fired in first cycle; second subscriber subscribed during drain should NOT receive a notification for the prior change
    assert_eq!(*c1.borrow(), 1);
    assert_eq!(*c2.borrow(), 0);
}

#[test]
fn subscribe_during_drain_then_mutate_triggers_next_cycle_notification() {
    let mut rt = StoreRuntimeImpl::new();
    let key = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));

    // First subscriber counter
    let c1 = Rc::new(RefCell::new(0usize));
    let c1_for_cb = c1.clone();
    let (c2, cb2_counter) = rc_counter();
    let subscribed = Rc::new(Cell::new(false));
    let subscribed_flag = subscribed.clone();
    let c2_cb = cb2_counter.clone();

    // In the first callback, subscribe c2 and then mutate the store so c2 is enqueued
    // into the next generation according to the design.
    let cb1: StoreCallback = Rc::new(move |store, _sub, rt: &mut StoreEffects| {
        *c1_for_cb.borrow_mut() += 1;
        if !subscribed_flag.get() {
            subscribed_flag.set(true);
            let _ = rt.subscribe(store, c2_cb.clone());
            rt.set_value(store, Box::new(2i32));
        }
    });

    let _sub1 = rt.subscribe(key, cb1);

    // Initial change to kick off drain
    rt.set_value(key, Box::new(1i32));
    rt.drain_notifications();

    // First subscriber fired in first cycle; mutation enqueued BOTH existing subscribers
    // (including the first) for the next generation, so c1 fires again too.
    assert_eq!(*c1.borrow(), 2);
    assert_eq!(*c2.borrow(), 1);
}

#[test]
fn multi_cycle_chain_a_to_b_to_c() {
    // This test ensures drain processes more than one cycle.
    let mut rt = StoreRuntimeImpl::new();
    let a = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));
    let b = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));
    let c = rt.alloc_store(Box::new(EmittingStore::new(Box::new(0i32), Some(mk_eq_fn::<i32>()))));

    // A's change sets B; B's change sets C; we count notifications on C.
    let b_key = b;
    let c_key = c;

    let cb_a_to_b: StoreCallback = Rc::new(move |_store, _sub, rt: &mut StoreEffects| {
        rt.set_value(b_key, Box::new(10i32));
    });
     
    let cb_b_to_c: StoreCallback = Rc::new(move |_store, _sub, rt: &mut StoreEffects| {
        rt.set_value(c_key, Box::new(20i32));
    });

    let (c_counter, c_notify_cb) = rc_counter();

    let _ = rt.subscribe(a, cb_a_to_b);
    let _ = rt.subscribe(b, cb_b_to_c);
    let _ = rt.subscribe(c, c_notify_cb);

    // Trigger the chain
    rt.set_value(a, Box::new(1i32));
    rt.drain_notifications();

    // Expect exactly one notification on C, processed after >1 cycles.
    assert_eq!(*c_counter.borrow(), 1);
}
