# UI data model

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

## Instructions

`DIP` = Device Independent Pixel

### Decoration

```
color { rgba: u32 }
color { rgb: u32  opacity: f32 }

background { color: Color }
background { color: Color opacity: f32 }
background { gradient: Gradient }
background { image : ImageResource }

border { all : DIP, color : Color }
border { top : DIP right : DIP bottom : DIP left : DIP color : Color }

corner_radius { top : DIP right : DIP bottom : DIP left : DIP }

shadow { color : Color offset_x : DIP offset_y : DIP deviation : DIP }
```

### Event

```
on_click { }
on_double_click { }
on_pointer_move { }
on_pointer_enter { }
on_pointer_leave { }
on_primary_down { }
on_primary_up { }
on_secondary_down { }
on_secondary_up { }
on_key_down { }
on_enter { }

on_close { }

no_pointer_events
with_pointer_events
```

### Layout

```
position { top : DIP left : DIP }

size { width : DIP height : DIP }
size_strategy { horizontal: container|content vertical : container|content }

width { DIP }
width { max }

height { DIP }
height { max }

fill_strategy { constrain|constrain-reverse|resize-to-max|none }

gap { width : DIP height : DIP}

padding { top : DIP right : DIP bottom : DIP left : DIP }
padding { horizontal : DIP vertical : DIP }

margin { top : DIP right : DIP bottom : DIP left : DIP }
margin { horizontal : DIP vertical : DIP }

align_self { horizontal: (<top|center|bottom>) vertical: (start|center|end>) }
align_items{ horizontal: (<top|center|bottom>) vertical: (start|center|end>) }

space_around
space_between

scroll { horizontal : bool vertical : bool }
vertical_scroll
horizontal_scroll

overflow

popup_align { vertical : (above|start|center|end|below) horizontal: (before|start|center|end|after) }

z_index { index : u32 }
```

### Text

`SP` : Scaled Pixel

```
font { name : String size : SP weight : u32 }
line_height { height : DPI }
no_select
text_wrap { none|wrap }
underline
small_caps
letter_spacing { value : f64 }
```

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
