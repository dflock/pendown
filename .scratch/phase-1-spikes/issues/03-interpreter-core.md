# Spike: interpreter core

Status: done
Type: prototype

## Question

Do the parsing and evaluation decisions (ADRs 0001, 0002, 0004, 0005, 0006 and 0007) work in practice, and is the interpreter fast enough for Instant mode?

## Build

A `pendown-logo` library crate (ADR 0019), with no GUI code, that implements about ten commands: `FD`, `RT`, `REPEAT`, `TO`/`END`, `IF`, `STOP`, `OUTPUT`, `MAKE`, `PRINT`, plus infix arithmetic. It sends drawing events to a channel, as in `implementation-plan.md`. This spike may become the start of the real `pendown-logo` crate, so write it with that in mind.

## Answer these, with tests

1. **Endless tail recursion:** `TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END` runs for a long time with memory staying flat (ADR 0007). Also test tail calls inside `IF` and `IFELSE`, and mutual recursion.
2. **Deep recursion that isn't a tail call:** a tree fractal 12 levels deep, and recursive Fibonacci of 25, work without overflowing. Measure how many nested calls a 256 MB stack allows, and recommend the depth limit (ADR 0007). Going past the limit gives a friendly error, not a crash.
3. **Calls before definitions:** a call above its `TO` in the same file parses correctly (ADRs 0001 and 0006).
4. **Minus sign:** `FD -5`, `:a - 5`, `:a -5`, `:a-5`, `- 3 + 4` and `MINUS 3 + 4` all parse as ADR 0001 says.
5. **Spans:** every instruction carries the right span, including inside `REPEAT` lists and procedure bodies.
6. **Numbers:** printing and equality follow ADR 0002.
7. **Stop:** setting the cancel flag stops a running `FOREVER` promptly.
8. **Speed:** how many instructions per second for a simple drawing loop, in release mode?

## Done when

The tests pass, the questions are answered in a comment below with the measured speed, and any ADR changes are noted.

## Comments

### 2026-10-21: spike results

Code: `spikes/03-interpreter-core/`, a library crate already named `pendown-logo`, written to become the real one.

- **Tests:** `cargo test` runs 37 tests in `tests/questions.rs`, grouped by question. They pass in debug and release.
- **Measurements:** `cargo run --release --bin bench -- --memory` writes what's in `bench-release.log`. `cargo run --bin bench` does the same for a debug build (`bench-debug.log`).

How it's built:
- **`reader`** splits text into words and lists, like UCBLogo's `parser_iterate`. Only whitespace and brackets end a word, so `[a+b]` is a list of one word, as in UCBLogo.
- **`parse`** then splits words into tokens, like UCBLogo's `runparse`, and groups them by arity. It finds every `TO` first.
- **`interp`** is the tree-walking evaluator from ADR 0007. Primitives are a table in `prims` with about 30 commands: the ten the issue asked for, plus `IFELSE`, `FOREVER`, `REPCOUNT`, `LT`, `BK`, `SHOW`, `TYPE`, `WORD`, `THING` and the arithmetic and comparison operations.
- **Events** go to an `EventSink`: a channel, a `Vec`, or anything else. The crate has no GUI code.

**Summary: yes.** The design works as the ADRs describe, and it's fast enough: 9–12 million instructions a second through the channel to the GUI, about ten times what spike 02 can draw. There are three changes to make:
1. Send events to the GUI in batches, not one at a time.
2. Set the depth limit to 10,000, with a stack guard behind it.
3. Build `pendown-logo` at `opt-level = 1` even in debug builds.

#### 1. Endless tail recursion

- **`SPIRAL` runs in constant space.**
  - Written on one line exactly as in the issue, it ran a million moves on a 1 MiB stack with a depth limit of 50 and was never more than one call deep.
  - In the 20 s memory run it did 56 M instructions. Resident memory stayed at 4.9 MiB the whole time.
- **Tail calls work in every position ADR 0007 lists,** each tested a million calls deep on that 1 MiB stack:
  - inside `IF`, `IFELSE` and the three-input `(IF tf [..] [..])`;
  - mutual recursion (`PING`/`PONG`);
  - `OUTPUT COUNTUP :n - 1 :acc + 1`;
  - `OUTPUT IFELSE :n = 0 [:acc] [COUNTUP ...]`.
- **Calls that aren't tail calls still nest,** as they should: a call with an instruction after it, and a call inside a `REPEAT` list, hit the depth limit.
- **Dynamic scope survives tail calls.** A procedure reached by a tail call still sees its caller's inputs, and the caller's variables come back afterwards.
- **Variables use shallow binding.** ADR 0007's frames are kept, but instead of walking them on every lookup:
  - each name has one slot holding its current value;
  - a frame records the values its inputs replaced, and puts them back when it ends.
  
  The results are the same as walking the frames, but every lookup is an array index. A variable inspector can rebuild the chain of frames from the saved values.
