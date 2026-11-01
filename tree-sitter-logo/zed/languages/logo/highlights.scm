; Logo highlights. No two patterns match the same node, since editors
; differ about which one wins.

(comment) @comment

["to" "end"] @keyword

(name) @function

(parameter) @variable.parameter

; Built-in commands and special variables, from Pendown's content/reference.kdl.
; Generated: change them with
; UPDATE_HIGHLIGHTS=1 cargo test -p pendown-logo --test tree_sitter
; BEGIN GENERATED
((word) @function.builtin
  (#match? @function.builtin "^(?i)(and|apply|arc|arctan|array|array\\?|arrayp|arraytolist|ascii|back|background|before\\?|beforep|bf|bfs|bg|bk|bl|butfirst|butfirsts|butlast|case|catch|char|circle|clean|clearscreen|combine|cond|cos|count|cs|dequeue|difference|do\\.until|do\\.while|ellipse|empty\\?|emptyp|end|equal\\?|equalp|error|exp|fc|fd|fence|fillcolor|filter|find|first|firsts|for|foreach|forever|form|forward|fput|global|greater\\?|greaterequal\\?|greaterequalp|greaterp|heading|hideturtle|home|ht|if|ifelse|ignore|int|invoke|iseq|item|last|left|less\\?|lessequal\\?|lessequalp|lessp|list|list\\?|listp|listtoarray|ln|local|localmake|log10|lowercase|lput|lt|make|map|map\\.se|mdarray|mditem|mdsetitem|member|member\\?|memberp|minus|modulo|name|not|notequal\\?|notequalp|number\\?|numberp|op|or|output|palette|pc|pd|pencolor|pendown|pendown\\?|pendownp|pensize|penup|pick|pop|pos|power|pr|print|product|pu|push|queue|quoted|quotient|radarctan|radcos|radsin|random|rectangle|reduce|remainder|remdup|remove|repcount|repeat|rerandom|reverse|right|round|rseq|rt|run|runresult|se|sentence|setbackground|setbg|setfc|setfillcolor|seth|setheading|setitem|setpalette|setpc|setpencolor|setpensize|setpos|setspeed|setx|setxy|sety|show|shown\\?|shownp|showturtle|sin|speed|sqrt|square|st|stop|substring\\?|substringp|sum|thing|throw|to|towards|turtlemode|type|until|uppercase|wait|while|window|word|word\\?|wordp|wrap|xcor|ycor)$"))

((word) @function.call
  (#not-match? @function.call "^(?i)(and|apply|arc|arctan|array|array\\?|arrayp|arraytolist|ascii|back|background|before\\?|beforep|bf|bfs|bg|bk|bl|butfirst|butfirsts|butlast|case|catch|char|circle|clean|clearscreen|combine|cond|cos|count|cs|dequeue|difference|do\\.until|do\\.while|ellipse|empty\\?|emptyp|end|equal\\?|equalp|error|exp|fc|fd|fence|fillcolor|filter|find|first|firsts|for|foreach|forever|form|forward|fput|global|greater\\?|greaterequal\\?|greaterequalp|greaterp|heading|hideturtle|home|ht|if|ifelse|ignore|int|invoke|iseq|item|last|left|less\\?|lessequal\\?|lessequalp|lessp|list|list\\?|listp|listtoarray|ln|local|localmake|log10|lowercase|lput|lt|make|map|map\\.se|mdarray|mditem|mdsetitem|member|member\\?|memberp|minus|modulo|name|not|notequal\\?|notequalp|number\\?|numberp|op|or|output|palette|pc|pd|pencolor|pendown|pendown\\?|pendownp|pensize|penup|pick|pop|pos|power|pr|print|product|pu|push|queue|quoted|quotient|radarctan|radcos|radsin|random|rectangle|reduce|remainder|remdup|remove|repcount|repeat|rerandom|reverse|right|round|rseq|rt|run|runresult|se|sentence|setbackground|setbg|setfc|setfillcolor|seth|setheading|setitem|setpalette|setpc|setpencolor|setpensize|setpos|setspeed|setx|setxy|sety|show|shown\\?|shownp|showturtle|sin|speed|sqrt|square|st|stop|substring\\?|substringp|sum|thing|throw|to|towards|turtlemode|type|until|uppercase|wait|while|window|word|word\\?|wordp|wrap|xcor|ycor)$"))

((variable) @variable.builtin
  (#match? @variable.builtin "^:(?i)(caseignoredp|erract|fullprintp|logoversion|printdepthlimit|printwidthlimit)$"))

((variable) @variable
  (#not-match? @variable "^:(?i)(caseignoredp|erract|fullprintp|logoversion|printdepthlimit|printwidthlimit)$"))
; END GENERATED

(quoted_word) @string

(number) @number

(operator) @operator

["[" "]" "{" "}" "(" ")"] @punctuation.bracket

"@" @punctuation.special
