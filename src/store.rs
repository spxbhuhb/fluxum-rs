use std::any::Any;
use std::rc::Rc;
use smallvec::SmallVec;
use thunderdome::{Arena, Index};

pub type StoreKey = Index;
pub type SubscriptionKey = Index;

pub type StoreGeneration = u64;

pub type StoreEffects = dyn StoreRuntime;

pub type StoreCallback = Rc<dyn Fn(StoreKey, SubscriptionKey, &mut StoreEffects)>;

pub struct StoreSubscription {
    store: StoreKey,
    callback: StoreCallback
}

const GEN_LIMIT: u64 = 1000;

/// The `Store` trait defines a generic interface for a storage mechanism that can hold and manage
/// dynamically typed data (`dyn Any`). It provides methods for reading, writing, and managing subscriptions.
/// It also offers a utility method to retrieve a debug/type name for diagnostic or tooling purposes.
pub trait Store {
    /// Untyped read; macro-generated code downcasts at use-sites.
    fn get_any(&self) -> &dyn Any;

    /// Untyped write; macro-generated code performs checked downcast.
    fn set_any(&mut self, value: Box<dyn Any>, sink: &mut SubSink);

    /// Register a listener. When returns with true, the store has been changed
    /// in this generation; therefore, the key should be added to the pending queue.
    fn subscribe(&mut self, key: SubscriptionKey, generation : StoreGeneration) -> bool;

    /// Remove a listener by id.
    fn unsubscribe(&mut self, key: SubscriptionKey);

    /// Subscriptions of this store. Used for cleanup, it is a life slice, not a snapshot.
    /// (Single-thread design guarantees that it cannot change during cleanup).
    fn subscriptions(&self) -> Option<&[SubscriptionKey]>;

    /// Dependencies of this store. Used for cleanup, it is a life slice, not a snapshot.
    /// (Single-thread design guarantees that it cannot change during cleanup).
    fn dependencies(&self) -> Option<&[SubscriptionKey]>;

    /// (Optional) stable debug/type name for tooling/metrics.
    fn debug_name(&self) -> &'static str { std::any::type_name::<Self>() }
}

pub trait StoreRuntime {
    /// Allocate a new store; returns a stable handle.
    fn alloc_store(&mut self, s: Box<dyn Store>) -> StoreKey;

    /// Remove a store.
    fn free_store(&mut self, key: StoreKey);

    /// Subscribe to a store.
    fn subscribe(&mut self, key: StoreKey, cb: StoreCallback) -> SubscriptionKey;

    /// Unsubscribe from a store.
    fn unsubscribe(&mut self, key: SubscriptionKey) -> bool;

    /// Erased write. Store appends subs into the sink; we enqueue those notifications
    /// into `pending`.
    fn set_value(&mut self, store_key: StoreKey, value: Box<dyn Any>);

    /// Drain the pending notifications and invoke callbacks.
    fn drain_notifications(&mut self);
}

// ---------------------------------------------------------------------------
// Runtime implementation
// ---------------------------------------------------------------------------

pub struct StoreRuntimeImpl {
    stores: Arena<Box<dyn Store>>,
    subscriptions: Arena<StoreSubscription>,
    pending: Vec<SubscriptionKey>,
    generation : StoreGeneration,
    is_draining: bool
}

impl StoreRuntimeImpl {
    pub fn new() -> Self {
        Self {
            stores: Arena::new(),
            subscriptions: Arena::new(),
            pending: Vec::new(),
            generation : 0,
            is_draining: false
        }
    }
}

impl StoreRuntime for StoreRuntimeImpl {

    #[inline]
    fn alloc_store(&mut self, s: Box<dyn Store>) -> StoreKey {
        self.stores.insert(s)
    }

    fn free_store(&mut self, key: StoreKey) {
        if let Some(store) = self.stores.remove(key) {
            // remove subscriptions this store has to other stores
            for sub in store.dependencies().unwrap_or_default() {
                self.unsubscribe(*sub);
            }
            // remove subscriptions to this store
            for sub in store.subscriptions().unwrap_or_default() {
                self.unsubscribe(*sub);
            }
        }
    }

