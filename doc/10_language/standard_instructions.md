# Standard Instructions

**DIP**: Device Independent Pixel (floating point)

## Color

`<color>` is a 32-bit integer representing a color.

## Layout

### Position

`position { top : DIP left : DIP }`

Places the node at the given position.

**IMPORTANT**: Only positional containers support this instruction.

Note: `top` and `left` are relative to the `top` and `left` of the container **content box**
(that is without the surrounding).

Note: `bottom` and `right` are intentionally left out. Use `top` and `left` translated by 
the height and width of the node.

### Dimensions

`<dim> { <value> max : DIP min : DIP }`

`<dim>` may be one of:

- `width` : The width of the node.
- `height` : The height of the node.

`<value>` may be one of:

| Value       | Description                                                |
|-------------|------------------------------------------------------------|
| DIP         | The exact size of the node.                                |
| `expand`    | Use the space proposed by the container, grow if possible. |
| `container` | Use all the space proposed by the container.               |
| `content`   | Use the size of the child nodes plus the surrounding.      |

- `max` the maximum size of the node, optional
- `min` the minimum size of the node, optional

**Shorthands**

| Shorthand        | Full                                          |
|------------------|-----------------------------------------------|
| `fit_content`    | `width { content } .. height { content }`     |
| `fill_width`     | `width { container }`                         |
| `fill_height`    | `height { container }`                        |
| `fill`           | `width { container } .. height { container }` |
| `expand`         | `width { expand } .. height { expand }`       |

Note: percentages are not supported. For weight-based layouts use `grid` and
define a template.

### Surrounding

`<surrounding-type> { top : DIP right : DIP bottom : DIP left : DIP color : <color> }`

`<surrounding-type>` may be one of:

- `padding` : The padding surrounding the node.
- `border` : The border surrounding the node.
- `margin` : The margin surrounding the node.

All the dimensions are optional, but at least one must be specified.

`color` : The color of the border, optional, only for `border`.

**Shorthands**

NOTE: `color` may be used with any of the shorthands when `<type>` is `border`.

| Shorthand                     | Full                                                      |
|-------------------------------|-----------------------------------------------------------|
| `<type> { horizontal : DIP }` | `<type> { left: DIP right : DIP }`                        |
| `<type> { vertical : DIP }`   | `<type> { top: DIP bottom : DIP }`                        |
| `<type> { DIP }`              | `<type> { top: DIP right : DIP bottom : DIP left : DIP }` |

### Fill strategy

```
fill_strategy { constrain|constrain_reverse|resize_to_max }
```

Specifies how a directional algorithmic container (such as row or column) should fill its children.

- `constrain` : The children are measured one-by-one. The space used by **earlier** children is subtracted from the space available to **later** children.
- `constrain_reverse` : The children are measured one-by-one. The space used by **later** children is subtracted from the space available to **earlier** children.
- `resize_to_max` : Children are measured one-by-one, then resized to size of the largest child.

**Shorthands**

| Shorthand           | Full                                  |
|---------------------|---------------------------------------|
| `constrain`         | `fill_strategy { constrain }`         |
| `constrain_reverse` | `fill_strategy { constrain_reverse }` |
| `resize_to_max`     | `fill_strategy { resize_to_max }`     |

### Gap

`gap { width : DIP height : DIP }`

The gap between children. Positional containers ignore this instruction.

Both dimensions are optional, but at least one must be specified.

**Shorthands**

| Shorthand     | Full                              |
|---------------|-----------------------------------|
| `gap { DIP }` | `gap { width: DIP height : DIP }` |

### Inner Alignment

`<target> { horizontal: (start|center|end) vertical: (<top|center|baseline|bottom>) }`

`<target>` may be one of:

- `align_self` : Align the node on the given axis.
- `align_items` : Align all the children on the given axis.`

`align_self` has precedence over `align_items`.

**Shorthands**

| Shorthand                          | Full                                                          |
|------------------------------------|---------------------------------------------------------------|
| `<target>_center`                  | `<target> { horizontal : center vertical: center }`           |
| `<target>_<horizontal>_<vertical>` | `<target> { horizontal : <horizontal> vertical: <vertical> }` |

Examples:

```text
align_self { horizontal: start vertical: bottom }
align_items { horizontal: center vertical: top }

align_self_center
align_items_center

align_self_start_top
align_items_center_bottom
```

### Outer Alignment

`align_relative { horizontal: (before|start|center|end|after) vertical : (above|start|center|end|below)  }`

Note: `align_relative` is used mostly by popups to align themselves to the component they are relative to.

The following diagram shows the positions for each alignment. The corners/edges touch the node 
they are relative to (end/start at the previous/next pixel).

```text
  Before-Above  Start-Above    Center-Above    End-Above After-Above
              ┌─────────────────────────────────────────┐ 
  Before-Start│Start-Start    Center-Start     End-Start│After-Start
              │                                         │
 Before-Center│Start-Center   Center-Center   End-Center│After-Center
              │                                         │
   Before-End │Start-End       Center-End        End-End│After-End
              └─────────────────────────────────────────┘   
  Before-Below Start-Below   Center-Below      End-Below After-Below
```

Note: naming is asymmetric on purpose, so there is no conflict between horizontal and vertical.

### Spacing

- `space_around` : Distribute the space around the children.
- `space_between` : Distribute the space between the children.

### Scroll

`scroll { horizontal|vertical|both }`

The container’s content box area is scrollable (whatever you define as “inner size” in your docs). 
All child sizes (including your “margins are part of size” rule) contribute to the scrollable extent.

`scroll { <dir> }` and `<dim> { content }` are mutually exclusive in the given direction.
These combinations on the same node generate a compile-time error:

- `scroll { horizontal }` and `width { content }`
- `scroll { vertical }` and `height { content }`

### Notes

**Overflow** is supported only by scrolling. In my experience overflow clip and hidden are
tools junior developers use to hide layout bugs.

### Decoration

```
color { rgba: u32 }
color { rgb: u32  opacity: f32 }

background { color: Color }
background { color: Color opacity: f32 }
background { gradient: Gradient }
background { image : ImageResource }

corner_radius { top : DIP right : DIP bottom : DIP left : DIP }

shadow { color : Color offset_x : DIP offset_y : DIP deviation : DIP }
```

Note: `border` is a mix of decoration and layout. Border width is accounted for in the layout.

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

### Text

`SP` : Scaled Pixel

```
font { name : String size : SP weight : u32 color : Color}
line_height { height : DIP }
no_select
text_wrap { none|wrap }
underline
small_caps
letter_spacing { value : f64 }
```
