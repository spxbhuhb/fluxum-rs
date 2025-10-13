// ast.rs
use proc_macro2::Span;
use syn::{Ident, Expr, Token};

pub struct FragmentFile {
    pub fragments: Vec<FragmentDecl>,
}

pub struct FragmentDecl {
    pub name: Ident,
    pub params: Vec<Param>,        // label: String
    pub body: Block,               // build stmts
    pub span: Span,
}

pub struct Param {
    pub name: Ident,
    pub ty: syn::Type,             // keep full Rust type for better checks
    pub span: Span,
}

pub struct Block {
    pub items: Vec<BuildStmt>,
    pub span: Span,
}

pub enum BuildStmt {
    Store(StoreDecl),              // store count = 0
    Node(NodeDecl),                // column { ... } .. modifier { ... }
    If(IfStmt),                    // if { } else { }
    Let(LetStmt),                  // let x = expr
    Expr(syn::Expr),               // bare expression handler, if you allow it
}

pub struct StoreDecl {
    pub kind: StoreKind,           // Const/Readable/Derived/Writable
    pub name: Ident,               // count
    pub init: StoreInit,           // literal/expr/derive spec
    pub span: Span,
}

pub enum StoreKind { Const, Readable, Derived, Writable }

pub enum StoreInit {
    Literal(syn::Expr),            // numbers/strings/arrays/styles etc.
    Derived(DerivedSpec),          // derived { uses: [...], body: Expr }
}

pub struct DerivedSpec {
    pub uses: Vec<Ident>,          // referenced stores
    pub body: syn::Expr,           // Rust expr computing the value
}

pub struct NodeDecl {
    pub name: Ident,               // column / button / text / SomeFragment
    pub args: Vec<NodeArg>,        // passed stores/consts/handlers
    pub children: Option<Block>,   // nested block
    pub chain: Vec<Modifier>,      // after `..` chain
    pub span: Span,
}

pub enum NodeArg {
    // explicit forms let lowering decide which OP_ARG_* to pick
    Pass(Ident),                   // an existing store by name
    Const(syn::Expr),              // literal -> const store
    Readable(syn::Expr),
    Derived(DerivedSpec),
    Writable(syn::Expr),
    EventHandler(EventHandler),    // on_click { ... }
}

pub struct EventHandler {
    pub name: Ident,               // on_click
    pub body: BlockOrExpr,         // block or single expr
}

pub enum BlockOrExpr { Block(Block), Expr(syn::Expr) }

pub struct Modifier {
    pub name: Ident,               // text_small
    pub arg: Option<syn::Expr>,    // style { ... } or padding { 16 }
}

pub struct IfStmt {
    pub cond: syn::Expr,
    pub then_block: Block,
    pub else_arm: Option<ElseArm>,
}

pub enum ElseArm { Block(Block), If(Box<IfStmt>) }

pub struct LetStmt {
    pub name: Ident,
    pub value: syn::Expr,
}
