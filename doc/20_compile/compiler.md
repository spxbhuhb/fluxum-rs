# Fragment Compiler

The compiler is a Rust procedural macro that turns the [Fragment DSL](../10_language/dsl.md) into [Fragment IR](fir.md).


## Surrounding Rewriter (Lowering Pass)

**Purpose:**
Ensure that all layout and decoration instructions (`padding`, `border`, `margin`, etc.)
are applied to container nodes.

**Transformation rule:**  
If a basic fragment receives surrounding instructions,
the compiler emits an enclosing container with those instructions
and moves the basic fragment inside it.

Example:

```dsl
text { "Hello" } .. padding { 8 }
```

is transformed into:

```text
box {
    padding { 8 }
    text { "Hello" }
}
```