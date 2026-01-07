; Keywords
[
  "break"
  "case"
  "chan"
  "const"
  "continue"
  "default"
  "defer"
  "else"
  "fallthrough"
  "for"
  "func"
  "go"
  "goto"
  "if"
  "import"
  "interface"
  "map"
  "package"
  "range"
  "return"
  "select"
  "struct"
  "switch"
  "type"
  "var"
] @keyword

; Functions
(function_declaration name: (identifier) @function)
(method_declaration name: (field_identifier) @function)
(call_expression function: (identifier) @function.call)
(call_expression function: (selector_expression field: (field_identifier) @function.call))

; Types
(type_identifier) @type
(type_spec name: (type_identifier) @type)

; Variables
(identifier) @variable
(field_identifier) @property

; Constants
(true) @constant.builtin
(false) @constant.builtin
(nil) @constant.builtin
(iota) @constant.builtin

; Strings
(raw_string_literal) @string
(interpreted_string_literal) @string
(rune_literal) @character

; Numbers
(int_literal) @number
(float_literal) @float
(imaginary_literal) @number

; Comments
(comment) @comment

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "%"
  "&"
  "|"
  "^"
  "<<"
  ">>"
  "&^"
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
  "&^="
  "&&"
  "||"
  "<-"
  "++"
  "--"
  "=="
  "<"
  ">"
  "="
  "!"
  "!="
  "<="
  ">="
  ":="
  "..."
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ";" ":" "."] @punctuation.delimiter

; Package
(package_identifier) @namespace
