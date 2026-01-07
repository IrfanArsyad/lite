; Keywords
[
  "and"
  "as"
  "assert"
  "async"
  "await"
  "break"
  "class"
  "continue"
  "def"
  "del"
  "elif"
  "else"
  "except"
  "exec"
  "finally"
  "for"
  "from"
  "global"
  "if"
  "import"
  "in"
  "is"
  "lambda"
  "nonlocal"
  "not"
  "or"
  "pass"
  "print"
  "raise"
  "return"
  "try"
  "while"
  "with"
  "yield"
  "match"
  "case"
] @keyword

; Functions
(function_definition name: (identifier) @function)
(call function: (identifier) @function.call)
(call function: (attribute attribute: (identifier) @function.call))

; Types/Classes
(class_definition name: (identifier) @type)

; Variables
(identifier) @variable
(attribute attribute: (identifier) @property)

; Constants
(true) @constant.builtin
(false) @constant.builtin
(none) @constant.builtin

; Strings
(string) @string
(interpolation) @string.special

; Numbers
(integer) @number
(float) @float

; Comments
(comment) @comment

; Operators
[
  "+"
  "-"
  "*"
  "**"
  "/"
  "//"
  "%"
  "@"
  "="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "+="
  "-="
  "*="
  "/="
  "//="
  "%="
  "**="
  "@="
  "&="
  "|="
  "^="
  ">>="
  "<<="
  ":="
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," ";" ":" "."] @punctuation.delimiter

; Parameters
(parameters (identifier) @parameter)
(default_parameter name: (identifier) @parameter)
(typed_parameter (identifier) @parameter)
(typed_default_parameter name: (identifier) @parameter)

; Decorators
(decorator) @attribute
