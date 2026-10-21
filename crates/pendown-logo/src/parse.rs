//! Turns read words into instructions, using the number of inputs each
//! procedure takes (ADR 0001).

use std::{collections::HashMap, ops::Range, rc::Rc};

use crate::{
    error::Error,
    names::{Names, Sym},
    number::{format_number, parse_number},
    prims::{self, PrimId},
    reader,
    span::{SourceId, Span},
    value::{List, Value},
};

/// How many inputs a procedure takes: `default` without parentheses, and
/// from `min` to `max` inside them, as in `(PRINT 1 2 3)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arity {
    pub min: usize,
    pub default: usize,
    pub max: Option<usize>,
}

impl Arity {
    pub const fn fixed(n: usize) -> Arity {
        Arity { min: n, default: n, max: Some(n) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Sum,
    Difference,
    Product,
    Quotient,
    Equal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    NotEqual,
}

impl Op {
    fn priority(self) -> u8 {
        match self {
            Op::Product | Op::Quotient => 3,
            Op::Sum | Op::Difference => 2,
            _ => 1,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Op::Sum => "+",
            Op::Difference => "-",
            Op::Product => "*",
            Op::Quotient => "/",
            Op::Equal => "=",
            Op::Less => "<",
            Op::Greater => ">",
            Op::LessEqual => "<=",
            Op::GreaterEqual => ">=",
            Op::NotEqual => "<>",
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Lit(Value, Span),
    Var(Sym, Span),
    /// Unary minus, which binds tighter than any infix operator.
    Neg(Box<Expr>, Span),
    Call(Box<Call>),
}

#[derive(Debug)]
pub struct Call {
    pub target: Target,
    /// The name as typed, for error messages.
    pub name: Rc<str>,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Prim(PrimId),
    /// Looked up when called, so redefining a procedure takes effect.
    User(Sym),
    Infix(Op),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Lit(_, span) | Expr::Var(_, span) | Expr::Neg(_, span) => *span,
            Expr::Call(call) => call.span,
        }
    }

    /// A Lisp-like rendering for tests, such as `(forward (- 5))`.
    pub fn sexp(&self, names: &Names) -> String {
        match self {
            Expr::Lit(Value::Word(w), _) if parse_number(w).is_none() => format!("\"{w}"),
            Expr::Lit(v, _) => v.show(),
            Expr::Var(sym, _) => format!(":{}", names.spelling(*sym)),
            Expr::Neg(e, _) => format!("(- {})", e.sexp(names)),
            Expr::Call(call) => {
                let name = match call.target {
                    Target::Prim(id) => prims::PRIMS[id].names[0].to_string(),
                    Target::User(sym) => names.spelling(sym).to_lowercase(),
                    Target::Infix(op) => prims::infix_prim(op).names[0].to_string(),
                };
                let args: Vec<_> = call.args.iter().map(|a| a.sexp(names)).collect();
                if args.is_empty() { format!("({name})") } else { format!("({name} {})", args.join(" ")) }
            }
        }
    }
}

#[derive(Debug)]
pub struct Instr {
    pub expr: Expr,
    pub span: Span,
}

pub type Block = Vec<Instr>;

pub struct ProcDef {
    pub sym: Sym,
    pub inputs: Vec<Sym>,
    pub body: Block,
    pub title: Span,
}

pub struct Program {
    pub procs: Vec<ProcDef>,
    pub top: Block,
}

/// What the parser needs to know about user procedures: their arities.
pub struct Cx<'a> {
    pub names: &'a mut Names,
    pub known: &'a dyn Fn(Sym) -> Option<Arity>,
    /// Procedures defined in the source being parsed, which take precedence.
    pub defining: HashMap<Sym, Arity>,
}

impl Cx<'_> {
    fn arity(&self, sym: Sym) -> Option<Arity> {
        self.defining.get(&sym).copied().or_else(|| (self.known)(sym))
    }
}