- **A tail call whose output has nowhere to go is an error.** If `TO F G END` and `G` outputs 3, running `F` gives "You don't say what to do with 3", as in UCBLogo.

#### 2. Deep recursion that isn't a tail call

- **`TREE 100 12`** draws its 8,190 moves 13 calls deep, and **`FIB 25`** prints 75025.
- **Stack used per call,** measured from the stack pointer, for four kinds of recursion:

  | Recursion | Release | Debug, unoptimised | Debug, `opt-level = 1` |
  |---|---|---|---|
  | A call, then another instruction | 1.3 KB | 9.3 KB | 1.4 KB |
  | Inside an expression (`OUTPUT 1 + E :n - 1`) | 3.5 KB | 30.7 KB | 3.8 KB |
  | Inside `REPEAT` and `IFELSE` lists | 4.8 KB | 42.9 KB | 5.4 KB |
  | **Calls that fit in 256 MiB, worst case** | **56,000** | **6,200** | **49,600** |

- **Recommendation: a depth limit of 10,000, with a stack guard behind it.**
  - 10,000 is far deeper than drawing programs go, and leaves five times headroom in release.
  - The guard checks how much stack is left at every call and every list parse. When less than 256 KiB is left, it gives the same friendly error, whatever the depth limit says. That covers kinds of recursion I haven't measured, and debug builds.
  - The tests run an endless recursion into both the limit and the guard; neither crashes.
  - Errors read: "DOWN called itself too many times: 10001 calls deep. Does it have a way to STOP?". Mutual recursion gives "Too many procedures were running inside each other (10001 deep) when A was called".
- **Build `pendown-logo` at `opt-level = 1` in the dev profile.** Unoptimised, each call uses nine times the stack, so the guard would stop at about 6,000 calls and debug builds would behave differently from release. The spike's `Cargo.toml` does this.
- **The 256 MiB stack is reserved address space.** Only the pages actually touched take memory, so a large stack costs nothing until a program recurses deeply.

#### 3. Calls before definitions

- **They work.**
  - `SQUARE 10` above its `TO` draws the square.
  - Definitions can call each other in any order.
  - A later `TO ADD :a :b` makes `PRINT ADD 1 2` parse as `(print (add 1 2))`.
- **Parse errors are collected, one per line, and nothing runs.** A run with an `END` missing, `FD` with no input and an unknown `FOO` reports all three.
- **Run starts from a clean slate, and the console keeps what the last run left (ADR 0006).**
  - After a run defines `SQUARE`, the console can call `SQUARE :size`.
  - The next Run clears it, so the console gets "I don't know how to SQUARE".
- **One-line definitions work too.** `TO SPIRAL :n FD :n RT 90 SPIRAL :n + 1 END` is the issue's own example.
  - UCBLogo needs `END` on a line of its own. Here, `TO` must still start a line, but any bare `END` ends the body.
  - This needs recording in ADR 0001.

#### 4. The minus sign

Every case parses as ADR 0001 says. The tests check each case's parse, and the values too:

| Source | Parses as | Value |
|---|---|---|
| `FD -5` | `(forward (- 5))` | |
| `PRINT :a - 5`, `PRINT :a-5` | `(print (difference :a 5))` | 5, with `:a` 10 |
| `SUM :a -5` | `(sum :a (- 5))`: two inputs | 5 |
| `PRINT :a -5` | `(print :a)`, then `(- 5)`: PRINT takes one input | |
| `PRINT - 3 + 4` | `(print (sum (- 3) 4))` | 1 |
| `PRINT MINUS 3 + 4` | `(print (minus (sum 3 4)))` | -7 |
| `PRINT 2*-3` | `(print (product 2 (- 3)))` | |
| `PRINT 1e-3 + 1` | `(print (sum 1e-3 1))`: the exponent's `-` stays in the number | |
| `FD - 5` | `(forward (- 5))` | |

`FD - 5` needs a line in ADR 0001. By ADR 0001's rule, its `-` (spaces on both sides, after a word) is infix, which would leave `FD` with nothing to subtract from. UCBLogo's parser (`paren.c`) treats a `-` wherever an input is expected as unary, and so does the spike. `SUM :a - 5` gives "Not enough inputs to SUM", because `:a - 5` is SUM's first input.

#### 5. Spans

- **Every instruction carries its span.**
  - For `SQUARE 10 PRINT "hi`, where `SQUARE` is `REPEAT 2 [FD :n RT 90]`, the `Instruction` events cover exactly:
    - `SQUARE 10`;
    - then `REPEAT 2 [FD :n RT 90]`;
    - then `FD :n` and `RT 90`, twice;
    - then `PRINT "hi`.
  - Infix expressions and one-line definitions are covered too.