    fn subscribe(&mut self, key: StoreKey, cb: StoreCallback) -> SubscriptionKey {
        if let Some(store) = self.stores.get_mut(key) {
            let sub = self.subscriptions.insert(StoreSubscription { store: key, callback: cb });
            let current = store.subscribe(sub, self.generation);
            if current { self.pending.push(sub); } // the store has changed in this generation, so we must enqueue the notification
            sub
        } else {
            panic!("attempt to subscribe to non-existent store");
        }
    }

    fn unsubscribe(&mut self, key: SubscriptionKey) -> bool {
        if let Some(sub) = self.subscriptions.remove(key) {
            if let Some(store) = self.stores.get_mut(sub.store) {
                store.unsubscribe(key);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn set_value(&mut self, store_key: StoreKey, value: Box<dyn Any>) {
        // local staging buffer for *this call*; avoids borrowing runtime during store logic
        let mut sink = SubSink::new(self.generation);

        if let Some(store) = self.stores.get_mut(store_key) {
            store.set_any(value, &mut sink);
        } else {
            panic!("attempt to write non-existent store");
        }

        if !sink.is_empty() {
            // append staged entries to the runtime's pending queue
            self.pending.extend(sink.local);
        }
    }

    /// Drain the pending notifications and invoke callbacks.
    fn drain_notifications(&mut self) {

        assert!(!self.is_draining);
        self.is_draining = true;

        let start_generation = self.generation;

        while !self.pending.is_empty() {
            // sanity check: if we don't finish the drain in GEN_LIMIT generations, something is wrong
            if self.generation > start_generation + GEN_LIMIT {
                panic!("store runtime: single drain generation limit has been exceeded");
            }

            // Take the current batch; allow callbacks to enqueue more work for *next* generation
            let batch = std::mem::take(&mut self.pending);

            // Increase generation, so callbacks can add other work for *next* generation
            // We are confined to a single-threaded runtime, so this is safe.
            self.generation += 1;

            for key in batch {
                if let Some(sub) = self.subscriptions.get_mut(key) {
                    let callback = sub.callback.clone();
                    callback(sub.store, key, self);
                }
            }            
        }

        self.is_draining = false;
    }
}

// ---------------------------------------------------------------------------
// SubSink: stores write subscriber keys into this
// ---------------------------------------------------------------------------

// A concrete sink the runtime uses during set_any: enqueues (store, sub) pairs.
pub struct SubSink {
    generation: u64,
    local: SmallVec<[SubscriptionKey; 8]>
}

impl SubSink {

    fn new(generation : u64) -> Self {
        Self { generation, local: SmallVec::new() }
    }

    #[inline]
    fn push(&mut self, subs: SmallVec<[SubscriptionKey; 8]>) {
        for k in subs { self.local.push(k); }
    }
    #[inline]
    fn len(&self) -> usize { self.local.len() }
    #[inline]
    fn is_empty(&self) -> bool { self.len() == 0 }
}

// ---------------------------------------------------------------------------
// Const
// ---------------------------------------------------------------------------

pub struct ConstInline<T: 'static>(pub T);

impl<T: 'static> ConstInline<T> {
    #[inline]
    pub fn get(&self) -> &T { &self.0 }
}

pub struct ConstErased<T: 'static>(ConstInline<T>);

impl<T: 'static> Store for ConstErased<T> {
    fn get_any(&self) -> &dyn Any {
        &self.0 .0
    }

    fn set_any(&mut self, _: Box<dyn Any>, _: &mut SubSink) {
        panic!("attempt to write const store, this is a framework error (or you've been naughty)")
    }

    fn subscribe(&mut self, _: SubscriptionKey, _ : StoreGeneration) -> bool {
        false // const stores never change
    }

    fn unsubscribe(&mut self, _: SubscriptionKey) {

    }

    fn subscriptions(&self) -> Option<&[SubscriptionKey]> {
        None
    }

    fn dependencies(&self) -> Option<&[SubscriptionKey]> {
        None
    }
}

// ---------------------------------------------------------------------------
// Emitting
// ---------------------------------------------------------------------------

// Signature for the value comparison function used by the EmittingStore to compare
// values before notifying subscribers. The code generation macro decides when
// to add this function to the store creation.
// TODO think about when and how to add EqFn, pay attention to monomorphism and code size
type EqFn = fn(old: &dyn Any, new: &dyn Any) -> bool;

pub fn mk_eq_fn<T: 'static + PartialEq>() -> EqFn {
    |old, new| {
        let (Some(o), Some(n)) = (old.downcast_ref::<T>(), new.downcast_ref::<T>()) else { return false; };
        o == n
    }
}