/// Parses a whole source. Every `TO` is found first, so a call can come
/// before its definition (ADRs 0001 and 0006). Errors are collected one per
/// line, so the editor can show them all.
pub fn parse_program(text: &str, source: SourceId, cx: &mut Cx) -> Result<Program, Vec<Error>> {
    let (program, errors) = parse_program_partly(text, source, cx);
    if errors.is_empty() { Ok(program) } else { Err(errors) }
}

/// Like `parse_program`, but also returns whatever parsed, alongside the
/// errors, for the editor's checks.
pub fn parse_program_partly(text: &str, source: SourceId, cx: &mut Cx) -> (Program, Vec<Error>) {
    let all = match reader::read(text, source) {
        Ok(all) => all,
        Err(e) => return (Program { procs: Vec::new(), top: Vec::new() }, vec![e]),
    };
    let items = &all.items;
    let spans = all.spans.as_ref().expect("source lists have spans");
    let mut errors = Vec::new();

    enum Seg {
        Top(Range<usize>),
        Def { sym: Sym, inputs: Vec<Sym>, body: Range<usize>, title: Span },
    }
    let starts_line = |i: usize| i == 0 || spans[i - 1].line != spans[i].line;
    let word = |i: usize| match &items[i] {
        Value::Word(w) => Some(&**w),
        _ => None,
    };
    let is = |i: usize, kw: &str| word(i).is_some_and(|w| w.eq_ignore_ascii_case(kw));

    // Pass 1: find the definitions, and the arity of each.
    let mut segs = Vec::new();
    let (mut i, mut top_start) = (0, 0);
    while i < items.len() {
        if is(i, "end") {
            errors.push(Error::new("END without a TO before it", Some(spans[i])));
            segs.push(Seg::Top(top_start..i));
            i += 1;
            top_start = i;
            continue;
        }
        if !(is(i, "to") && starts_line(i)) {
            i += 1;
            continue;
        }
        segs.push(Seg::Top(top_start..i));
        let line = spans[i].line;
        let on_line = |j: usize| j < items.len() && spans[j].line == line;
        let title_start = spans[i];
        let mut j = i + 1;

        // The title: TO, a name, and the inputs, all on one line.
        let name = if on_line(j) { word(j) } else { None };
        let mut title = title_start;
        let sym = match name {
            Some(name) if name.starts_with([':', '"']) || parse_number(name).is_some() => {
                errors.push(Error::new(format!("{name} can't be the name of a procedure"), Some(spans[j])));
                None
            }
            Some(name) if prims::lookup(name).is_some() => {
                errors.push(Error::new(
                    format!("{} is already a Pendown command, so pick another name", name.to_uppercase()),
                    Some(spans[j]),
                ));
                None
            }
            Some(name) => {
                let sym = cx.names.define(name);
                title = title.to(spans[j]);
                if cx.defining.contains_key(&sym) {
                    errors.push(Error::new(format!("{name} is defined twice"), Some(spans[j])));
                }
                Some(sym)
            }
            None => {
                errors.push(Error::new("TO needs the name of the new procedure", Some(title_start)));
                None
            }
        };
        if name.is_some() {
            j += 1;
        }
        let mut inputs = Vec::new();
        while on_line(j) && let Some(w) = word(j).and_then(|w| w.strip_prefix(':')) {
            inputs.push(cx.names.define(w));
            title = title.to(spans[j]);
            j += 1;
        }

        // The body runs to END: on its own line, or on the same line as TO.
        let body_start = j;
        let mut end = None;
        while j < items.len() {
            if is(j, "end") {
                end = Some(j);
                break;
            }
            if is(j, "to") && starts_line(j) {
                break;
            }
            j += 1;
        }
        match end {
            Some(end) => {
                if let Some(sym) = sym {
                    cx.defining.insert(sym, Arity::fixed(inputs.len()));
                    segs.push(Seg::Def { sym, inputs, body: body_start..end, title });
                }
                i = end + 1;
            }
            None => {
                let name = name.unwrap_or("TO").to_uppercase();
                errors.push(Error::new(format!("{name} has no END"), Some(title)));
                i = j;
            }
        }
        top_start = i;
    }
    segs.push(Seg::Top(top_start..items.len()));

    // Pass 2: parse every line, now that all the arities are known.
    let mut program = Program { procs: Vec::new(), top: Vec::new() };
    for seg in segs {
        match seg {
            Seg::Top(range) => {
                let tokens = runparse(&items[range.clone()], Some(&spans[range]), None);
                parse_lines(&tokens, cx, &mut program.top, &mut errors);
            }
            Seg::Def { sym, inputs, body, title } => {
                let tokens = runparse(&items[body.clone()], Some(&spans[body]), None);
                let mut block = Vec::new();
                parse_lines(&tokens, cx, &mut block, &mut errors);
                program.procs.push(ProcDef { sym, inputs, body: block, title });
            }
        }
    }
    (program, errors)
}

