; Selectors
(tag_name) @tag
(class_name) @type
(id_name) @type
(attribute_name) @tag.attribute
(pseudo_class_selector (class_name) @function)
(pseudo_element_selector (tag_name) @function)

; Properties
(property_name) @property

; Values
(plain_value) @variable
(color_value) @constant
(string_value) @string
(integer_value) @number
(float_value) @float

; Keywords
[
  "@media"
  "@import"
  "@charset"
  "@namespace"
  "@keyframes"
  "@supports"
  "@font-face"
  "@page"
  "and"
  "or"
  "not"
  "only"
  "from"
  "to"
] @keyword

; Units
(unit) @type

; Functions
(function_name) @function

; Comments
(comment) @comment

; Punctuation
["{" "}"] @punctuation.bracket
["(" ")"] @punctuation.bracket
["[" "]"] @punctuation.bracket
["," ";" ":"] @punctuation.delimiter

; Important
(important) @keyword
