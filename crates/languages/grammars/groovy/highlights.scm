(line_comment) @comment
(block_comment) @comment
(groovydoc_comment) @comment.documentation

(number_literal) @number
(string_literal) @string
(string_fragment) @string
(boolean_literal) @boolean
(null_literal) @constant.builtin

[
  "+" "-" "*" "/" "%" "!" "~" "<<" ">>" ">>>"
  "<" "<=" ">" ">=" "==" "!=" "&" "^" "|" "&&" "||"
  ".." "..<" "<.." "<..<" "===" "!==" "<=>" "=~" "==~"
  "?:" "?" "=" "+=" "-=" "*=" "/=" "%=" "**=" "<<="
  ">>=" ">>>=" "&=" "^=" "|=" "?=" "**" "++" "--"
  "." "?." "??." "*." ".&" ".@" "::" "*:" "->" "..."
] @operator

(identifier) @variable

[
  "new" "as" "in" "instanceof"
] @keyword.operator

[
  "if" "else" "while" "do" "for" "return" "break" "continue"
  "throw" "throws" "assert" "try" "catch" "finally" "switch"
  "case" "default" "yield"
] @keyword.control

[
  "class" "trait" "interface" "enum" "record" "def" "var"
  "package" "import" "extends" "implements"
] @keyword

[
  "public" "private" "protected" "static" "final" "abstract"
  "synchronized" "native" "transient" "volatile" "strictfp"
] @keyword.modifier

(field_access field: (identifier) @property)
(safe_navigation_expression property: (identifier) @property)
(spread_dot_expression property: (identifier) @property)
(method_pointer_expression method: (identifier) @function)

(class_declaration name: (identifier) @type.definition)
(trait_declaration name: (identifier) @type.definition)
(interface_declaration name: (identifier) @type.definition)
(enum_declaration name: (identifier) @type.definition)
(record_declaration name: (identifier) @type.definition)
(method_declaration name: (identifier) @function)
(constructor_declaration name: (identifier) @constructor)
(formal_parameter name: (identifier) @variable.parameter)
(closure_parameter name: (identifier) @variable.parameter)
(variable_declarator name: (identifier) @variable)
(field_declaration (variable_declarator name: (identifier) @property))

(method_invocation function: (identifier) @function.call)
(method_invocation function: (field_access field: (identifier) @function.call))
(command_chain receiver: (identifier) @function.call)
(type_identifier) @type

((type_identifier) @type.builtin
  (#match? @type.builtin "^(boolean|byte|char|double|float|int|long|short|void)$"))

(annotation "@" @attribute)
