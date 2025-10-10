```text
Program        ::= FragmentDecl*

FragmentDecl   ::= "fragment!"? "{" FragmentBody "}"
FragmentBody   ::= FragmentHeader FragmentMembers
FragmentHeader ::= Ident "(" ParamList? ")"?
ParamList      ::= Param ("," Param)*
Param          ::= Ident ":" Type

FragmentMembers ::= (StoreDecl | NodeDecl)*

StoreDecl      ::= "store" Ident "=" Expr

NodeDecl       ::= NodeOpen Block NodeClose?
NodeOpen       ::= Ident OptArgs "{" OpenInstrList?
NodeClose      ::= "}" CloseInstrList?
Block          ::= FragmentMembers?    // children

OpenInstrList  ::= Instr (Chain Instr)*
CloseInstrList ::= Instr (Chain Instr)*
Chain          ::= ".."

Instr          ::= StyleUse | InstrCall | InstrFlag

StyleUse       ::= Ident                      // refers to a named style
InstrCall      ::= Ident "{" ArgList? "}"
InstrFlag      ::= Ident                      // e.g., no_select, space_between

ArgList        ::= Arg ("," Arg)*
Arg            ::= Ident ":" Value
OptArgs        ::= /* empty */ | "{" ArgList "}"

Value          ::= Number Unit? 
                 | String
                 | ColorLiteral 
                 | Enum
                 | Bool
                 | ResourceRef
                 | Tuple
                 | List
                 | InterpString              // for text content interpolation

Unit           ::= "DIP" | "SP"
Enum           ::= Ident                      // e.g., max, container, content, wrap, etc.
Bool           ::= "true" | "false"
Tuple          ::= "(" Value ("," Value)+ ")"
List           ::= "[" Value ("," Value)* "]"
ColorLiteral   ::= "#" Hex6 | "rgb" "(" Int "," Int "," Int ")"
ResourceRef    ::= Ident                      // font names, image ids, etc.
Number         ::= Int | Float
Int            ::= [0-9]+
Float          ::= [0-9]+"."[0-9]+
String         ::= "\"" .*? "\""               // usual escapes allowed
Ident          ::= [_A-Za-z][_0-9A-Za-z]*

StyleDecl      ::= "style" Ident "(" ParamList? ")"? "{" Instr (Chain Instr)* "}"
```