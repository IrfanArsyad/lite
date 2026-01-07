; Keywords
[
  "as"
  "async"
  "await"
  "break"
  "const"
  "continue"
  "crate"
  "dyn"
  "else"
  "enum"
  "extern"
  "fn"
  "for"
  "if"
  "impl"
  "in"
  "let"
  "loop"
  "match"
  "mod"
  "move"
  "mut"
  "pub"
  "ref"
  "return"
  "self"
  "Self"
  "static"
  "struct"
  "super"
  "trait"
  "type"
  "union"
  "unsafe"
  "use"
  "where"
  "while"
] @keyword

; Types
(type_identifier) @type
(primitive_type) @type.builtin

; Functions
(function_item name: (identifier) @function)
(call_expression function: (identifier) @function.call)
(call_expression function: (field_expression field: (field_identifier) @function.call))
(macro_invocation macro: (identifier) @function.macro)

; Variables
(identifier) @variable
(field_identifier) @property
(shorthand_field_identifier) @property

; Constants
(boolean_literal) @constant.builtin
(const_item name: (identifier) @constant)

; Strings
(string_literal) @string
(raw_string_literal) @string
(char_literal) @character

; Numbers
(integer_literal) @number
(float_literal) @float

; Comments
(line_comment) @comment
(block_comment) @comment

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "%"
  "="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "!"
  "&"
  "|"
  "^"
  "~"
  "<<"
  ">>"
  "+="
  "-="
  "*="
  "/="
  "%="
  "&="
  "|="
  "^="
  "<<="
  ">>="
  ".."
  "..="
  "=>"
  "->"
  "::"
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ";" ":" "."] @punctuation.delimiter

; Attributes
(attribute_item) @attribute
(inner_attribute_item) @attribute

; Lifetimes
(lifetime) @label