pub struct EmittingStore {
    value: Box<dyn Any>,
    eq_fn: Option<EqFn>,           // None => AlwaysNotify
    last_set_gen: StoreGeneration, // the store runtime generation when the last set_any was called
    subs: SmallVec<[SubscriptionKey; 8]>
}

impl EmittingStore {
    pub fn new(value: Box<dyn Any>, eq_fn: Option<EqFn>) -> Self {
        // First real set_any marks it to the live gen.
        // Subscribe will only return true if a real write happened in the current gen.
        Self {
            value,
            eq_fn,
            last_set_gen: u64::MAX,
            subs: SmallVec::new()
        }
    }
}

impl Store for EmittingStore {
    fn get_any(&self) -> &dyn Any {
        &*self.value
    }

    fn set_any(&mut self, value: Box<dyn Any>, sink: &mut SubSink) {
        // if the value is equal to the old value, we don't need to notify
        // TODO think about reference types (Box, Rc, Arc) and how to compare them
        if let Some(eq_fn) = self.eq_fn {
            if eq_fn(&*self.value, &*value) { return; }
        }

        self.value = value;
        if self.last_set_gen != sink.generation {
            sink.push(self.subs.clone()); // make clone so further changes don't affect the sink
            self.last_set_gen = sink.generation;
        }
    }

    fn subscribe(&mut self, key: SubscriptionKey, generation: StoreGeneration) -> bool {
        if !self.subs.contains(&key) {
            self.subs.push(key);
        }
        self.last_set_gen == generation
    }

    fn unsubscribe(&mut self, key: SubscriptionKey) {
        if let Some(i) = self.subs.iter().position(|&x| x == key) {
            self.subs.swap_remove(i); // no need to keep order, everything must work with any order by design
        }
    }

    fn subscriptions(&self) -> Option<&[SubscriptionKey]> {
        Some(&self.subs)
    }

    fn dependencies(&self) -> Option<&[SubscriptionKey]> {
        None
    }
}

// ---------------------------------------------------------------------------
// Derived
// ---------------------------------------------------------------------------

pub struct DerivedStore {
    base: EmittingStore,
    // The subscriptions this store depends on, used for cleanup
    deps: SmallVec<[SubscriptionKey; 4]>,
}

impl DerivedStore {
    pub fn new(
        initial: Box<dyn Any>,
        eq_fn: Option<EqFn>,
        callback: StoreCallback,
        deps: impl IntoIterator<Item = StoreKey>,
        runtime: &mut StoreEffects,
    ) -> Self {

        let mut deps_vec: SmallVec<[SubscriptionKey; 4]> = SmallVec::new();

        for dep in deps {
            deps_vec.push(runtime.subscribe(dep, callback.clone()));
        }

        DerivedStore {
            base: EmittingStore::new(initial, eq_fn),
            deps: deps_vec,
        }
    }
}

impl Store for DerivedStore {
    fn get_any(&self) -> &dyn Any {
        &*self.base.value
    }

    fn set_any(&mut self, value: Box<dyn Any>, sink: &mut SubSink) {
        // For a derived store, set_any is typically called by the derive function.
        //
        // Calling `set_any` of the base store ensures that the notification logic
        // provided by EmittingStore is triggered.
        //
        self.base.set_any(value, sink);
    }

    fn subscribe(&mut self, key: SubscriptionKey, generation: StoreGeneration) -> bool {
        self.base.subscribe(key, generation)
    }

    fn unsubscribe(&mut self, key: SubscriptionKey) {
        self.base.unsubscribe(key);
    }

    fn subscriptions(&self) -> Option<&[SubscriptionKey]> {
        self.base.subscriptions()
    }

    fn dependencies(&self) -> Option<&[SubscriptionKey]> {
        Some(&self.deps)
    }
}