/// Parses a list as instructions, the first time it's run. A list written in
/// the source uses its words' spans. A list built at run time has none, so
/// its instructions all get `fallback`, the span of the instruction that ran
/// it (ADR 0001). Newlines inside a list don't end instructions.
pub fn parse_list(list: &List, fallback: Span, cx: &mut Cx) -> Result<Block, Error> {
    let tokens = runparse(&list.items, list.spans.as_deref(), Some(fallback));
    let mut parser = Parser { tokens: &tokens, pos: 0, cx, depth: 0 };
    let mut block = Vec::new();
    while parser.pos < tokens.len() {
        block.push(parser.instruction()?);
    }
    Ok(block)
}

/// Parses top-level or procedure-body tokens, where each line is parsed on
/// its own, as in UCBLogo: `FD` on one line and `50` on the next is "not
/// enough inputs". A line continues while a parenthesis is open.
fn parse_lines(tokens: &[Token], cx: &mut Cx, block: &mut Block, errors: &mut Vec<Error>) {
    let mut start = 0;
    while start < tokens.len() {
        let line = tokens[start].span.line;
        let mut end = start;
        let mut depth = 0i32;
        while end < tokens.len() && (tokens[end].span.line == line || depth > 0) {
            match tokens[end].tok {
                Tok::LParen => depth += 1,
                Tok::RParen => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        let mut parser = Parser { tokens: &tokens[start..end], pos: 0, cx, depth: 0 };
        while parser.pos < parser.tokens.len() {
            match parser.instruction() {
                Ok(instr) => block.push(instr),
                Err(e) => {
                    errors.push(e);
                    break;
                }
            }
        }
        start = end;
    }
}

#[derive(Debug)]
pub(crate) enum Tok {
    Lit(Value),
    Word(Rc<str>),
    Quoted(Rc<str>),
    Var(Rc<str>),
    Op(Op),
    /// A `-` at the start of a word with more after it, as in `-5` or `-:a`.
    Neg,
    LParen,
    RParen,
    List(Rc<List>),
}

#[derive(Debug)]
pub(crate) struct Token {
    pub(crate) tok: Tok,
    pub(crate) span: Span,
}

/// Splits read words into tokens, like UCBLogo's `runparse`.
fn runparse(items: &[Value], spans: Option<&[Span]>, fallback: Option<Span>) -> Vec<Token> {
    let mut out = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let span = spans.map(|s| s[i]);
        let whole = span.or(fallback).expect("either spans or a fallback");
        match item {
            Value::List(list) => out.push(Token { tok: Tok::List(list.clone()), span: whole }),
            Value::Num(n) => out.push(Token { tok: Tok::Lit(Value::Num(*n)), span: whole }),
            Value::Word(w) => split_word(w, span, whole, &mut out),
        }
    }
    out
}

fn is_paren(c: u8) -> bool {
    matches!(c, b'(' | b')')
}

fn is_infix(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'*' | b'/' | b'=' | b'<' | b'>')
}

