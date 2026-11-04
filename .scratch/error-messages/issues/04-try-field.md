# The `try` field, with first examples

Status: ready-for-agent
Type: task

## What

A new optional field in `content/reference.kdl`, the card's example:

```kdl
forward {
  eg """
  FORWARD dist
  FD dist
  """
  try "FD 100"
}
```

- `Command` in `reference.rs` gets it, under a name that isn't a Rust keyword, such as `short_example`. The header comment in the kdl says what it's for.
- Rules, checked by a test: one line, 40 characters at most, and it runs without an error on an empty program.
- It needs no saved picture, as `tests/examples.rs` requires of the help's examples. A separate test runs them.

**First examples:** draft `try` for each command the tutorial teaches, found by going through `content/tutorial`, plus any command a help page's first examples lean on. Use the shortest form a learner would write (`FD`, not `FORWARD`). Where a command only makes sense with others, such as `REPCOUNT`, the example can be a small whole instruction.

## Done when

- The field parses, and the test above passes for every `try`.
- The list of drafted examples is in a comment below, for the maintainer to skim. Then set the status to `resolved` once they have.

## Comments
