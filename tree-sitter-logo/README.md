# tree-sitter-logo

A [tree-sitter](https://tree-sitter.github.io/) grammar for Logo, as [UCBLogo](https://people.eecs.berkeley.edu/~bh/logo.html) and Pendown speak it, with a [Zed](https://zed.dev/) extension that uses it.

It lives in Pendown's repository for now, in `tree-sitter-logo/`, ready to become a repository of its own: Zed fetches grammars by git URL and commit.

## What it parses

Tree-sitter can't know how many inputs a procedure takes, so it can't group an instruction with its inputs as Logo's own parser does. It parses the tokens and the structure:

- words, numbers, `"quoted` words, `:variables` and the infix operators;
- `[...]` lists, `{...}` arrays (with `@origin`), `(...)` and `;` comments;
- `TO ... END` definitions, on several lines or one, in any case: the name, the inputs, and the body.

That's enough for colours, brackets, outlines, indentation and folding. UCBLogo's optional inputs on a title line, `[:size 100]`, parse as a list starting the body.

## Queries

- `highlights.scm`: Pendown's built-in commands and special variables are listed in it, generated from `pendown-logo.kdl`. Pendown's tests check it's up to date: `UPDATE_HIGHLIGHTS=1 cargo test -p pendown-logo --test tree_sitter` regenerates it, and copies the queries into the Zed extension.
- `brackets.scm`, `outline.scm` (procedures), `indents.scm` and `folds.scm`.

No two highlight patterns match the same node, since editors differ about which pattern wins.

## Working on it

With Node:

```sh
npx tree-sitter-cli@0.25.10 generate
npx tree-sitter-cli@0.25.10 test
```

`test/corpus/` has the parse tests, and `test/highlight/` the highlight tests.

## The Zed extension

`zed/` is the extension: `extension.toml`, and `languages/logo/` with `config.toml` (`.logo` files, `;` comments, brackets) and a copy of the queries. Once the grammar has its own repository, set `rev` in `extension.toml` to its commit.

## Before publishing

- Choose a licence: Pendown has none yet.
- Make the repository, push this folder to it, and set the extension's `rev`.
