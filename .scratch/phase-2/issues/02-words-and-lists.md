# The rest of words and lists

Status: ready-for-agent
Type: task
Blocked by: 01

## What

- **Constructors:** `FPUT`, `LPUT`, `COMBINE`, `REVERSE`.
- **Selectors:** `FIRSTS`, `BUTFIRSTS`/`BFS`, `PICK`, `REMOVE`, `REMDUP`, `QUOTED`.
- **Stacks and queues:** `PUSH`, `POP`, `QUEUE`, `DEQUEUE`, on the variable named by their first input.
- **Predicates:** `WORDP`, `LISTP`, `NUMBERP`, `BEFOREP`, `MEMBERP`, `SUBSTRINGP`, with their `?` forms.
- **Queries:** `MEMBER`, `ASCII`, `CHAR`, `LOWERCASE`, `UPPERCASE`.

## Done when

Each command has tests of its behaviour and its errors in `pendown-logo`, or the GUI and CLI tests cover it.

## Comments
