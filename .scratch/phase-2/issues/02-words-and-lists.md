# The rest of words and lists

Status: done
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

### 2026-10-21: done

In `prims/data.rs`. `POP` and `DEQUEUE` share code, since both take from the front. Tests: `tests/words.rs`.
