# Fragment DSL

1. **Fragment DSL** is a declarative language for describing fragments.
2. The DSL is formally defined in [dsl.pest](dsl.pest).
3. The [**Fragment Compiler**](compiler.md) turns the DSL into the [**Fragment IR**](fir.md).

Example:

```rust
fragment! {
   Counter(label : String) {
      store count = 0

      column {
         padding { 16 } .. border { Red, 1 }
         
         button {
            on_click { count = count + 1 }
            text { "Click me" }
         }
   
         text { "${label}: ${count}" } .. text_small
      }
   }
}
```

## Structure

The DSL declares fragments, each having:

- a name
- external store declarations (parameters, optional)
- internal store declarations (optional)
- building statements (optional)

A building statement may be:

- fragment creation
- rendering instruction
- control structure

## Resources

The DSL may declare or reference resources.

The DSL **declares** resources it contains the actual value of the resource, such as a string
literal or an inline style.

The DSL **references** resources when the resource is declared outside the fragment.

Resource types:

- numbers
- strings
- images
- vector graphics
- styles
- fonts
- files

>> TODO explain resource stores (const for literals, something else for downloaded resources)
>> TODO explain ResourceTable