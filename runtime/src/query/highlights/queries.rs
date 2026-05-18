//! Example highlight queries for common languages.

/// Rust syntax highlighting query.
pub const RUST_HIGHLIGHTS: &str = r#"
; Comments
(line_comment) @comment
(block_comment) @comment

; Strings
(string_literal) @string
(char_literal) @string

; Numbers
(integer_literal) @number
(float_literal) @number

; Keywords
[
  "as" "async" "await" "break" "const" "continue" "crate" "dyn"
  "else" "enum" "extern" "false" "fn" "for" "if" "impl" "in"
  "let" "loop" "match" "mod" "move" "mut" "pub" "ref" "return"
  "self" "Self" "static" "struct" "super" "trait" "true" "type"
  "unsafe" "use" "where" "while"
] @keyword

; Functions
(function_item name: (identifier) @function)
(call_expression function: (identifier) @function.call)

; Types
(type_identifier) @type
(primitive_type) @type.builtin

; Variables
(identifier) @variable

; Operators
[
  "+" "-" "*" "/" "%" "^" "!" "&" "|" "&&" "||"
  "<<" ">>" "==" "!=" "<" "<=" ">" ">="
  "=" "+=" "-=" "*=" "/=" "%=" "^=" "&=" "|="
  "<<=" ">>=" "?" "=>" "->" "::" ".." "..="
] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["." "," ":" ";"] @punctuation.delimiter
"#;

/// Python syntax highlighting query.
pub const PYTHON_HIGHLIGHTS: &str = r#"
; Comments
(comment) @comment

; Strings
(string) @string

; Numbers
(integer) @number
(float) @number

; Keywords
[
  "and" "as" "assert" "async" "await" "break" "class" "continue"
  "def" "del" "elif" "else" "except" "finally" "for" "from"
  "global" "if" "import" "in" "is" "lambda" "nonlocal" "not"
  "or" "pass" "raise" "return" "try" "while" "with" "yield"
] @keyword

; Functions
(function_definition name: (identifier) @function)
(call function: (identifier) @function.call)

; Constants
(true) @constant.builtin
(false) @constant.builtin
(none) @constant.builtin

; Variables
(identifier) @variable
"#;
