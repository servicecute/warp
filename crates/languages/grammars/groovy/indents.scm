[
  (class_body)
  (enum_body)
  (switch_block)
  (block)
  (closure)
  (list_literal)
  (map_literal)
  (argument_list)
  (formal_parameters)
] @indent.begin

["}" "]" ")"] @indent.end

[(switch_case) (switch_arrow_case) (switch_default)] @indent.branch
