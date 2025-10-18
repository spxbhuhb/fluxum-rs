# Browser Adapter

Note: this file is very preliminary, it is just a bunch of ideas.

The browser adapter requires cross-origin isolation.

## Architecture

```text
[Browser main thread]
    └─ [Outer event handler]
         ├─ Transform the browser event to binary representation in a `SharedArrayBuffer`.
         └─ Send a message to the Drain Worker that contains (offset, size and sequence) of the event in the `SharedArrayBuffer`.
      ↓
[Event collector worker]
    └─ [Inner event handler]
         ├─ Put event (offset, size and sequence) into a round-robin queue.
         └─ Notify the Frame Worker.
      ↓   
[Frame worker]
    └─ [Frame processing loop]
         ├─ Wait for a notification from the Event collector worker.
         ├─ Process all events in the round-robin queue.
         ├─ Transform UI patching operations into binary representation in a `SharedArrayBuffer`.
         └─ Send a message to the Browser main thread that contains (offset, size and sequence) of the patch in the `SharedArrayBuffer`.
```

Note: there are two workers because it provides a way to drain the event queue. Without two independent workers events
would have to be processed in one-by-one.

### Data structures

There are two `SharedArrayBuffer`s:

1. One for events from the browser to the frame worker: `event SAB`.
2. One for UI patching operations from the frame worker to the browser: `patch SAB`.

The event collector worker and the frame worker share the queue of events.

### Synchronization

Messages sent using `postMessage` store:

- offset and size of the data in the `SharedArrayBuffer`
- sequence number of the message
- current write pointer in the **other** `SharedArrayBuffer`

The workers advance write pointers as they process messages. They only advance
when they get it in a message, so it is ensured that the other worker already
processed the data.

**Sending and receiving algorithm**

- Producer (writer):
    - Write the payload bytes into the SAB.
    - Write the sequence number into the SAB with Atomics.store — this is a release.
    - Send a message carrying (offset, size, sequence) to the consumer (or write to the round-robin queue and notify).
- Consumer (reader), upon receiving the message or a notification:
    - Read the sequence number with Atomics.load — this is an acquire.
    - Read and process payload.

## Backpressure

- The browser main thread drops events if there is not enough space in the `event SAB` (all events until space is available).
- The frame worker waits for enough space in the `patch SAB` before emitting a patch.
- There is no event coalescing.