pub(crate) fn split_word(text: &str, span: Option<Span>, whole: Span, out: &mut Vec<Token>) {
    let sub = |a: usize, b: usize| match span {
        Some(s) => Span { start: s.start + a as u32, end: s.start + b as u32, ..s },
        None => whole,
    };
    let b = text.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n {
        let start = i;
        let tok = match b[i] {
            b'"' => {
                i += 1;
                while i < n && !is_paren(b[i]) {
                    i += 1;
                }
                Tok::Quoted(text[start + 1..i].into())
            }
            b':' => {
                i += 1;
                while i < n && !is_paren(b[i]) && !is_infix(b[i]) {
                    i += 1;
                }
                Tok::Var(text[start + 1..i].into())
            }
            // A minus with a space before and no space after is unary.
            b'-' if i == 0 && n > 1 => {
                i += 1;
                Tok::Neg
            }
            b'(' => {
                i += 1;
                Tok::LParen
            }
            b')' => {
                i += 1;
                Tok::RParen
            }
            c if is_infix(c) => {
                let next = b.get(i + 1).copied();
                let (op, len) = match (c, next) {
                    (b'<', Some(b'=')) => (Op::LessEqual, 2),
                    (b'<', Some(b'>')) => (Op::NotEqual, 2),
                    (b'>', Some(b'=')) => (Op::GreaterEqual, 2),
                    (b'+', _) => (Op::Sum, 1),
                    (b'-', _) => (Op::Difference, 1),
                    (b'*', _) => (Op::Product, 1),
                    (b'/', _) => (Op::Quotient, 1),
                    (b'=', _) => (Op::Equal, 1),
                    (b'<', _) => (Op::Less, 1),
                    _ => (Op::Greater, 1),
                };
                i += len;
                Tok::Op(op)
            }
            _ => {
                // A word runs to the next parenthesis or operator, except
                // that a sign straight after a number's `e` is kept: `1e-3`.
                let mut after_e = false;
                let mut digits = false;
                while i < n && !is_paren(b[i]) && (!is_infix(b[i]) || (after_e && matches!(b[i], b'+' | b'-'))) {
                    after_e = digits && matches!(b[i], b'e' | b'E');
                    digits |= b[i].is_ascii_digit();
                    i += 1;
                }
                let w = &text[start..i];
                match parse_number(w) {
                    Some(x) if format_number(x) == w => Tok::Lit(Value::Num(x)),
                    Some(_) => Tok::Lit(Value::word(w)),
                    None => Tok::Word(w.into()),
                }
            }
        };
        out.push(Token { tok, span: sub(start, i) });
    }
}

/// Expressions nested deeper than this are an error, so the parser can't
/// overflow the stack on silly input like ten thousand `(`.
const MAX_NESTING: u32 = 1000;

struct Parser<'t, 'c, 'n> {
    tokens: &'t [Token],
    pos: usize,
    cx: &'c mut Cx<'n>,
    depth: u32,
}

