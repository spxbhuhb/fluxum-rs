# Fluxum

A modern, reactive, truly platform-independent UI library for Rust —  
with a concise, declarative DSL where code reads like intent rather than syntax.

---

## Project Status: Early Design Phase

Fluxum is currently in the **planning and early design** stage.

The repository serves as a place to **document ideas, architecture drafts, and experimental prototypes** —  
not as a usable library (yet).

Expect:

- Rapidly changing concepts and structures
- Incomplete or exploratory code
- Discussions and design documents instead of working examples

If you’re interested in the long-term vision or want to follow the design process, this is the right place.

In this case check [overview.md](doc/overview.md).

If you’re looking for a production-ready UI library — check back later!

---

## Current Focus

- Designing the architecture
- Writing parts which have somewhat finished designs

---

## Relationship to Adaptive (Kotlin)

Fluxum builds on ideas and experience from [**Adaptive**](https://adaptive.fun) —  
a much broader library originally written in Kotlin ([github.com/spxbhuhb/adaptive](https://github.com/spxbhuhb/adaptive)).

Adaptive covers a wide range of application-level functionality.

Fluxum takes inspiration from it but focuses specifically on **the UI-related aspects**, reimagined in Rust.

Planned migrations from Adaptive include:

- **Fragment DSL**
- **Layout algorithms**
- **Style system** and resource management concepts
- **Platform connectors** for rendering and input abstraction
- **UI fragment library**

Fluxum is not a direct port — rather, it evolves Adaptive’s UI architecture into a Rust-native form  
while narrowing the scope to focus on declarative and reactive UI foundations.

---

## Contributing (Later)

While Fluxum is not ready for external contributions yet, early feedback on design direction,  
architecture, and API ideas is welcome through GitHub discussions or issues.  
Once the design stabilizes, contribution guidelines will be added.

---

## License

This project is licensed under either of

- **Apache License, Version 2.0**, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- **MIT license**, ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted  
for inclusion in this project by you, as defined in the Apache-2.0 license,  
shall be dual licensed as above, without any additional terms or conditions.