- **Lists from the source carry their words' spans.** A list's instructions are parsed when it first runs, with the words' own spans, and the parse is cached. Errors inside a list point at the word, such as `FOO` in `REPEAT 2 [FD 10 FOO]`.
- **Lists built while a program runs have no spans.** They're parsed each time they run, and use the span of the instruction that ran them (ADR 0001).
- **Errors carry spans and the chain of calls (ADR 0005).** `FD "far` inside `B`, called from `A`, called from line 9, gives:
  - the message "FD doesn't like far as input";
  - a span covering `"far`;
  - the chain "in B (line 2), called from A (line 9)".
  
  Parse errors point at the token: `RT` for "Not enough inputs to RT", the `(` that has no `)`, and the `[` that has no `]`.
- **`starts_line`** is set when an instruction runs from a different line than the last one. So:
  - a one-line `REPEAT 4 [FD 50 RT 90]` is one step;
  - a `REPEAT` list spread over several lines steps through its lines.
  
  One effect for spike 04: after `SQUARE 10` returns, the rest of its line (`PRINT "hi`) starts a new step, because the last line run was in `SQUARE`'s body.
- **Each line parses separately,** as in UCBLogo. `FD` on one line and `50` on the next is "Not enough inputs to FD". A line continues while a parenthesis or bracket is open.
- **`TO`'s title line has a span,** so the GUI can jump from a call to its definition.

#### 6. Numbers

Printing follows C's `%.15g`, which is what UCBLogo does. Every row is a test:

| `PRINT` | Shows |
|---|---|
| `4/2` | `2` |
| `3/2`, `-7/2` | `1.5`, `-3.5` |
| `0.1 + 0.2` | `0.3` |
| `1/3`, `2/3` | `0.333333333333333`, `0.666666666666667` |
| `123456789012345` | `123456789012345` |
| `2 * 4503599627370496` (2^53) | `9.00719925474099e+15` |
| `1e20 * 1`, `1/100000` | `1e+20`, `1e-05` |
| `0 * -1` | `0`, not `-0` |
| `1.50` | `1.50`: a literal not in its printed form stays the word as typed |
| `1.50 + 0` | `1.5` |
| `WORD 1 2`, `(WORD 1 2) + 1` | `12`, `13` |

- **Equality works as ADR 0002 says.** `2 = 2.0`, `"2.0 = 2` and `"ABC = "abc` are all true.
- **Errors:**
  - `1 / 0` gives "Can't divide by zero", and `1e300 * 1e300` gives "That number is too big".
  - `REPEAT 2.5` gives "REPEAT doesn't like 2.5 as input", and `"abc + 1` gives "+ doesn't like abc as input".
- **Fast number literals:** a literal typed in its printed form (`10`, `1.5`) is stored as a number when it's parsed, so `FD 10` doesn't convert text in a loop.
- **ADR 0002 needs three small corrections:**
  - Whole numbers of 1e15 or more print in exponent form, as `%.15g` does: 2^53 prints as `9.00719925474099e+15`.
  - Numbers below 1e-4 also use exponent form, so 1e-5 prints as `1e-05`.
  - Its example `WORD 1.50 ""` should be `WORD 1.50 "`, because `""` is the word `"`.
- **A decision for a human: `0.1 + 0.2 = 0.3` is false.** It prints `0.3`, but the two numbers differ in the 17th digit. UCBLogo gives false too. Children will hit this, for example when stepping by 0.1. There are two options:
  - Keep it, as UCBLogo does.
  - Compare numbers rounded to 15 significant digits, so that two numbers are equal exactly when they print the same. `<` and `>` would round the same way, to stay consistent.
  
  I'd round: what you see is what's equal.

#### 7. Stop

- **Setting the cancel flag stops `FOREVER [FD 1 RT 1]` within 50–80 µs.** The flag is checked before every instruction, and the release test stopped it after about 2.2 M events.
- **A blocked interpreter wakes up too.** When the animator is paused and the channel is full, dropping the receiver makes the blocked send fail, and the run ends with `Halted` (under 50 ms in the test).
- **Stop is its own kind of result,** `Flow::Halt`, separate from errors, so a later `CATCH` can't intercept it. `STOP` and `OUTPUT` at top level are friendly errors.

#### 8. Speed

These are release-build figures. Each loop runs 4 M instructions. "With events" sends an `Instruction` event before every instruction, as the GUI needs:

