/**
 * @file Logo, as UCBLogo and Pendown speak it.
 *
 * Tree-sitter can't know how many inputs a procedure takes, so it can't
 * group an instruction with its inputs as Logo's own parser does. It parses
 * the tokens and the structure: words, numbers, quoted words, variables,
 * operators, lists, arrays, parentheses, comments, and procedure
 * definitions, `TO ... END`. That's enough for colours, brackets, outlines
 * and folding.
 */

/// A case-insensitive keyword, such as TO.
const keyword = (word) =>
  alias(
    token(prec(1, new RegExp(word.split('').map((c) => `[${c.toLowerCase()}${c.toUpperCase()}]`).join('')))),
    word,
  );

module.exports = grammar({
  name: 'logo',

  extras: ($) => [/\s/, $.comment],

  rules: {
    source_file: ($) => repeat(choice($.procedure_definition, $._thing)),

    procedure_definition: ($) =>
      seq(
        keyword('to'),
        field('name', alias($.word, $.name)),
        repeat(field('input', $._input)),
        field('body', alias(repeat($._thing), $.body)),
        keyword('end'),
      ),

    /// The title's inputs come first: a variable or number after the name is
    /// one of them, not the start of the body.
    /// UCBLogo's optional inputs, `[:size 100]`, parse as a list starting
    /// the body.
    _input: ($) => prec(2, choice(alias($.variable, $.parameter), $.number)),

    _thing: ($) =>
      choice($.word, $.number, $.quoted_word, $.variable, $.operator, $.list, $.array, $.parenthesized),

    list: ($) => seq('[', repeat($._thing), ']'),

    array: ($) => seq('{', repeat($._thing), '}', optional(seq('@', $.number))),

    parenthesized: ($) => seq('(', repeat($._thing), ')'),

    comment: (_) => token(seq(';', /.*/)),

    number: (_) => /(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?/,

    quoted_word: (_) => /"[^\s\[\](){}]*/,

    variable: (_) => /:[^\s\[\](){};"+\-*\/=<>]+/,

    operator: (_) => choice('+', '-', '*', '/', '=', '<', '>', '<=', '>=', '<>'),

    word: (_) => /[^\s\[\](){};"':+\-*\/=<>0-9@][^\s\[\](){};+\-*\/=<>]*/,
  },
});
