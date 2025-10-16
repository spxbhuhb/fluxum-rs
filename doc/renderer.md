# Fragment Renderer

## Definitions

**Actual UI**: The platform-dependent UI which is actually shown to the user.

**Adapter**: A platform-dependent adapter that is able to create/remove/update nodes in the actual UI.

**Node**: A part of the actual UI that can be added/removed/updated. DOM node, View instance, UIView instance.

**Container** : A node that can contain other nodes.

**Positional Container** : A container in which the children are placed at pre-defined positions.

**Algorithmic Container** : A container in which the children are placed by a layout algorithm.

**DIP**: Device Independent Pixel

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

`content`, called **content box** is the actual content of the node, for layouts it 
contains the children, for non-layout nodes it contains the actual content such as 
text, image, etc.

Width and height together define the size of the node.

In contrast with other layout systems, **margin is counted into the size** of a node.

As the picture above shows, top-left means the top-left corner of the surrounding,
margins included.

- **padding** uses the background of the node.
- **border** uses the specified border color.
- **margin** is transparent.

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
   ├─ Compute layout → update derived render data (positions, bounds, line boxes, etc.)
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

```rust
struct DIP(f32); // device independent pixel

struct Surrounding {
    top : DIP,
    right : DIP,
    left : DIP,
    bottom: DIP
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
    padding: Surrounding,
    background_color : Color
}
```

## Primitives

Each supported platform provides the following primitives.

### Group

A logical parent with no paint of its own.

Supported styling:

- none

### Rectangle

A generic rectangle to paint background and borders.

Supported styling:

- background
- border
- corner_radius
- shadow

### Text

A non-breakable string of characters.

Supported styling:

- font { name, size, weight, color }
- line_height
- no_select
- underline
- small_caps
- letter_spacing
- text_overflow { visible / ellipsis }

### Raster image

An image (PNG, JPG).

Size of the image is specified by layout.

Supported styling:

- none

### Icon

A tiny, stylable symbol (font-glyph or vector).

Size of the icon is specified by the layout.

Supported styling:

- color

### Native text host

A platform native text input.

Supported styling:

- font { name, size, weight, color }
- line_height
- letter_spacing

### Native host

Renders native platform widgets, provideds way to use special features of a given platform.

Supported styling:

- none