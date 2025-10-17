# Layout

**Intrinsic node**: A node that has an intrinsic size (text size, image size, etc.)

**Content-sized node**: A node that has a size defined by its content (column, row, etc.) All intrinsic nodes are content-sized.

Starting point of the layout process:

1. ART has all the supplied render data.
2. ART contains all nodes.
3. The actual UI has a width and a height (measured in a platform-dependent way).

We start with the first viewport and perform a general, mostly algorithm antagonistic process.

1. Measure all content-sized nodes.
   1. Intrinsic nodes simply return their `intrinsic_size`.
   2. Container nodes perform a layout pass and return with `content_size + surrounding`. 
2. Distribute available space among children.
3. Layout all non-content sized children.
4. Position all children.
5. Generate layout patches