| Program | Events go nowhere | One channel send per event | Batches of 256 |
|---|---|---|---|
| `REPEAT 2000000 [FD 1 RT 1]` | 21 M/s | 5.3 M/s | **12 M/s** (24 M events/s) |
| Tail-recursive loop: `IF`, `FD`, `RT`, a call | 12 M/s | 1.8 M/s | **9.4 M/s** (14 M events/s) |
| `FIB 25`: 243,000 calls | 6.6 M/s | 1.7 M/s | **6.0 M/s** (3 M calls/s) |

- **The interpreter is fast enough for Instant mode.** A drawing loop makes several million segments a second. That's far more than the GUI can draw: spike 02 found about 1 µs per segment with vello_cpu, about 1 M segments a second. Instant mode will be limited by drawing, and the bounded channel will hold the interpreter back to match.
- **Sending one event at a time costs too much.**
  - A bounded `std::sync::mpsc` channel manages 2–11 M events a second.
  - At that rate, the channel costs more than everything else the interpreter does, and the GUI thread pays for it too.
  - Sending `Vec<Event>` batches of 256, over a channel holding 4 batches, makes runs 2.5–5 times faster.
  - `BatchSender` sends a batch when it's full, when it has been open for 4 ms, and when the run ends.
- **Batching has one consequence for spike 04.** Every sync point must flush the batch first: reading input, the end of the program, and each line while stepping. Otherwise the animator can't reach the `Sync` it's waiting for.
- A program that computes for a long time without drawing can hold up to 15 events past the 4 ms, because the clock is only checked every 16 events. A check every few hundred instructions in the evaluator would fix that, if it matters.

#### Changes this suggests

- **ADR 0001:**
  - Any bare `END` ends a definition, so one-line `TO ... END` works.
  - A `-` wherever an input is expected is unary (`FD - 5`), as in UCBLogo.
  - Each line parses on its own unless a bracket or parenthesis is open.
- **ADR 0002:**
  - Large whole numbers and numbers below 1e-4 print in exponent form, as `%.15g` does.
  - Fix the `WORD 1.50 "` example.
  - Record the decision on `0.1 + 0.2 = 0.3`.
- **ADR 0007:**
  - Variables use shallow binding.
  - The depth limit is 10,000, with a stack guard that gives the same error when fewer than 256 KiB are left.
  - `pendown-logo` builds at `opt-level = 1` in the dev profile.
  - `OUTPUT IFELSE ...` puts the chosen list in tail position.
  - A tail call written as an instruction can't output.
- **ADR 0009 and the plan's "Channel size" question:** events go in batches of 256, at most 4 ms old, over a channel of 4 batches. Sync points flush first.
- **ADR 0005:** a call chain from a deep recursion has thousands of entries, all the same. The GUI should collapse repeats, such as "called from DOWN, 10,000 times". Tail calls leave no entry, as in UCBLogo.
- **ADR 0004:** the spike keeps the first spelling seen anywhere, which could be a `:x` reference rather than a `TO` or `MAKE`. That's easy to fix in the real crate.

#### For a human to decide

- Should `0.1 + 0.2 = 0.3` be true? The suggestion is yes: compare numbers rounded to 15 significant digits.
- Is a depth limit of 10,000 right?
- Should one-line `TO ... END` be accepted, as here?

#### Not covered

- Turtle geometry beyond `FD`/`RT` (`WRAP` and `FENCE`, arcs, shapes), `LOCAL`, list primitives and colours.
- UCBLogo's `|vertical bars|`, `~` line continuation and backslash escapes.
- Pause and Step: spike 04.
- **The editor's static parse (ADR 0001)** only checks top-level lines and procedure bodies, because lists are parsed when they run. So `REPEAT 4 [FD]` is only an error at run time. The editor should also try parsing any list literal that looks like instructions, to show errors while the user types.

### 2026-10-21: closed

Decisions:
- **`0.1 + 0.2 = 0.3` is true.** Numbers compare rounded to 15 significant digits, and `<`, `>`, `<=`, `>=` and `<>` round the same way. This is now in the spike, with tests, and whole numbers below 2^53 still compare exactly, so the loops ran no slower. 39 tests pass.
- **A depth limit of 10,000 is good for now.**
- **Both forms of `TO ... END` are accepted.** The spike already did this, and a test now mixes both in one program.

Updated:
- ADR 0001: words and tokens, line-by-line parsing, `TO`/`END` forms, unary minus where an input is expected.
- ADR 0002: `%.15g` printing, rounded comparison, the `WORD 1.50 "` example.
- ADR 0005: tail calls and long call chains.
- ADR 0007: tail positions, shallow binding, the limit, the stack guard and `opt-level = 1`.
- ADR 0009 and `implementation-plan.md`: batched events, which answers the plan's "Channel size" question.
- `pendown-logo.kdl`: `differs` notes on `TO` and the six comparisons.

ADR 0004's display spelling needs no change; the real crate should follow it, rather than the spike.
