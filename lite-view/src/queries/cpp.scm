; Keywords
[
  "alignas"
  "alignof"
  "break"
  "case"
  "catch"
  "class"
  "co_await"
  "co_return"
  "co_yield"
  "concept"
  "const"
  "consteval"
  "constexpr"
  "constinit"
  "const_cast"
  "continue"
  "decltype"
  "default"
  "delete"
  "do"
  "dynamic_cast"
  "else"
  "enum"
  "explicit"
  "export"
  "extern"
  "final"
  "for"
  "friend"
  "goto"
  "if"
  "inline"
  "mutable"
  "namespace"
  "new"
  "noexcept"
  "operator"
  "override"
  "private"
  "protected"
  "public"
  "register"
  "reinterpret_cast"
  "requires"
  "return"
  "sizeof"
  "static"
  "static_assert"
  "static_cast"
  "struct"
  "switch"
  "template"
  "this"
  "throw"
  "try"
  "typedef"
  "typeid"
  "typename"
  "union"
  "using"
  "virtual"
  "volatile"
  "while"
] @keyword

; Types
(type_identifier) @type
(primitive_type) @type.builtin
(sized_type_specifier) @type.builtin
(auto) @type.builtin
(class_specifier name: (type_identifier) @type)

; Functions
(function_declarator declarator: (identifier) @function)
(function_declarator declarator: (qualified_identifier name: (identifier) @function))
(call_expression function: (identifier) @function.call)
(call_expression function: (qualified_identifier name: (identifier) @function.call))
(call_expression function: (field_expression field: (field_identifier) @function.call))

; Variables
(identifier) @variable
(field_identifier) @property

; Constants
(true) @constant.builtin
(false) @constant.builtin
(null) @constant.builtin
(nullptr) @constant.builtin

; Strings
(string_literal) @string
(raw_string_literal) @string
(char_literal) @character

; Numbers
(number_literal) @number

; Comments
(comment) @comment

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
  "::"
  "<=>"
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ";"] @punctuation.delimiter

; Namespace
(namespace_identifier) @namespace

; Templates
(template_argument_list) @punctuation.bracket