impl Parser<'_, '_, '_> {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos).map(|t| &t.tok)
    }

    fn instruction(&mut self) -> Result<Instr, Error> {
        let expr = self.expr(1)?;
        Ok(Instr { span: expr.span(), expr })
    }

    /// Infix expressions by precedence climbing, left-associative.
    fn expr(&mut self, min_priority: u8) -> Result<Expr, Error> {
        self.depth += 1;
        if self.depth > MAX_NESTING {
            let span = self.tokens[self.pos.min(self.tokens.len() - 1)].span;
            return Err(Error::new("This expression is nested too deeply", Some(span)));
        }
        let mut lhs = self.unary()?;
        while let Some(&Tok::Op(op)) = self.peek()
            && op.priority() >= min_priority
        {
            self.pos += 1;
            let rhs = self.expr(op.priority() + 1)?;
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Call(Box::new(Call {
                target: Target::Infix(op),
                name: op.symbol().into(),
                args: vec![lhs, rhs],
                span,
            }));
        }
        self.depth -= 1;
        Ok(lhs)
    }

    /// A `-` where an input is expected is unary, whether or not there's a
    /// space after it, as in UCBLogo. It applies to the next operand only.
    fn unary(&mut self) -> Result<Expr, Error> {
        match self.peek() {
            Some(Tok::Neg | Tok::Op(Op::Difference)) => {
                let span = self.tokens[self.pos].span;
                self.pos += 1;
                let operand = self.unary()?;
                let span = span.to(operand.span());
                Ok(Expr::Neg(Box::new(operand), span))
            }
            Some(&Tok::Op(op)) => {
                let span = self.tokens[self.pos].span;
                Err(Error::new(format!("{} needs something before it", op.symbol()), Some(span)))
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let Some(token) = self.tokens.get(self.pos) else {
            let span = self.tokens.last().expect("not called on an empty line").span;
            return Err(Error::new("Expected an input here", Some(span)));
        };
        let span = token.span;
        self.pos += 1;
        match &token.tok {
            Tok::Lit(v) => Ok(Expr::Lit(v.clone(), span)),
            Tok::Quoted(w) => Ok(Expr::Lit(Value::Word(w.clone()), span)),
            Tok::List(list) => Ok(Expr::Lit(Value::List(list.clone()), span)),
            Tok::Var(name) if name.is_empty() => Err(Error::new("A variable name is missing after :", Some(span))),
            Tok::Var(name) => Ok(Expr::Var(self.cx.names.intern(name), span)),
            Tok::RParen => Err(Error::new("Unexpected )", Some(span))),
            Tok::Word(name) => {
                let name = name.clone();
                let (target, arity) = self.resolve(&name, span)?;
                let mut args = Vec::with_capacity(arity.default);
                for _ in 0..arity.default {
                    if matches!(self.peek(), None | Some(Tok::RParen)) {
                        let message = format!("Not enough inputs to {}", name.to_uppercase());
                        return Err(Error::new(message, Some(span)));
                    }
                    args.push(self.expr(1)?);
                }
                let span = args.last().map_or(span, |a| span.to(a.span()));
                Ok(Expr::Call(Box::new(Call { target, name, args, span })))
            }
            Tok::LParen => self.parenthesized(span),
            Tok::Neg | Tok::Op(_) => unreachable!("handled by unary"),
        }
    }

    /// `(` starts a call with any number of inputs, if a procedure name comes
    /// next, and otherwise a grouped expression.
    fn parenthesized(&mut self, open: Span) -> Result<Expr, Error> {
        let missing = |at: Span| Error::new("This ( has no matching )", Some(at));
        if let Some(Tok::Word(name)) = self.peek() {
            let name = name.clone();
            let name_span = self.tokens[self.pos].span;
            self.pos += 1;
            let (target, arity) = self.resolve(&name, name_span)?;
            let mut args = Vec::new();
            loop {
                match self.peek() {
                    None => return Err(missing(open)),
                    Some(Tok::RParen) => break,
                    _ => args.push(self.expr(1)?),
                }
            }
            let close = self.tokens[self.pos].span;
            self.pos += 1;
            let upper = name.to_uppercase();
            if args.len() < arity.min {
                return Err(Error::new(format!("Not enough inputs to {upper}"), Some(name_span)));
            }
            if arity.max.is_some_and(|max| args.len() > max) {
                return Err(Error::new(format!("Too many inputs to {upper}"), Some(open.to(close))));
            }
            return Ok(Expr::Call(Box::new(Call { target, name, args, span: open.to(close) })));
        }
        let expr = self.expr(1)?;
        match self.peek() {
            Some(Tok::RParen) => {
                self.pos += 1;
                Ok(expr)
            }
            None => Err(missing(open)),
            Some(_) => {
                let span = self.tokens[self.pos].span;
                Err(Error::new("Too much inside ( ): is an operator missing?", Some(span)))
            }
        }
    }

    fn resolve(&mut self, name: &str, span: Span) -> Result<(Target, Arity), Error> {
        if let Some(id) = prims::lookup(name) {
            return Ok((Target::Prim(id), prims::PRIMS[id].arity));
        }
        let sym = self.cx.names.intern(name);
        match self.cx.arity(sym) {
            Some(arity) => Ok((Target::User(sym), arity)),
            None => Err(Error::new(format!("I don't know how to {}", name.to_uppercase()), Some(span))),
        }
    }
}
