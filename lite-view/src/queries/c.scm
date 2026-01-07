; Keywords
[
  "break"
  "case"
  "const"
  "continue"
  "default"
  "do"
  "else"
  "enum"
  "extern"
  "for"
  "goto"
  "if"
  "inline"
  "register"
  "restrict"
  "return"
  "sizeof"
  "static"
  "struct"
  "switch"
  "typedef"
  "union"
  "volatile"
  "while"
  "_Alignas"
  "_Alignof"
  "_Atomic"
  "_Generic"
  "_Noreturn"
  "_Static_assert"
  "_Thread_local"
] @keyword

; Types
(type_identifier) @type
(primitive_type) @type.builtin
(sized_type_specifier) @type.builtin

; Functions
(function_declarator declarator: (identifier) @function)
(call_expression function: (identifier) @function.call)
(call_expression function: (field_expression field: (field_identifier) @function.call))

; Variables
(identifier) @variable
(field_identifier) @property

; Constants
(true) @constant.builtin
(false) @constant.builtin
(null) @constant.builtin

; Strings
(string_literal) @string
(system_lib_string) @string
(char_literal) @character

; Numbers
(number_literal) @number

; Comments
(comment) @comment

; Preprocessor
(preproc_directive) @keyword
(preproc_include) @keyword
(preproc_def) @keyword
(preproc_ifdef) @keyword
(preproc_else) @keyword
(preproc_if) @keyword

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
  "++"
  "--"
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
  "->"
  "."
  "?"
  ":"
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ";"] @punctuation.delimiter
