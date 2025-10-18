# Browser Adapter

Note: this file is very preliminary, it is just a bunch of ideas.

- The Rust code is compiled to WebAssembly and runs in a worker (separate thread).
- All DOM manipulation is done in the main thread with JavaScript.

## Browser events

A `SharedArrayBuffer` is used to communicate events from the browser to the Rust code.

The buffer is separated into two areas:

- a small array of 64-bit atomics for the stateful lane
- a larger ring of events for the transactional lane

Browser events are split into two lanes:

- stateful
- transactional

## Stateful lane

The stateful lane contains 64-bit atomics:
 
- window resize: 1 slot
- continuous pointermove during drags: 1 slot
- viewport scroll: N slots (per scrollable viewport)

**Transactional lane, delivers all**

- pointerdown/up, click, enter/leave
- key*
- wheel (usually transactional; may coalesce when scroll-animate)

### Passing events to Rust

>> explain how the events are moved from the buffers to the event queue, maybe
>> poll at flow start?

From there the standard [Render flow](../40_render/renderer.md#render-flow) 
takes care of it.

## Backpressure

Happens when:

1. the DOM updates can't keep up with the Rust code,
2. the Rust code can't keep up with events from the browser (e.g., mouse move).

>> I think (not sure) that we can handle this with the buffer rotation. The only problem
>> is when the buffer is full, we need to drop some events.
