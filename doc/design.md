# Context

I am planning a reactive, multiplatform UI library in Rust. This document is just a plan
I'm working on, not a final design specification, so it contains quite a few inconsistencies
and there are problems not yet solved.

# Goals

**Main objective:** Build a modern, reactive, truly platform-independent UI library in Rust 
that emphasizes clarity and expressiveness — a concise DSL where code reads like intent rather 
than syntax.

1. **Declarative Composition:**
    Enable developers to define application structure and behavior through a concise, clean, 
    declarative syntax. The library should make it easy to express reactive and reusable UI 
    elements without focusing on platform or implementation details.
2. **Compact Runtime Footprint:**
    Ensure generated code remains small and efficient. The library should avoid unnecessary
    monomorphization and strive for minimal runtime overhead even with many components.
3. **Platform Independence:**
    Support multiple platforms through a clean separation between the declarative layer and 
    platform-specific rendering backends. Core logic and layout behavior must remain portable.
4. **Deterministic Rendering and Layout:**
    Provide a layout and rendering pipeline that behaves predictably across backends. 
    Platform-dependent operations should be minimal, deterministic, and isolated from reactive logic.

# Architecture

The library provides a structured way to define, compile, and execute declarative 
programs that can target multiple backends — such as interactive UIs, PDF documents, or image exports.

It separates **compile-time** and **runtime** responsibilities, making the system 
both modular and extensible.

## High-Level Flow

```text
DSL Source
   ↓
[Fragment Compiler] ──→ Fragment IR (FIR)
   ↓
[Fragment Linker] ──→ Linked Runtime Tree of instances (with reactive graph of stores and signals)
   ↓
[Fragment Renderer] ──→ Output (UI / PDF / PNG / etc.)
```

## Stages and Responsibilities

| Stage                  | Component                                | Responsibility                                                                                                                                     |
|------------------------|------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| **1. Source Code**     | **Fragment DSL**                         | **Defines** declarative fragment structures written by the developer.                                                                              |
| **2. Compile Stage**   | **Fragment Compiler** (procedural macro) | **Transforms** the DSL into backend-neutral **Fragment IR (FIR)**; performs validation, symbol resolution, and metadata embedding.                 |
| **3. Link Stage**      | **Fragment Linker** (runtime)            | **Instantiates and links** fragments, stores, and handlers into a connected runtime tree.                                                          |
| **4. Execution Stage** | **Fragment Renderer** (runtime)          | **Applies** the linked runtime tree to the target backend. After the initial build, updates are propagated incrementally through reactive signals. |

## Core Concepts

* **Fragment Compiler (FC)**
  The procedural macro that compiles the Fragment DSL into **Fragment IR (FIR)**.

* **FIR (Fragment Intermediate Representation)**
  A backend-agnostic program that describes how a fragment should be built, connected, and parameterized for later linking and rendering.

* **Fragment Linker (FL)**
  The runtime component that takes FIR definitions, instantiates fragments, and links them into a connected runtime tree with stores, actions, and derived values.

* **Fragment Renderer (FR)**
  The runtime component that applies the linked runtime tree to a specific backend. It performs the initial build and then reacts to fine-grained signal updates.

* **Fragment**
  A reusable declarative unit described in FIR. A fragment can be headless (logic-only) or produce renderable nodes.

* **Instance**
  A concrete runtime instance of a fragment created by the Fragment Linker.

* **Store**
  A backend-neutral state container exposing reactive signals to fragments.

# Store subsystem

1. The application state is represented by a (possibly high) number of stores.
2. `Store` = the sole source of truth.
    1. `Readable`: `get_any() : &dyn Any`, `subscribe(id)`, `unsubscribe(id)`
    2. `Writable`: adds `set_any(&dyn Any)`
3. Change propagation is push: when a Writable changes, it notifies subscribers.
4. Value access is pull: subscribers decide when to call `get()` (might be immediate, but could be deferred to the render/compute phase).

Core tenets:

- **Single abstraction**: `Store`.
- **One promise**: “You’ll be notified when the truth might have changed.”
- **One action**: “Call `get()` when you care.”

Notes: 

- One single thread manages the state of the application, no multi-thread synchronization is needed.
- Stores use `dyn Any` to avoid type constraints, the macro should ensure type safety and generate 
  downcast when necessary
- The store subsystem is mainly used from generated code. While it has to be consistent and 
  error-free, the code generation will offer guarantees. This means that fail paths do not have to 
  be developer-friendly; they are supposed not to be reached anyway.

Each fragment has two store sets:

- **External stores**: passed to the fragment
- **Internal stores**: declared and owned by the fragment

## Store types

| Kind       | Emits | Can write? | Who owns lifetime? | Typical use                    |
|------------|-------|------------|--------------------|--------------------------------|
| `Readable` | Yes   | No         | Declaring owner    | One-way data flow to child     |
| `Writable` | Yes   | Yes        | Declaring owner    | Two-way models, internal state |
| `Const`    | No    | No         | Creator            | Literals, config flags         |
| `Derived`  | Yes   | No         | Creator            | Computed from other stores     |

