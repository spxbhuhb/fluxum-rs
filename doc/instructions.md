## Instructions

`DIP` = Device Independent Pixel

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
font { name : String size : SP weight : u32 }
line_height { height : DPI }
no_select
text_wrap { none|wrap }
underline
small_caps
letter_spacing { value : f64 }
```
