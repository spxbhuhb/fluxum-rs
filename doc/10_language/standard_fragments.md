# Standard Fragments

These fragments are part of the standard library.

## Primitives

### Text

### Image

### Icon

### Native Input

### Native Host

## Containers

Containers are fragments and manually implemented actual UI nodes that can contain other nodes.

### Box

Positions its children:

- directly with x and y coordinates **or**
- by aligning them with standard alignment instructions

### Column

Positions its children vertically below each other.

### Row

Positions its children horizontally next to each other.

### Flow Box

Positions its children in a row next to each other until there is no more available space. 
When there is no more space, opens a new row below.

### Grid

A partial implementation of the CSS grid.

### Split pane

Splits an area horizontally or vertically.