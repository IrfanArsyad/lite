; Headings
(atx_heading (atx_h1_marker)) @keyword
(atx_heading (atx_h2_marker)) @keyword
(atx_heading (atx_h3_marker)) @keyword
(atx_heading (atx_h4_marker)) @keyword
(atx_heading (atx_h5_marker)) @keyword
(atx_heading (atx_h6_marker)) @keyword
(setext_heading) @keyword

; Inline
(emphasis) @string
(strong_emphasis) @string
(strikethrough) @comment

; Code
(code_span) @string
(fenced_code_block) @string
(indented_code_block) @string

; Links
(link_text) @string
(link_destination) @property
(link_title) @string

; Lists
(list_marker_minus) @punctuation
(list_marker_plus) @punctuation
(list_marker_star) @punctuation
(list_marker_dot) @punctuation
(list_marker_parenthesis) @punctuation

; Block quotes
(block_quote_marker) @punctuation

; Thematic breaks
(thematic_break) @punctuation

; HTML in markdown
(html_block) @tag