Note: Both `Readable` and `Writable` stores are implemented by `EmittingStore`. The difference
is semantic at the store subsystem level. The macro keeps track of store types and refuses code
that would write into a readable store.

## Store runtime

The store runtime:

- owns stores, provides `StoreKey` for referencing stores
- owns subscriptions, provides `SubscriptionKey` for referencing subscriptions
- provides functions to add/remove stores, subscribe/unsubscribe stores
- provides `set_value` function to change values in the stores with delayed notification
    - `set_value` queues the subscriptions to be notified
    - `drain_notifications` call the callbacks to deliver the notifications

## Store value change

In general, store value changes result in notifications. This is not desired when
the `set_any` is called with the same value as the current one.

As stores are type erased, comparison of the current and the new value is not trivial.

Emitting stores may support this optimization by having a comparison function `eq_fn`. This
function may be passed to the store during creation, so it can call it whenever `set_any`
is called and skip the change if the values are the same.

Code generation can use the `mk_eq_fn` function to create the `eq_fn` function for
any type that supports `PartialEq`.

## Notification mechanism

Application state is independent of rendering; reactivity is handled by stores calling the
callback function provided at subscription time.

- Callback calls **DO NOT** preserve the order of subscriptions.
- Callbacks are processed in batches called *generations*.
- `drain_notifications` works in a loop where each cycle performs the following steps.
    1. Sanity check, the loop should finish in less than or equal `GEN_LIMIT` cycles (1000 by default).
    2. Take all pending notifications from the pending queue: the **processed generation**.
    3. Empty the pending queue and increase the generation counter,
    4. Call callback functions of the **processed generation**.
        1. May change store values, those changes belong to the **next generation**.
        2. May queue a re-render (which is processed outside the store subsystem).
    5. Stop the cycle if the **next generation** has no notifications.

This ensures that all pending work is finished and all cascading changes are applied to
the appropriate stores.

Note: the `drain_notifications` function is not reentrant, it panics if called from within
a callback, this is by design.

`set_value` function of stores save the current generation of the runtime (`last_set_gen` 
field of the store when the store is writable).

### Subscribe during drain

Subscription during drain (which is almost all subscriptions in practice) has the
following behaviour.

If `last_set_gen` of the subscribed store is the current generation of the runtime, the
runtime adds the subscription to the pending queue. This ensures that the subscriber
will be notified of store changes as well.

If `last_set_gen` is not the current generation of the runtime, the subscription is **NOT**
added to the pending queue. If some callbacks change the store value later during the drain,
the subscription will be automatically added to the pending queue by the normal mechanism.

### Unsubscribe during drain

Unsubscribe during drain (also almost all cases in practice) simply removes the subscription
from the subscriptions managed by the runtime.

If the subscription key is reached later during the drain, the function won't be able to
fetch the `StoreSubscription` as it has been already removed. In this case it simply skips
the key and goes on.

### Derived store subscription

Derived stores subscribe to other stores. This subscription is performed by the linker at the
time fragments instances are built by executing the IR.

When a derived store is removed, the runtime automatically removes all subscriptions to
other stores.

Derived store logic is implemented in the callback function provided to the stores the
derived store subscribes to. In most cases the macro generates this function.

## Store types and traits

```rust
use thunderdome::{Index,Arena};

pub type StoreKey = Index;
pub type SubscriptionKey = Index;
pub type StoreCallback = Rc<dyn Fn(StoreKey, SubscriptionKey, &mut StoreEffects)>;

pub struct StoreSubscription {
    store: StoreKey,
    callback: StoreCallback
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
```

# DSL

The library defines a DSL for the fragment compiler to translate into fragment IR.

DSL example:

```rust
fragment! {
   Counter(label : String) {
      store count = 0

      column {
         padding { 16 } .. border { Red, 1 }
         
         button {
            on_click { count = count + 1 }
            text { "Click me" }
         }
   
         text { "${label}: ${count}" } .. text_small
      }
   }
}
```

Notes:

- While the '..' chaining is not Rust-idiomatic it is very clean and it provides extremely good readibility.

>> TODO provide some metadata for tooling and diagnostics, maybe emit it as metadata.

# Resources

The DSL may declare or reference resources.

The DSL **declares** resources it contains the actual value of the resource, such as a string 
literal or an inline style.

The DSL **references** resources when the resource is declared outside the fragment.

Resource types:

- numbers
- strings
- images
- vector graphics
- styles
- fonts
- files

>> TODO explain resource stores (const for literals, something else for downloaded resources)
>> TODO explain ResourceTable

# Instance subsystem

The instance subsystem manages fragment instances.

