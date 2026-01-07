; Keywords
[
  "case"
  "do"
  "done"
  "elif"
  "else"
  "esac"
  "export"
  "fi"
  "for"
  "function"
  "if"
  "in"
  "local"
  "readonly"
  "select"
  "then"
  "until"
  "while"
  "declare"
  "typeset"
  "unset"
  "unsetenv"
] @keyword

; Functions
(function_definition name: (word) @function)
(command_name) @function.call

; Variables
(variable_name) @variable
(special_variable_name) @variable.builtin
(simple_expansion) @variable
(expansion) @variable

; Strings
(string) @string
(raw_string) @string
(ansi_c_string) @string
(heredoc_body) @string

; Numbers
(number) @number

; Comments
(comment) @comment

; Operators
[
  "="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "|"
  "&"
  ";"
  ";;"
  "!"
  "+"
  "-"
  "*"
  "/"
  "%"
] @operator

; Punctuation
["(" ")" "[" "]" "[[" "]]" "{" "}"] @punctuation.bracket
["," ";"] @punctuation.delimiter

; Redirects
(file_redirect) @operator
(heredoc_redirect) @operator
