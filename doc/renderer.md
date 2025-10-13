# Fragment Renderer

**Actual UI**: The platform-dependent UI which is actually shown to the user.

**Adapter**: A platform-dependent adapter that is able to create/remove/update nodes in the actual UI.

**Node**: A part of the actual UI that can be added/removed/updated. DOM node, View instance, UIView instance.

## Render flow

```text
[Event Handler]
   ├─ Primary store mutations (direct changes from event handlers)
   ↓
[Store runtime] 
   ├─ Drain notifications (cascading store mutations), add fragments to render queue
   ↓
[Instruction Applier]
   ├─ Apply fragment instructions → update supplied render data
   ├─ Determine invalidation type (layout / paint)
   ├─ Generate paint patches
   ↓
[Layout Engine]
   ├─ Measure intrinsic sizes
   ├─ Compute layout
   ├─ Produce derived render data (positions, bounds, line boxes, etc.)
   ├─ Generate position patches
   ↓
[Adapter]
   ├─ Apply position & paint patches in batch (DIP → px)
   └─ Commit to actual UI
   ↓
Output (UI / PDF / PNG / etc.)
```

## Render data

Each primitive has a render data structure which consists of two major parts:

- supplied render data:
    - styling, alignment, layout algorithm
- derived render data:
    - actual positions

### Supplied render data

Each primitive fragment has an external store called `instructions`. The
 **Fragment Compiler** initializes this store from the instructions present in
the DSL.

The primitive fragments subscribe to this store, and whenever the value of the 
store changes, they update the supplied render data according to the instructions.

Instructions which handle dimensional data **always** store the value in DIP
(Device Independent Pixel).

Render data **always** stores dimensional data as **raw pixels**.

```rust
struct DIP(f32); // device independent pixel
struct PX(f32); // raw pixel

struct Surrounding {
    top : DIP,
    right : DIP,
    left : DIP,
    bottom: DIP
}

struct RawSurrounding {
    top : PX,
    right : PX,
    left : PX,
    bottom: PX
}

struct Color {
    red : u8,
    green : u8,
    blue : u8,
    opacity : u8
}

enum Instruction {
    Padding(Surrounding),
    BackgroundColor(Color)
}

struct RenderData {
    dirty_mask : u64, // 1 when the field has been changed since the last render
    padding: RawSurrounding,
    background_color : Color
}
```

## Box model

```text
    Top
Left┌────────────────────────────────────────────┐   ┬
    │                  MARGIN                    │   │
    │  ┌──────────────────────────────────────┐  │   │
    │  │               BORDER                 │  │   │
    │  │  ┌───────────────────────────────┐   │  │   │
    │  │  │           PADDING             │   │  │   │  Height
    │  │  │  ┌─────────────────────────┐  │   │  │   │
    │  │  │  │        CONTENT          │  │   │  │   │
    │  │  │  └─────────────────────────┘  │   │  │   │
    │  │  └───────────────────────────────┘   │  │   │
    │  └──────────────────────────────────────┘  │   │
    └────────────────────────────────────────────┘   ┴ 
    ├────────────────── Width ───────────────────┤
```

The first three: margin, border and padding together are called **surrounding**.

Width and height together define the size of the node.

In contrast with other layout systems, **margin is counted into the size** of a node.

As the picture above shows, top-left means the top-left corner of the surrounding,
margins included.

## Primitives

Each supported platform provides the following primitives:

1. Box
    - A generic rectangle to paint and clip.
    - Props: background (solid or none), border {color, width}, corner_radius, opacity, clip (on/off).
2. Text
    - Props: content (string), font (ref), size, weight, line_height, color, align (start/center/end), wrap (word/any/none), overflow (visible/ellipsis).
3. Raster image
    - Props: source (image ref), fit (fill/contain/cover/none), align (x,y inside box), opacity.
4. Icon
    - A tiny, stylable symbol (font-glyph or vector).
    - Props: icon_ref (glyph id or vector path id), size, color, opacity.
5. Native text host
    - Platform native text input
    - Props:
        - kind: "single" | "multiline" | "password" | "number" | "search"
        - placeholder?: string
        - font, size, color, align, line_height? (styling parity with Text)
        - keyboard_hint?: Text | Email | Number | Url | Password
        - autocorrect?: bool, autocapitalize?: bool, spellcheck?: bool
        - selection_color?, caret_color? (optional, platform may ignore)
6. Group
    - A logical parent with no paint of its own.
    - Props: opacity, transform (translate only to start), isolation (new stacking context? optional).
7. Native host
    - Renders native platform widgets, provideds way to use special features of a given platform.

## Somewhat outdated stuff

### UI data model

Parts:

- supplied render data:
    - styling, alignment, layout algorithm
- measurements (intrinsic sized)
- derived render data:
    - actual positions
- layout tree:
    - container fragment <-> contained fragment
- event handlers (maybe part of supplied render data)

All these are conceptually platform-independent. And could be handled as any other
application data (in stores).

The renderers can simply take this data and update the UI.

Things to analyse:

- layout
    - intristic sizes (text fragment for example)
- layers
    - dialogs
    - popups and dropdowns
    - notifications (snackbar)
- events
    - i'm thinking about something like this:
        - event queue loop:
            - take event, execute event handler - changes stores
            - drain_notifications will execute all necessary app state changes
                - could update render data as well, queue changed render data
            - apply changed render data to actual UI

Should we go with platform-dependent animations or simply model them ourselves?