```rust
pub type InstanceKey = Index;

trait InstanceRuntime {

}

struct InstanceRuntimeImpl {
    instances: Arena<FragmentInst>
}

pub struct FragmentInst {
    pub id: u32,
    pub desc: &'static FragmentDesc,
    pub internal_stores: smallvec::SmallVec<[StoreKey; 8]>, // used for cleanup when the instance is dropped
    pub subscriptions: smallvec::SmallVec<[SubscriptionKey; 8]>, // used for cleanup when the instance is dropped
    pub children: smallvec::SmallVec<[FragmentInst; 8]> // used for cleanup when the instance is dropped
}
```

# IR

- The struct `FragmentIR` contains the IR of the fragment.
- The Fragment Linker processes the IR to build fragment instances.
- Once a fragment instance is built, the IR is not used anymore by that instance.

Binary structure of the IR:

1. The IR is a byte stream of instructions.
2. Instructions are variable-length, the first byte of the instruction contains
   1. Opcode in the low 6 bits
   2. An optional `length of arguments - 1` (in bytes) in the high 2 bits, zero if the instruction has no arguments.
   3. Argument length is a means to compress the argument; it is not opcode-dependent.
3. This allows for:
   1. 64 basic instructions.
   2. Up to 4 bytes for arguments.

This format has been created to minimize the size of the generated code.

```rust
const ARG_LEN_1 = 0x00;         // 1 byte for argument
const ARG_LEN_2 = 0x01 << 6;   // 2 byte for argument
const ARG_LEN_3 = 0x02 << 6;   // 3 byte for argument
const ARG_LEN_4 = 0x03 << 6;   // 4 byte for argument

#[repr(C)]
pub struct FragmentIR {
   pub node_count: u16,
   pub ext_store_count: u16,
   pub own_store_count: u16,
   pub resources: &'static ResourceTable,
   pub dependencies: &'static [FragmentIR],
   pub events_handlers: &'static [EventHandler], 
   pub derived_handlers: &'static [Fn()],
   pub ops: &'static [u8],
}

// instruction set

const OP_VERSION:       u8 = 0; // version of the IR format

const OP_CONST:         u8 = 1; // create a const store
const OP_READABLE:      u8 = 2; // create a readable store
const OP_DERIVED:       u8 = 3; // create a derived store
const OP_WRITABLE:      u8 = 4; // create a writable store

const OP_BEGIN:         u8 = 5; // create a new fragment instance
const OP_ARG_PASS:      u8 = 6; // pass through an existing store to the current fragment instance
const OP_ARG_CONST:     u8 = 7; // create a const store and use it as an external store for the current fragment instance
const OP_ARG_READABLE:  u8 = 8; // create a readable store and use it as an external store for the current fragment instance
const OP_ARG_DERIVED:   u8 = 9; // create a derived store and use it as an external store for the current fragment instance
const OP_ARG_WRITABLE:  u8 = 10; // create a writable store and use it as an external store for the current fragment instance
const OP_ARG_EH:        u8 = 11; // event handler

const OP_END:           u8 = 62; // end of the current fragment instance
```

Generated code example:

```rust
/// This is a conceptual example for readibility.
/// The macro would emit a constant binary BLOB for `ops`.
///
/// stores:
///   0 - external, the label parameter
///   1 - internal, writable, counter
///   2 - internal, derived, "${label}: ${count}"
pub static COUNTER_DESC = FragmentIR {

    resources: &[
        Const(0),
        Const("Click me"),
        Const([Padding(16), Border(Red, 1)]),
        Const([TextSmall()])
    ],

    dependencies: &[
        COLUMN_DESC,
        BUTTON_DESC,
        TEXT_DESC,
    ],

    events_handlers: &[
        EhDesc(&[1], counter_increment_fn), // uses store 1 (count)
    ],

    derived_handlers: &[
        DeriveDesc(&[0, 1], label_derive_fn), // uses stores 0 (label) and 1 (count)
    ],

    // inline ops
    ops: &[
        op!(OP_VERSION, 1), // version of the IR format

        op!(OP_WRITABLE, 0), // Creates a writable store by copying the value from Const(0) in resources
        op!(OP_DERIVED, 0),  // Creates a derived store from the first description in derived_handlers

        op!(OP_BEGIN, 0),    // COLUMN_DESC in dependencies
        op!(OP_ARG_CONST, 2), // Const([Padding(16), Border(Red, 1)]) in resources

        op!(OP_BEGIN, 1),    // BUTTON_DESC in dependencies
        op!(OP_ARG_EH, 0),   // first of the events_handlers

        op!(OP_BEGIN, 2),    // TEXT_DESC in dependencies
        op!(OP_ARG_CONST, 1), // Const("Click me") in resources
        op!(OP_END, 0),      // text

        op!(OP_END, 0),      // button

        op!(OP_BEGIN, 2),    // TEXT_DESC in dependencies
        op!(OP_ARG_DERIVED, 2), // content of the derived store at store index 2
        op!(OP_END, 0),      // text

        op!(OP_END, 0),      // column
    ],
};
```
