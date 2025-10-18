# Runtime

The runtime orchestrates the execution of all dynamic behavior in the system.

It owns the store subsystem, manages fragment instances, and coordinates rendering 
through discrete runtime frames.

Each frame represents a complete, deterministic processing step that transforms events 
and state changes into visible output.

## Runtime Frame

The runtime frame is a transactional execution invoked by the adapter 
when events are available for processing.

It runs event handlers, drains store notifications, and generates render patches, 
which the adapter applies to produce the final UI.

Each frame runs to completion before another begins.
If new events arrive during a frame, they are queued and processed in the next one.

```text
[Event Handler]
   ├─ Primary store mutations (direct changes from event handlers)
   ↓
[Store runtime] 
   ├─ Drain notifications (cascading store mutations), add nodes to render queue
   ↓
[Fragment Renderer]
   ├─ Process the render queue
   ├─ Generate UI patches
   ↓
[Adapter]
   ├─ Apply UI patches
   └─ Commit to actual UI
   ↓
Output (UI / PDF / PNG / etc.)
```