# Context

I am planning a reactive, multiplatform UI library in Rust. This document is just a plan
I'm working on, not a final design specification, so it contains quite a few inconsistencies
and there are problems not yet solved.

# Goals

**Main objective:** Build a modern, reactive, truly platform-independent UI library in Rust 
that emphasizes clarity and expressiveness — a concise DSL where code reads like intent rather 
than syntax.

1. **Declarative Composition:**
    Enable developers to define application structure and behavior through a concise, clean, 
    declarative syntax. The library should make it easy to express reactive and reusable UI 
    elements without focusing on platform or implementation details.
2. **Compact Runtime Footprint:**
    Ensure generated code remains small and efficient. The library should avoid unnecessary
    monomorphization and strive for minimal runtime overhead even with many components.
3. **Platform Independence:**
    Support multiple platforms through a clean separation between the declarative layer and 
    platform-specific rendering backends. Core logic and layout behavior must remain portable.
4. **Deterministic Rendering and Layout:**
    Provide a layout and rendering pipeline that behaves predictably across backends. 
    Platform-dependent operations should be minimal, deterministic, and isolated from reactive logic.

# Architecture

The library provides a structured way to define, compile, and execute declarative 
programs that can target multiple backends — such as interactive UIs, PDF documents, or image exports.

It separates **compile-time** and **runtime** responsibilities, making the system 
both modular and extensible.

## High-Level Flow

```text
DSL Source
   ↓
[Fragment Compiler] ──→ Fragment IR (FIR)
   ↓
[Fragment Linker] ──→ Linked Runtime Tree of instances (with reactive graph of stores and signals)
   ↓
[Fragment Renderer] ──→ Output (UI / PDF / PNG / etc.)
```

## Stages and Responsibilities

| Stage                  | Component                                | Responsibility                                                                                                                                     |
|------------------------|------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| **1. Source Code**     | **Fragment DSL**                         | **Defines** declarative fragment structures written by the developer.                                                                              |
| **2. Compile Stage**   | **Fragment Compiler** (procedural macro) | **Transforms** the DSL into backend-neutral **Fragment IR (FIR)**; performs validation, symbol resolution, and metadata embedding.                 |
| **3. Link Stage**      | **Fragment Linker** (runtime)            | **Instantiates and links** fragments, stores, and handlers into a connected runtime tree.                                                          |
| **4. Execution Stage** | **Fragment Renderer** (runtime)          | **Applies** the linked runtime tree to the target backend. After the initial build, updates are propagated incrementally through reactive signals. |

## Core Concepts

* [**Fragment DSL**](dsl.md)
  A declarative language for describing fragments.

* [**Fragment Compiler (FC)**](compiler.md)
  The procedural macro that compiles the Fragment DSL into **Fragment IR (FIR)**.

* [**Fragment Intermediate Representation (FIR)**](fir.md)
  A backend-agnostic program that describes how a fragment should be built, connected, and parameterized for later linking and rendering.

* [**Fragment Linker (FL)**](linker.md)
  The runtime component that takes FIR definitions, instantiates fragments, and links them into a connected runtime tree with stores, actions, and derived values.

* [**Fragment Renderer (FR)**](renderer.md)
  The runtime component that applies the linked runtime tree to a specific backend. It performs the initial build and then reacts to fine-grained signal updates.

* **Fragment**
  A reusable declarative unit described in FIR. A fragment can be headless (logic-only) or produce renderable nodes.

* [**Instance**](instances.md)
  A concrete runtime instance of a fragment created by the Fragment Linker.

* [**Store**](stores.md)
  A backend-neutral state container exposing reactive signals to fragments.
