//! A closed subset of ClojureScript for an advanced query's `:result-transform` and
//! `:view`, run by a tree-walking interpreter with a step limit and a depth limit.
//!
//! The language has anonymous functions, `if`, `let`, `and`, `or`, `when`, the values of
//! EDN, and a fixed table of functions ([`BUILTINS`]). It has no definitions, loops, named
//! functions, interop, regular expressions, files, processes, clock or randomness, so a
//! function can only walk the finite collections it is given. Self-application can still
//! recurse, and the step and depth limits stop it. Every symbol outside the table is an
//! unsupported-form error at read time, so a script that reads can fail at run time only
//! through a thrown error or a limit.
use std::cmp::Ordering;
use std::fmt::{self, Write as _};
use std::rc::Rc;

use crate::query::{QueryError, QueryErrorKind};
use crate::query_form::{Form, FormKind};
use crate::query_page::Row;

/// Evaluation steps before a query stops.
pub const STEP_LIMIT: u64 = 20_000_000;
/// Nested function calls before a query stops.
pub const DEPTH_LIMIT: usize = 64;

/// A value in the closed language.
#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    Str(Rc<str>),
    Keyword(Rc<str>),
    Vector(Rc<Vec<Value>>),
    /// A sequence, as `map` and `concat` return.
    List(Rc<Vec<Value>>),
    Map(Rc<Vec<(Value, Value)>>),
    /// A set, in the order its members were added.
    Set(Rc<Vec<Value>>),
    /// A published page, with `:block/name`, `:block/original-name` and
    /// `:block/properties`.
    Row(Rc<Row>),
    Fn(Rc<Closure>),
    Builtin(Builtin),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.pr_str())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        equal(self, other)
    }
}

impl Value {
    pub fn str(text: &str) -> Value {
        Value::Str(Rc::from(text))
    }

    pub fn keyword(name: &str) -> Value {
        Value::Keyword(Rc::from(name))
    }

    pub fn vector(items: Vec<Value>) -> Value {
        Value::Vector(Rc::new(items))
    }

    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }

    /// `str` of one value: `nil` prints as nothing.
    pub fn to_text(&self) -> String {
        match self {
            Value::Nil => String::new(),
            Value::Str(s) => s.to_string(),
            _ => self.pr_str(),
        }
    }

    /// The printed form, as `pr-str` shows it.
    pub fn pr_str(&self) -> String {
        match self {
            Value::Nil => "nil".into(),
            Value::Bool(b) => b.to_string(),
            Value::Num(n) => number_text(*n),
            Value::Str(s) => format!("{s:?}"),
            Value::Keyword(k) => format!(":{k}"),
            Value::Vector(items) => format!("[{}]", join_pr(items)),
            Value::List(items) => format!("({})", join_pr(items)),
            Value::Set(items) => format!("#{{{}}}", join_pr(items)),
            Value::Map(pairs) => {
                let mut out = String::from("{");
                for (position, (k, v)) in pairs.iter().enumerate() {
                    if position > 0 {
                        out.push_str(", ");
                    }
                    let _ = write!(out, "{} {}", k.pr_str(), v.pr_str());
                }
                out.push('}');
                out
            }
            Value::Row(row) => format!("{{:block/name {:?}}}", row.name),
            Value::Fn(_) | Value::Builtin(_) => "#<Function>".into(),
        }
    }
}

fn join_pr(items: &[Value]) -> String {
    items
        .iter()
        .map(Value::pr_str)
        .collect::<Vec<_>>()
        .join(" ")
}

/// A number as JavaScript prints it: whole numbers without a decimal point.
pub fn number_text(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Why a run stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Thrown {
    /// An error the function raised, as ClojureScript would.
    Error(String),
    Limit(Limit),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Steps,
    Depth,
}

impl fmt::Display for Thrown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Thrown::Error(message) => f.write_str(message),
            Thrown::Limit(Limit::Steps) => {
                write!(f, "the query reached the limit of {STEP_LIMIT} steps")
            }
            Thrown::Limit(Limit::Depth) => {
                write!(
                    f,
                    "the query reached the limit of {DEPTH_LIMIT} nested calls"
                )
            }
        }
    }
}

fn error<T>(message: impl Into<String>) -> Result<T, Thrown> {
    Err(Thrown::Error(message.into()))
}

/// Counts the work of one query: every evaluated form, every page-clause pair and every
/// element a function builds. Two runs over the same input count the same steps.
#[derive(Debug)]
pub struct Meter {
    steps: u64,
    limit: u64,
    depth: usize,
}

impl Default for Meter {
    fn default() -> Self {
        Self::with_limit(STEP_LIMIT)
    }
}

impl Meter {
    pub fn with_limit(limit: u64) -> Self {
        Self {
            steps: 0,
            limit,
            depth: 0,
        }
    }

    pub fn steps(&self) -> u64 {
        self.steps
    }

    pub fn charge(&mut self, n: u64) -> Result<(), Thrown> {
        self.steps = self.steps.saturating_add(n);
        if self.steps > self.limit {
            Err(Thrown::Limit(Limit::Steps))
        } else {
            Ok(())
        }
    }

    fn enter(&mut self) -> Result<(), Thrown> {
        if self.depth >= DEPTH_LIMIT {
            return Err(Thrown::Limit(Limit::Depth));
        }
        self.depth += 1;
        Ok(())
    }

    fn leave(&mut self) {
        self.depth -= 1;
    }
}

// ---------------------------------------------------------------------------------------
// Compiled form

#[derive(Debug)]
pub enum Expr {
    Lit(Value),
    Local(Rc<str>),
    Builtin(Builtin),
    Vector(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Set(Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    When(Box<Expr>, Vec<Expr>),
    Let(Vec<(Rc<str>, Expr)>, Vec<Expr>),
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Fn(Rc<FnDef>),
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Debug)]
pub struct FnDef {
    params: Vec<Rc<str>>,
    body: Vec<Expr>,
}

pub struct Closure {
    def: Rc<FnDef>,
    env: Env,
}

/// Names bound by `let` and function parameters, as a chain that closures share.
#[derive(Clone, Default)]
pub struct Env(Option<Rc<EnvNode>>);

struct EnvNode {
    name: Rc<str>,
    value: Value,
    parent: Env,
}

impl Env {
    fn bind(&self, name: Rc<str>, value: Value) -> Env {
        Env(Some(Rc::new(EnvNode {
            name,
            value,
            parent: self.clone(),
        })))
    }

    fn get(&self, name: &str) -> Option<&Value> {
        let mut node = self.0.as_deref();
        while let Some(current) = node {
            if &*current.name == name {
                return Some(&current.value);
            }
            node = current.parent.0.as_deref();
        }
        None
    }
}

/// A function the language provides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Get,
    GetIn,
    Contains,
    Count,
    First,
    Last,
    SortBy,
    Sort,
    Compare,
    Reverse,
    Take,
    Drop,
    Into,
    Vec,
    Vector,
    Concat,
    Map,
    Filter,
    Remove,
    Str,
    Eq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    Not,
    Replace,
    Join,
    LowerCase,
    UpperCase,
}

/// The functions of the language by the symbol that names them. A garden asks for another
/// by adding a row here and a test.
pub const BUILTINS: &[(&str, Builtin)] = &[
    ("get", Builtin::Get),
    ("get-in", Builtin::GetIn),
    ("contains?", Builtin::Contains),
    ("count", Builtin::Count),
    ("first", Builtin::First),
    ("last", Builtin::Last),
    ("sort-by", Builtin::SortBy),
    ("sort", Builtin::Sort),
    ("compare", Builtin::Compare),
    ("reverse", Builtin::Reverse),
    ("take", Builtin::Take),
    ("drop", Builtin::Drop),
    ("into", Builtin::Into),
    ("vec", Builtin::Vec),
    ("vector", Builtin::Vector),
    ("concat", Builtin::Concat),
    ("map", Builtin::Map),
    ("filter", Builtin::Filter),
    ("remove", Builtin::Remove),
    ("str", Builtin::Str),
    ("=", Builtin::Eq),
    ("not=", Builtin::NotEq),
    ("<", Builtin::Lt),
    (">", Builtin::Gt),
    ("<=", Builtin::Le),
    (">=", Builtin::Ge),
    ("not", Builtin::Not),
    ("clojure.string/replace", Builtin::Replace),
    ("clojure.string/join", Builtin::Join),
    ("clojure.string/lower-case", Builtin::LowerCase),
    ("clojure.string/upper-case", Builtin::UpperCase),
];

pub fn builtin(name: &str) -> Option<Builtin> {
    BUILTINS
        .iter()
        .find(|(symbol, _)| *symbol == name)
        .map(|(_, builtin)| *builtin)
}

/// A function the author wrote, ready to run.
#[derive(Clone, Debug)]
pub struct Script {
    def: Rc<FnDef>,
}

impl PartialEq for Script {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.def, &other.def)
    }
}

impl Script {
    /// Call the function with `args`.
    pub fn call(&self, args: Vec<Value>, meter: &mut Meter) -> Result<Value, Thrown> {
        let closure = Closure {
            def: self.def.clone(),
            env: Env::default(),
        };
        call_closure(&closure, args, meter)
    }
}

fn unsupported(form: &Form) -> QueryError {
    unsupported_name(form.name(), form.offset)
}

fn unsupported_name(name: String, offset: usize) -> QueryError {
    QueryError {
        kind: QueryErrorKind::Unsupported(name),
        offset,
    }
}

fn syntax(message: impl Into<String>, offset: usize) -> QueryError {
    QueryError {
        kind: QueryErrorKind::Syntax(message.into()),
        offset,
    }
}

/// The attributes a page row has.
pub const ATTRIBUTES: [&str; 3] = ["block/name", "block/original-name", "block/properties"];

/// Read a function: `(fn [params] body…)` in the closed language.
pub fn parse_script(form: &Form) -> Result<Script, QueryError> {
    let FormKind::List(items) = &form.kind else {
        return Err(unsupported(form));
    };
    if !matches!(items.first().map(|f| &f.kind), Some(FormKind::Symbol(s)) if s == "fn") {
        return Err(unsupported(form));
    }
    let mut scope = Vec::new();
    match compile_fn(items, form, &mut scope)? {
        Expr::Fn(def) => Ok(Script { def }),
        _ => unreachable!("compile_fn returns a function"),
    }
}

fn compile_fn(items: &[Form], form: &Form, scope: &mut Vec<Rc<str>>) -> Result<Expr, QueryError> {
    let Some(params) = items.get(1) else {
        return Err(syntax("fn needs a parameter vector", form.offset));
    };
    let FormKind::Vector(names) = &params.kind else {
        // `(fn name [..])` and multi-arity forms are outside the language.
        return Err(unsupported_name(
            format!("fn {}", params.name()),
            params.offset,
        ));
    };
    let mut bound = Vec::new();
    for name in names {
        match &name.kind {
            FormKind::Symbol(symbol) if symbol != "&" => bound.push(Rc::from(symbol.as_str())),
            _ => return Err(unsupported(name)),
        }
    }
    let depth = scope.len();
    scope.extend(bound.iter().cloned());
    let body = items[2..]
        .iter()
        .map(|item| compile(item, scope))
        .collect::<Result<Vec<_>, _>>();
    scope.truncate(depth);
    Ok(Expr::Fn(Rc::new(FnDef {
        params: bound,
        body: body?,
    })))
}

pub fn compile(form: &Form, scope: &mut Vec<Rc<str>>) -> Result<Expr, QueryError> {
    Ok(match &form.kind {
        FormKind::Nil => Expr::Lit(Value::Nil),
        FormKind::Bool(b) => Expr::Lit(Value::Bool(*b)),
        FormKind::Num(n) => Expr::Lit(Value::Num(*n)),
        FormKind::Str(s) => Expr::Lit(Value::str(s)),
        FormKind::Keyword(k) => {
            if let Some((namespace, _)) = k.split_once('/')
                && matches!(namespace, "block" | "db")
                && !ATTRIBUTES.contains(&k.as_str())
            {
                return Err(unsupported(form));
            }
            Expr::Lit(Value::keyword(k))
        }
        FormKind::Symbol(symbol) => {
            if scope.iter().rev().any(|name| &**name == symbol) {
                Expr::Local(Rc::from(symbol.as_str()))
            } else if let Some(builtin) = builtin(symbol) {
                Expr::Builtin(builtin)
            } else {
                return Err(unsupported(form));
            }
        }
        FormKind::Vector(items) => Expr::Vector(compile_all(items, scope)?),
        FormKind::Set(items) => Expr::Set(compile_all(items, scope)?),
        FormKind::Map(pairs) => Expr::Map(
            pairs
                .iter()
                .map(|(k, v)| Ok((compile(k, scope)?, compile(v, scope)?)))
                .collect::<Result<_, QueryError>>()?,
        ),
        FormKind::Other(_) => return Err(unsupported(form)),
        FormKind::List(items) => compile_list(items, form, scope)?,
    })
}

fn compile_all(items: &[Form], scope: &mut Vec<Rc<str>>) -> Result<Vec<Expr>, QueryError> {
    items.iter().map(|item| compile(item, scope)).collect()
}

fn compile_list(items: &[Form], form: &Form, scope: &mut Vec<Rc<str>>) -> Result<Expr, QueryError> {
    let Some(head) = items.first() else {
        return Ok(Expr::Lit(Value::List(Rc::new(Vec::new()))));
    };
    let args = &items[1..];
    if let FormKind::Symbol(symbol) = &head.kind
        && !scope.iter().rev().any(|name| &**name == symbol)
    {
        match symbol.as_str() {
            "fn" => return compile_fn(items, form, scope),
            "if" => {
                if !(2..=3).contains(&args.len()) {
                    return Err(syntax("if takes two or three forms", form.offset));
                }
                return Ok(Expr::If(
                    Box::new(compile(&args[0], scope)?),
                    Box::new(compile(&args[1], scope)?),
                    args.get(2)
                        .map(|form| compile(form, scope).map(Box::new))
                        .transpose()?,
                ));
            }
            "when" => {
                let Some(test) = args.first() else {
                    return Err(syntax("when needs a test", form.offset));
                };
                return Ok(Expr::When(
                    Box::new(compile(test, scope)?),
                    compile_all(&args[1..], scope)?,
                ));
            }
            "and" => return Ok(Expr::And(compile_all(args, scope)?)),
            "or" => return Ok(Expr::Or(compile_all(args, scope)?)),
            "let" => return compile_let(args, form, scope),
            _ => {}
        }
    }
    Ok(Expr::Call(
        Box::new(compile(head, scope)?),
        compile_all(args, scope)?,
    ))
}

fn compile_let(args: &[Form], form: &Form, scope: &mut Vec<Rc<str>>) -> Result<Expr, QueryError> {
    let Some(Form {
        kind: FormKind::Vector(bindings),
        ..
    }) = args.first()
    else {
        return Err(syntax("let needs a binding vector", form.offset));
    };
    if bindings.len() % 2 != 0 {
        return Err(syntax("let needs a value for every name", form.offset));
    }
    let depth = scope.len();
    let mut compiled = Vec::new();
    let mut failed = None;
    for pair in bindings.chunks(2) {
        let name = match &pair[0].kind {
            FormKind::Symbol(symbol) => Rc::<str>::from(symbol.as_str()),
            _ => {
                failed = Some(unsupported(&pair[0]));
                break;
            }
        };
        match compile(&pair[1], scope) {
            Ok(value) => {
                scope.push(name.clone());
                compiled.push((name, value));
            }
            Err(e) => {
                failed = Some(e);
                break;
            }
        }
    }
    let body = if failed.is_none() {
        compile_all(&args[1..], scope)
    } else {
        Ok(Vec::new())
    };
    scope.truncate(depth);
    if let Some(e) = failed {
        return Err(e);
    }
    Ok(Expr::Let(compiled, body?))
}

// ---------------------------------------------------------------------------------------
// Evaluation

fn call_closure(closure: &Closure, args: Vec<Value>, meter: &mut Meter) -> Result<Value, Thrown> {
    if args.len() != closure.def.params.len() {
        return error(format!(
            "Wrong number of args ({}) passed to a function of {}",
            args.len(),
            closure.def.params.len()
        ));
    }
    meter.enter()?;
    let mut env = closure.env.clone();
    for (name, value) in closure.def.params.iter().zip(args) {
        env = env.bind(name.clone(), value);
    }
    let result = eval_body(&closure.def.body, &env, meter);
    meter.leave();
    result
}

fn eval_body(body: &[Expr], env: &Env, meter: &mut Meter) -> Result<Value, Thrown> {
    let mut last = Value::Nil;
    for expr in body {
        last = eval(expr, env, meter)?;
    }
    Ok(last)
}

pub fn eval(expr: &Expr, env: &Env, meter: &mut Meter) -> Result<Value, Thrown> {
    meter.charge(1)?;
    match expr {
        Expr::Lit(value) => Ok(value.clone()),
        Expr::Local(name) => Ok(env.get(name).cloned().unwrap_or(Value::Nil)),
        Expr::Builtin(builtin) => Ok(Value::Builtin(*builtin)),
        Expr::Vector(items) => {
            let values = items
                .iter()
                .map(|item| eval(item, env, meter))
                .collect::<Result<_, _>>()?;
            Ok(Value::vector(values))
        }
        Expr::Set(items) => {
            let mut set: Vec<Value> = Vec::new();
            for item in items {
                let value = eval(item, env, meter)?;
                if !set.iter().any(|member| equal(member, &value)) {
                    set.push(value);
                }
            }
            Ok(Value::Set(Rc::new(set)))
        }
        Expr::Map(pairs) => {
            let mut map: Vec<(Value, Value)> = Vec::new();
            for (k, v) in pairs {
                let key = eval(k, env, meter)?;
                let value = eval(v, env, meter)?;
                map_insert(&mut map, key, value);
            }
            Ok(Value::Map(Rc::new(map)))
        }
        Expr::If(test, then, otherwise) => {
            if eval(test, env, meter)?.truthy() {
                eval(then, env, meter)
            } else if let Some(otherwise) = otherwise {
                eval(otherwise, env, meter)
            } else {
                Ok(Value::Nil)
            }
        }
        Expr::When(test, body) => {
            if eval(test, env, meter)?.truthy() {
                eval_body(body, env, meter)
            } else {
                Ok(Value::Nil)
            }
        }
        Expr::And(items) => {
            let mut last = Value::Bool(true);
            for item in items {
                last = eval(item, env, meter)?;
                if !last.truthy() {
                    break;
                }
            }
            Ok(last)
        }
        Expr::Or(items) => {
            let mut last = Value::Nil;
            for item in items {
                last = eval(item, env, meter)?;
                if last.truthy() {
                    break;
                }
            }
            Ok(last)
        }
        Expr::Let(bindings, body) => {
            let mut env = env.clone();
            for (name, value) in bindings {
                let value = eval(value, &env, meter)?;
                env = env.bind(name.clone(), value);
            }
            eval_body(body, &env, meter)
        }
        Expr::Fn(def) => Ok(Value::Fn(Rc::new(Closure {
            def: def.clone(),
            env: env.clone(),
        }))),
        Expr::Call(head, args) => {
            let function = eval(head, env, meter)?;
            let args = args
                .iter()
                .map(|arg| eval(arg, env, meter))
                .collect::<Result<Vec<_>, _>>()?;
            apply(&function, args, meter)
        }
    }
}

/// Call a function value, a keyword, a map, a set or a vector with arguments.
pub fn apply(function: &Value, args: Vec<Value>, meter: &mut Meter) -> Result<Value, Thrown> {
    match function {
        Value::Fn(closure) => call_closure(closure, args, meter),
        Value::Builtin(builtin) => call_builtin(*builtin, &args, meter),
        Value::Keyword(_) | Value::Map(_) | Value::Set(_) | Value::Vector(_) => {
            let (target, key) = match function {
                Value::Keyword(_) => (args.first(), function),
                _ => (Some(function), args.first().unwrap_or(&Value::Nil)),
            };
            if args.is_empty() || args.len() > 2 {
                return error(format!(
                    "Wrong number of args ({}) passed to {}",
                    args.len(),
                    function.pr_str()
                ));
            }
            Ok(
                match (lookup(target.unwrap_or(&Value::Nil), key)?, args.get(1)) {
                    (Some(found), _) => found,
                    (None, Some(default)) => default.clone(),
                    (None, None) => Value::Nil,
                },
            )
        }
        other => error(format!("{} is not a function", other.pr_str())),
    }
}

fn arity(name: &str, args: &[Value], min: usize, max: usize) -> Result<(), Thrown> {
    if args.len() < min || args.len() > max {
        return error(format!(
            "Wrong number of args ({}) passed to {name}",
            args.len()
        ));
    }
    Ok(())
}

fn map_insert(map: &mut Vec<(Value, Value)>, key: Value, value: Value) {
    match map.iter_mut().find(|(k, _)| equal(k, &key)) {
        Some(entry) => entry.1 = value,
        None => map.push((key, value)),
    }
}

/// The value under a key, or nothing when the key is absent. A page row has three
/// attributes and rejects any other keyword.
pub fn lookup(collection: &Value, key: &Value) -> Result<Option<Value>, Thrown> {
    Ok(match collection {
        Value::Map(pairs) => pairs
            .iter()
            .find(|(k, _)| equal(k, key))
            .map(|(_, v)| v.clone()),
        Value::Row(row) => match key {
            Value::Keyword(name) => match &**name {
                "block/name" => Some(Value::str(&row.name)),
                "block/original-name" => Some(Value::str(&row.original_name)),
                "block/properties" => row.has_properties().then(|| row.properties_value()),
                other => return error(format!("unsupported attribute :{other}")),
            },
            _ => None,
        },
        Value::Vector(items) | Value::List(items) => match key {
            Value::Num(n) if *n >= 0.0 && n.fract() == 0.0 => items.get(*n as usize).cloned(),
            _ => None,
        },
        Value::Set(items) => items.iter().find(|member| equal(member, key)).cloned(),
        _ => None,
    })
}

/// `get`: the value under a key, or nil.
pub fn get(collection: &Value, key: &Value) -> Result<Value, Thrown> {
    Ok(lookup(collection, key)?.unwrap_or(Value::Nil))
}

/// `contains?`: whether a map, set or row has the key, or a vector the index.
pub fn contains(collection: &Value, key: &Value) -> Result<bool, Thrown> {
    Ok(match collection {
        Value::Nil => false,
        Value::Map(pairs) => pairs.iter().any(|(k, _)| equal(k, key)),
        Value::Set(items) => items.iter().any(|member| equal(member, key)),
        Value::Row(row) => match key {
            Value::Keyword(name) if ATTRIBUTES.contains(&&**name) => {
                &**name != "block/properties" || row.has_properties()
            }
            _ => false,
        },
        Value::Vector(items) | Value::List(items) => {
            matches!(key, Value::Num(n) if *n >= 0.0 && n.fract() == 0.0 && (*n as usize) < items.len())
        }
        other => {
            return error(format!(
                "contains? not supported on type {}",
                other.pr_str()
            ));
        }
    })
}

pub fn items(value: &Value) -> Result<Rc<Vec<Value>>, Thrown> {
    Ok(match value {
        Value::Vector(items) | Value::List(items) | Value::Set(items) => items.clone(),
        Value::Nil => Rc::new(Vec::new()),
        Value::Map(pairs) => Rc::new(
            pairs
                .iter()
                .map(|(k, v)| Value::vector(vec![k.clone(), v.clone()]))
                .collect(),
        ),
        Value::Str(s) => Rc::new(s.chars().map(|c| Value::str(&c.to_string())).collect()),
        other => {
            return error(format!(
                "Don't know how to create a sequence from {}",
                other.pr_str()
            ));
        }
    })
}

fn number(value: &Value, what: &str) -> Result<f64, Thrown> {
    match value {
        Value::Num(n) => Ok(*n),
        other => error(format!("{what} needs a number, not {}", other.pr_str())),
    }
}

fn string<'v>(value: &'v Value, what: &str) -> Result<&'v str, Thrown> {
    match value {
        Value::Str(s) => Ok(s),
        other => error(format!("{what} needs a string, not {}", other.pr_str())),
    }
}

/// `=` in ClojureScript: sequences equal element by element, numbers by value.
pub fn equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Num(x), Value::Num(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Keyword(x), Value::Keyword(y)) => x == y,
        (Value::Vector(x) | Value::List(x), Value::Vector(y) | Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| equal(p, q))
        }
        (Value::Set(x), Value::Set(y)) => {
            x.len() == y.len() && x.iter().all(|m| y.iter().any(|n| equal(m, n)))
        }
        (Value::Map(x), Value::Map(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.iter().any(|(k2, v2)| equal(k, k2) && equal(v, v2)))
        }
        (Value::Row(x), Value::Row(y)) => x.index == y.index,
        (Value::Fn(x), Value::Fn(y)) => Rc::ptr_eq(x, y),
        (Value::Builtin(x), Value::Builtin(y)) => x == y,
        _ => false,
    }
}

fn utf16_order(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

fn ordering_number(order: Ordering) -> f64 {
    match order {
        Ordering::Less => -1.0,
        Ordering::Equal => 0.0,
        Ordering::Greater => 1.0,
    }
}

/// `compare` in ClojureScript: `nil` first; strings by UTF-16 code unit; numbers by
/// value; booleans, keywords and vectors among themselves. Any other pairing throws.
pub fn compare(a: &Value, b: &Value) -> Result<f64, Thrown> {
    let mismatch = || error(format!("Cannot compare {} to {}", a.to_text(), b.to_text()));
    Ok(match (a, b) {
        (Value::Nil, Value::Nil) => 0.0,
        (Value::Nil, _) => -1.0,
        (_, Value::Nil) => 1.0,
        (Value::Num(x), Value::Num(y)) => {
            ordering_number(x.partial_cmp(y).unwrap_or(Ordering::Equal))
        }
        (Value::Str(x), Value::Str(y)) => ordering_number(utf16_order(x, y)),
        (Value::Bool(x), Value::Bool(y)) => ordering_number(x.cmp(y)),
        (Value::Keyword(x), Value::Keyword(y)) => {
            let split = |k: &str| match k.split_once('/') {
                Some((n, name)) => (Some(n.to_owned()), name.to_owned()),
                None => (None, k.to_owned()),
            };
            let (xn, xname) = split(x);
            let (yn, yname) = split(y);
            ordering_number(match (xn, yn) {
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(p), Some(q)) => utf16_order(&p, &q).then(utf16_order(&xname, &yname)),
                (None, None) => utf16_order(&xname, &yname),
            })
        }
        (Value::Vector(x), Value::Vector(y)) => {
            if x.len() != y.len() {
                ordering_number(x.len().cmp(&y.len()))
            } else {
                for (p, q) in x.iter().zip(y.iter()) {
                    let c = compare(p, q)?;
                    if c != 0.0 {
                        return Ok(c);
                    }
                }
                0.0
            }
        }
        _ => return mismatch(),
    })
}

/// A value as JavaScript's `<` reads it when the operands are not both strings: `null` is
/// 0, a boolean is 0 or 1, a numeric string is its number, and anything else is NaN.
fn js_number(value: &Value) -> f64 {
    match value {
        Value::Nil => 0.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::Num(n) => *n,
        Value::Str(s) => {
            let text = s.trim();
            if text.is_empty() {
                0.0
            } else if text
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'-' | b'+' | b'e' | b'E'))
            {
                text.parse().unwrap_or(f64::NAN)
            } else {
                f64::NAN
            }
        }
        _ => f64::NAN,
    }
}

/// The order `<`, `>`, `<=` and `>=` use in ClojureScript, which compiles them to
/// JavaScript's operators: two strings by UTF-16 code unit, anything else as numbers. A
/// NaN has no order, so every comparison with it is false.
fn js_order(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Str(x), Value::Str(y)) => Some(utf16_order(x, y)),
        _ => js_number(a).partial_cmp(&js_number(b)),
    }
}

/// A comparator's answer as a number, as ClojureScript reads a function that returns a
/// boolean: true means the first argument sorts first.
fn comparator(function: &Value, a: &Value, b: &Value, meter: &mut Meter) -> Result<f64, Thrown> {
    if let Value::Builtin(Builtin::Compare) = function {
        meter.charge(1)?;
        return compare(a, b);
    }
    match apply(function, vec![a.clone(), b.clone()], meter)? {
        Value::Num(n) => Ok(n),
        result if result.truthy() => Ok(-1.0),
        _ => {
            let reverse = apply(function, vec![b.clone(), a.clone()], meter)?;
            Ok(if reverse.truthy() { 1.0 } else { 0.0 })
        }
    }
}

/// A stable merge sort that stops on the first error and never panics for a comparator
/// that disagrees with itself.
fn merge_sort(
    items: Vec<Value>,
    order: &mut dyn FnMut(&Value, &Value) -> Result<f64, Thrown>,
) -> Result<Vec<Value>, Thrown> {
    if items.len() <= 1 {
        return Ok(items);
    }
    let mut right = items;
    let left_part: Vec<Value> = right.drain(..right.len() / 2).collect();
    let left = merge_sort(left_part, order)?;
    let right = merge_sort(right, order)?;
    let mut merged = Vec::with_capacity(left.len() + right.len());
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();
    while let (Some(l), Some(r)) = (left.peek(), right.peek()) {
        if order(l, r)? > 0.0 {
            merged.push(right.next().expect("peeked"));
        } else {
            merged.push(left.next().expect("peeked"));
        }
    }
    merged.extend(left);
    merged.extend(right);
    Ok(merged)
}

fn text_of(args: &[Value]) -> String {
    args.iter().map(Value::to_text).collect()
}

fn replace_with(text: &str, pattern: &str, replacement: &str) -> String {
    // JavaScript reads `$$` and `$&` in a replacement string.
    if !replacement.contains('$') {
        return text.replace(pattern, replacement);
    }
    let mut expanded = String::new();
    let mut chars = replacement.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('$') => {
                    chars.next();
                    expanded.push('$');
                }
                Some('&') => {
                    chars.next();
                    expanded.push_str(pattern);
                }
                _ => expanded.push('$'),
            }
        } else {
            expanded.push(c);
        }
    }
    text.replace(pattern, &expanded)
}

/// Call one of the language's functions.
pub fn call_builtin(builtin: Builtin, args: &[Value], meter: &mut Meter) -> Result<Value, Thrown> {
    match builtin {
        Builtin::Get => {
            arity("get", args, 2, 3)?;
            Ok(match (lookup(&args[0], &args[1])?, args.get(2)) {
                (Some(found), _) => found,
                (None, Some(default)) => default.clone(),
                (None, None) => Value::Nil,
            })
        }
        Builtin::GetIn => {
            arity("get-in", args, 2, 3)?;
            let mut current = args[0].clone();
            for key in items(&args[1])?.iter() {
                meter.charge(1)?;
                match lookup(&current, key)? {
                    Some(found) => current = found,
                    None => {
                        return Ok(args.get(2).cloned().unwrap_or(Value::Nil));
                    }
                }
            }
            Ok(current)
        }
        Builtin::Contains => {
            arity("contains?", args, 2, 2)?;
            Ok(Value::Bool(contains(&args[0], &args[1])?))
        }
        Builtin::Count => {
            arity("count", args, 1, 1)?;
            Ok(Value::Num(match &args[0] {
                Value::Str(s) => s.encode_utf16().count(),
                Value::Row(_) => 3,
                other => items(other)?.len(),
            } as f64))
        }
        Builtin::First | Builtin::Last => {
            arity("first", args, 1, 1)?;
            let seq = items(&args[0])?;
            let found = if builtin == Builtin::First {
                seq.first()
            } else {
                seq.last()
            };
            Ok(found.cloned().unwrap_or(Value::Nil))
        }
        Builtin::Compare => {
            arity("compare", args, 2, 2)?;
            Ok(Value::Num(compare(&args[0], &args[1])?))
        }
        Builtin::SortBy | Builtin::Sort => {
            let (key, order, collection) = match (builtin, args.len()) {
                (Builtin::SortBy, 2) => {
                    (Some(&args[0]), Value::Builtin(Builtin::Compare), &args[1])
                }
                (Builtin::SortBy, 3) => (Some(&args[0]), args[1].clone(), &args[2]),
                (Builtin::Sort, 1) => (None, Value::Builtin(Builtin::Compare), &args[0]),
                (Builtin::Sort, 2) => (None, args[0].clone(), &args[1]),
                _ => {
                    return error(format!(
                        "Wrong number of args ({}) passed to sort",
                        args.len()
                    ));
                }
            };
            let seq = items(collection)?;
            meter.charge(seq.len() as u64)?;
            let sorted = merge_sort(seq.as_ref().clone(), &mut |a, b| {
                let (a, b) = match key {
                    Some(key) => (
                        apply(key, vec![a.clone()], meter)?,
                        apply(key, vec![b.clone()], meter)?,
                    ),
                    None => (a.clone(), b.clone()),
                };
                comparator(&order, &a, &b, meter)
            })?;
            Ok(Value::List(Rc::new(sorted)))
        }
        Builtin::Reverse => {
            arity("reverse", args, 1, 1)?;
            let seq = items(&args[0])?;
            meter.charge(seq.len() as u64)?;
            Ok(Value::List(Rc::new(seq.iter().rev().cloned().collect())))
        }
        Builtin::Take | Builtin::Drop => {
            arity("take", args, 2, 2)?;
            let n = number(&args[0], "take")?.max(0.0) as usize;
            let seq = items(&args[1])?;
            let part: Vec<Value> = if builtin == Builtin::Take {
                seq.iter().take(n).cloned().collect()
            } else {
                seq.iter().skip(n).cloned().collect()
            };
            meter.charge(part.len() as u64)?;
            Ok(Value::List(Rc::new(part)))
        }
        Builtin::Into => {
            arity("into", args, 1, 2)?;
            let Some(from) = args.get(1) else {
                return Ok(args[0].clone());
            };
            let from = items(from)?;
            meter.charge(from.len() as u64)?;
            Ok(match &args[0] {
                Value::Vector(to) => {
                    let mut out = to.as_ref().clone();
                    out.extend(from.iter().cloned());
                    Value::vector(out)
                }
                Value::Set(to) => {
                    let mut out = to.as_ref().clone();
                    for item in from.iter() {
                        if !out.iter().any(|member| equal(member, item)) {
                            out.push(item.clone());
                        }
                    }
                    Value::Set(Rc::new(out))
                }
                Value::Map(to) => {
                    let mut out = to.as_ref().clone();
                    for entry in from.iter() {
                        match entry {
                            Value::Vector(pair) if pair.len() == 2 => {
                                map_insert(&mut out, pair[0].clone(), pair[1].clone());
                            }
                            Value::Map(more) => {
                                for (k, v) in more.iter() {
                                    map_insert(&mut out, k.clone(), v.clone());
                                }
                            }
                            other => {
                                return error(format!(
                                    "Vector arg to map conj must be a pair, not {}",
                                    other.pr_str()
                                ));
                            }
                        }
                    }
                    Value::Map(Rc::new(out))
                }
                Value::List(to) => {
                    let mut out: Vec<Value> = from.iter().rev().cloned().collect();
                    out.extend(to.iter().cloned());
                    Value::List(Rc::new(out))
                }
                Value::Nil => Value::List(Rc::new(from.iter().rev().cloned().collect())),
                other => {
                    return error(format!("Cannot add elements to {}", other.pr_str()));
                }
            })
        }
        Builtin::Vec => {
            arity("vec", args, 1, 1)?;
            let seq = items(&args[0])?;
            meter.charge(seq.len() as u64)?;
            Ok(Value::vector(seq.as_ref().clone()))
        }
        Builtin::Vector => {
            meter.charge(args.len() as u64)?;
            Ok(Value::vector(args.to_vec()))
        }
        Builtin::Concat => {
            let mut parts = Vec::new();
            let mut total = 0u64;
            for arg in args {
                let part = items(arg)?;
                total += part.len() as u64;
                parts.push(part);
            }
            // Charge before building, so a runaway `concat` stops before it allocates.
            meter.charge(total)?;
            let mut out = Vec::with_capacity(total as usize);
            for part in parts {
                out.extend(part.iter().cloned());
            }
            Ok(Value::List(Rc::new(out)))
        }
        Builtin::Map => {
            if args.len() < 2 {
                return error(format!(
                    "Wrong number of args ({}) passed to map",
                    args.len()
                ));
            }
            let collections = args[1..].iter().map(items).collect::<Result<Vec<_>, _>>()?;
            let length = collections.iter().map(|c| c.len()).min().unwrap_or(0);
            meter.charge(length as u64)?;
            let mut out = Vec::with_capacity(length);
            for position in 0..length {
                let call_args = collections.iter().map(|c| c[position].clone()).collect();
                out.push(apply(&args[0], call_args, meter)?);
            }
            Ok(Value::List(Rc::new(out)))
        }
        Builtin::Filter | Builtin::Remove => {
            arity("filter", args, 2, 2)?;
            let seq = items(&args[1])?;
            meter.charge(seq.len() as u64)?;
            let keep = builtin == Builtin::Filter;
            let mut out = Vec::new();
            for item in seq.iter() {
                if apply(&args[0], vec![item.clone()], meter)?.truthy() == keep {
                    out.push(item.clone());
                }
            }
            Ok(Value::List(Rc::new(out)))
        }
        Builtin::Str => Ok(Value::str(&text_of(args))),
        Builtin::Eq | Builtin::NotEq => {
            if args.is_empty() {
                return error("Wrong number of args (0) passed to =");
            }
            let all_equal = args.windows(2).all(|pair| equal(&pair[0], &pair[1]));
            Ok(Value::Bool(all_equal == (builtin == Builtin::Eq)))
        }
        Builtin::Lt | Builtin::Gt | Builtin::Le | Builtin::Ge => {
            if args.is_empty() {
                return error("Wrong number of args (0) passed to a comparison");
            }
            for pair in args.windows(2) {
                let order = js_order(&pair[0], &pair[1]);
                let holds = match builtin {
                    Builtin::Lt => order == Some(Ordering::Less),
                    Builtin::Gt => order == Some(Ordering::Greater),
                    Builtin::Le => matches!(order, Some(Ordering::Less | Ordering::Equal)),
                    _ => matches!(order, Some(Ordering::Greater | Ordering::Equal)),
                };
                if !holds {
                    return Ok(Value::Bool(false));
                }
            }
            Ok(Value::Bool(true))
        }
        Builtin::Not => {
            arity("not", args, 1, 1)?;
            Ok(Value::Bool(!args[0].truthy()))
        }
        Builtin::Replace => {
            arity("clojure.string/replace", args, 3, 3)?;
            let text = string(&args[0], "clojure.string/replace")?;
            let pattern = string(
                &args[1],
                "clojure.string/replace (regular expressions are not supported; the match",
            )?;
            let replacement = string(&args[2], "clojure.string/replace")?;
            meter.charge(text.len() as u64)?;
            Ok(Value::str(&replace_with(text, pattern, replacement)))
        }
        Builtin::Join => {
            arity("clojure.string/join", args, 1, 2)?;
            let (separator, collection) = match args {
                [collection] => ("", collection),
                [separator, collection] => (string_or_text(separator), collection),
                _ => unreachable!("arity checked"),
            };
            let seq = items(collection)?;
            meter.charge(seq.len() as u64)?;
            let parts: Vec<String> = seq.iter().map(Value::to_text).collect();
            Ok(Value::str(&parts.join(separator)))
        }
        Builtin::LowerCase | Builtin::UpperCase => {
            arity("clojure.string/lower-case", args, 1, 1)?;
            let text = args[0].to_text();
            Ok(Value::str(&if builtin == Builtin::LowerCase {
                text.to_lowercase()
            } else {
                text.to_uppercase()
            }))
        }
    }
}

fn string_or_text(value: &Value) -> &str {
    match value {
        Value::Str(s) => s,
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_form::read_form;

    fn run(source: &str, args: Vec<Value>) -> Result<Value, Thrown> {
        let script = parse_script(&read_form(source, 0).unwrap()).unwrap();
        script.call(args, &mut Meter::default())
    }

    fn eval_str(body: &str) -> Value {
        run(&format!("(fn [] {body})"), Vec::new()).unwrap()
    }

    fn strs(values: &[&str]) -> Value {
        Value::vector(values.iter().map(|s| Value::str(s)).collect())
    }

    fn read_error(source: &str) -> QueryError {
        parse_script(&read_form(source, 0).unwrap()).unwrap_err()
    }

    #[test]
    fn special_forms_and_access() {
        assert_eq!(eval_str("(if nil 1 2)"), Value::Num(2.0));
        assert_eq!(eval_str("(let [a 2 b (str a \"x\")] b)"), Value::str("2x"));
        assert_eq!(eval_str("(and 1 2 nil 3)"), Value::Nil);
        assert_eq!(eval_str("(or nil false 4)"), Value::Num(4.0));
        assert_eq!(eval_str("(when true 1 2)"), Value::Num(2.0));
        assert_eq!(eval_str("(when false 1)"), Value::Nil);
        assert_eq!(eval_str("(not nil)"), Value::Bool(true));
        assert_eq!(eval_str("(get {:a 1} :a)"), Value::Num(1.0));
        assert_eq!(eval_str("(get {:a 1} :b 9)"), Value::Num(9.0));
        assert_eq!(
            eval_str("(get-in {:a {:b [1 2]}} [:a :b 1])"),
            Value::Num(2.0)
        );
        // A key that is present with a nil value is found; the default is for absent keys.
        assert_eq!(eval_str("(:a {:a nil} 9)"), Value::Nil);
        assert_eq!(eval_str("(get {:a nil} :a 9)"), Value::Nil);
        assert_eq!(eval_str("(get-in {:a {:b nil}} [:a :b] 5)"), Value::Nil);
        assert_eq!(eval_str("(get-in {:a nil} [:a :b] 5)"), Value::Num(5.0));
        assert_eq!(eval_str("(:a {:a 3})"), Value::Num(3.0));
        assert_eq!(eval_str("(:z {:a 3} 7)"), Value::Num(7.0));
        assert_eq!(eval_str("({:a 3} :a)"), Value::Num(3.0));
        assert_eq!(eval_str("(contains? #{\"a\"} \"a\")"), Value::Bool(true));
        assert_eq!(eval_str("(contains? {:a nil} :a)"), Value::Bool(true));
        assert_eq!(eval_str("(count [1 2 3])"), Value::Num(3.0));
        assert_eq!(eval_str("(first [4 5])"), Value::Num(4.0));
        assert_eq!(eval_str("(last [4 5])"), Value::Num(5.0));
        assert_eq!(eval_str("(first nil)"), Value::Nil);
    }

    #[test]
    fn collections_and_text() {
        assert_eq!(eval_str("(reverse [1 2 3])"), eval_str("[3 2 1]"));
        assert_eq!(eval_str("(take 2 [1 2 3])"), eval_str("[1 2]"));
        assert_eq!(eval_str("(drop 2 [1 2 3])"), eval_str("[3]"));
        assert_eq!(eval_str("(into [1] [2 3])"), eval_str("[1 2 3]"));
        assert_eq!(
            eval_str("(into {} [[:a 1] [:b 2]])"),
            eval_str("{:a 1 :b 2}")
        );
        assert_eq!(eval_str("(into #{1} [1 2])"), eval_str("#{1 2}"));
    }

    #[test]
    fn higher_order_functions() {
        assert_eq!(
            eval_str("(map (fn [x] (str x \"!\")) [\"a\" \"b\"])"),
            strs(&["a!", "b!"])
        );
        assert_eq!(
            eval_str("(filter (fn [x] (not= x 2)) [1 2 3])"),
            eval_str("[1 3]")
        );
        assert_eq!(
            eval_str("(remove (fn [x] (= x 2)) [1 2 3])"),
            eval_str("[1 3]")
        );
        assert_eq!(
            eval_str("(vec (concat [1] [2] nil [3]))"),
            eval_str("[1 2 3]")
        );
        assert_eq!(eval_str("(vector 1 \"a\")"), eval_str("[1 \"a\"]"));
    }

    #[test]
    fn comparisons_and_strings() {
        assert_eq!(eval_str("(< 1 2 3)"), Value::Bool(true));
        assert_eq!(eval_str("(>= 3 3 2)"), Value::Bool(true));
        assert_eq!(eval_str("(> 1 2)"), Value::Bool(false));
        assert_eq!(eval_str("(<= \"a\" \"b\")"), Value::Bool(true));
        assert_eq!(eval_str("(= [1 2] [1 2])"), Value::Bool(true));
        assert_eq!(eval_str("(= 1 1 2)"), Value::Bool(false));
        assert_eq!(eval_str("(not= 1 2)"), Value::Bool(true));
        assert_eq!(eval_str("(str \"a\" nil 1 :k)"), Value::str("a1:k"));
        assert_eq!(
            eval_str("(clojure.string/replace \"a/b/c\" \"/\" \"%2F\")"),
            Value::str("a%2Fb%2Fc")
        );
        assert_eq!(
            eval_str("(clojure.string/join \", \" [\"a\" \"b\"])"),
            Value::str("a, b")
        );
        assert_eq!(eval_str("(clojure.string/join [1 2])"), Value::str("12"));
        assert_eq!(
            eval_str("(clojure.string/lower-case \"AbC\")"),
            Value::str("abc")
        );
        assert_eq!(
            eval_str("(clojure.string/upper-case \"AbC\")"),
            Value::str("ABC")
        );
    }

    #[test]
    fn compare_matches_clojurescript() {
        // nil sorts first; strings by UTF-16 code unit, so an astral character (a pair of
        // surrogates, from 0xD800) sorts before U+FF5E, unlike a comparison of UTF-8 bytes.
        assert_eq!(eval_str("(compare nil \"a\")"), Value::Num(-1.0));
        assert_eq!(eval_str("(compare \"a\" nil)"), Value::Num(1.0));
        assert_eq!(eval_str("(compare nil nil)"), Value::Num(0.0));
        assert_eq!(eval_str("(compare 1 2)"), Value::Num(-1.0));
        assert_eq!(eval_str("(compare \"b\" \"a\")"), Value::Num(1.0));
        assert_eq!(
            eval_str("(compare \"\u{1F600}\" \"\u{FF5E}\")"),
            Value::Num(-1.0)
        );
        assert_eq!(eval_str("(compare true false)"), Value::Num(1.0));
        assert_eq!(eval_str("(compare :a :b)"), Value::Num(-1.0));
        assert_eq!(eval_str("(compare [1 2] [1 3])"), Value::Num(-1.0));
        assert_eq!(eval_str("(compare [1 2 3] [9 9])"), Value::Num(1.0));
        // Timestamps in different zones compare as strings, not by instant.
        assert_eq!(
            eval_str("(compare \"2026-01-01T00:30:00+02:00\" \"2025-12-31T23:00:00Z\")"),
            Value::Num(1.0)
        );
        let thrown = run("(fn [] (compare \"a\" 1))", Vec::new()).unwrap_err();
        assert_eq!(thrown, Thrown::Error("Cannot compare a to 1".into()));
        assert!(run("(fn [] (compare :a \"a\"))", Vec::new()).is_err());
    }

    #[test]
    fn sort_by_is_stable_and_takes_a_comparator() {
        let rows = "[[\"b\" 1] [\"a\" 2] [\"b\" 0] [\"a\" 1]]";
        assert_eq!(
            eval_str(&format!("(vec (sort-by first {rows}))")),
            eval_str("[[\"a\" 2] [\"a\" 1] [\"b\" 1] [\"b\" 0]]")
        );
        assert_eq!(
            eval_str(&format!(
                "(vec (sort-by first (fn [a b] (compare b a)) {rows}))"
            )),
            eval_str("[[\"b\" 1] [\"b\" 0] [\"a\" 2] [\"a\" 1]]")
        );
        // A boolean comparator reads true as "first argument first".
        assert_eq!(
            eval_str("(vec (sort-by (fn [x] x) (fn [a b] (> a b)) [1 3 2]))"),
            eval_str("[3 2 1]")
        );
        assert_eq!(eval_str("(vec (sort [3 1 2]))"), eval_str("[1 2 3]"));
        assert_eq!(
            eval_str("(vec (sort (fn [a b] (compare b a)) [3 1 2]))"),
            eval_str("[3 2 1]")
        );
        assert_eq!(
            eval_str("(vec (sort-by :k [{:k nil :n 1} {:k \"a\" :n 2}]))"),
            eval_str("[{:k nil :n 1} {:k \"a\" :n 2}]")
        );
        assert!(run("(fn [] (sort [\"a\" 1]))", Vec::new()).is_err());
    }

    #[test]
    fn the_newest_first_transform() {
        let rows = Value::vector(vec![
            Value::vector(vec![Value::str("feb"), Value::str("2026-02-01T10:00:00Z")]),
            Value::vector(vec![Value::str("mar"), Value::str("2026-03-01T10:00:00Z")]),
            Value::vector(vec![Value::str("dec"), Value::str("2025-12-31T23:00:00Z")]),
        ]);
        let sorted = run(
            "(fn [rows] (sort-by (fn [row] (get row 1)) (fn [a b] (compare b a)) rows))",
            vec![rows],
        )
        .unwrap();
        let names: Vec<String> = items(&sorted)
            .unwrap()
            .iter()
            .map(|row| get(row, &Value::Num(0.0)).unwrap().to_text())
            .collect();
        assert_eq!(names, ["mar", "feb", "dec"]);
    }

    #[test]
    fn rejects_forms_outside_the_language_with_offsets() {
        for (source, name) in [
            ("(fn [] (slurp \"x\"))", "slurp"),
            ("(fn [rows] (partition 2 rows))", "partition"),
            ("(fn [] (def a 1))", "def"),
            ("(fn [] (loop [] 1))", "loop"),
            ("(fn [] (js/alert 1))", "js/alert"),
            ("(fn [s] (re-find #\"a\" s))", "re-find"),
            ("(fn [] #\"a\")", "#\""),
            ("(fn [] #(inc %))", "#("),
            ("(fn [] (:db/id 1))", ":db/id"),
            ("(fn [r] (get r :block/page))", ":block/page"),
            ("(fn f [] 1)", "fn f"),
            ("(fn [[a b]] a)", "["),
            ("(fn [] (let [[a] [1]] a))", "["),
        ] {
            let error = read_error(source);
            assert_eq!(
                error.kind,
                QueryErrorKind::Unsupported(name.to_owned()),
                "{source}"
            );
            assert!(error.offset > 0 || source.starts_with("(fn f"), "{source}");
        }
        let error = read_error("(fn [] (slurp \"x\"))");
        assert_eq!(error.offset, 8);
        // Not a function at all.
        assert!(matches!(
            parse_script(&read_form("#(inc %)", 0).unwrap())
                .unwrap_err()
                .kind,
            QueryErrorKind::Unsupported(_)
        ));
        assert!(matches!(
            parse_script(&read_form("x", 0).unwrap()).unwrap_err().kind,
            QueryErrorKind::Unsupported(_)
        ));
    }

    #[test]
    fn page_attributes_and_other_keys() {
        assert!(
            parse_script(
                &read_form(
                    "(fn [r] [(:block/name r) (:block/original-name r) (:block/properties r)])",
                    0
                )
                .unwrap()
            )
            .is_ok()
        );
        // A plain property key is fine; only block/ and db/ attributes are checked.
        assert!(parse_script(&read_form("(fn [p] (:podcast-published-at p))", 0).unwrap()).is_ok());
    }

    #[test]
    fn calls_and_arity_errors_throw() {
        assert!(run("(fn [] (get))", Vec::new()).is_err());
        assert!(run("(fn [] ((fn [a] a)))", Vec::new()).is_err());
        assert!(run("(fn [] (1 2))", Vec::new()).is_err());
        assert!(run("(fn [] (contains? \"abc\" 1))", Vec::new()).is_err());
        // `<` follows JavaScript: no error for mixed types, and NaN orders nothing.
        assert_eq!(eval_str("(< 1 \"a\")"), Value::Bool(false));
        assert_eq!(eval_str("(>= 1 \"a\")"), Value::Bool(false));
    }

    #[test]
    fn limits_stop_runaway_work() {
        // Self-application never ends; the depth limit stops it.
        let thrown = run("(fn [] ((fn [f] (f f)) (fn [f] (f f))))", Vec::new()).unwrap_err();
        assert_eq!(thrown, Thrown::Limit(Limit::Depth));
        // A chain of concat doubles the collection each time; the step limit stops it
        // before the memory does.
        let mut source = String::from("(fn [] (let [a [1 2 3 4]");
        let mut previous = "a".to_owned();
        for n in 0..40 {
            source.push_str(&format!(" b{n} (concat {previous} {previous})"));
            previous = format!("b{n}");
        }
        source.push_str(&format!("] (count {previous})))"));
        let started = std::time::Instant::now();
        let thrown = run(&source, Vec::new()).unwrap_err();
        assert_eq!(thrown, Thrown::Limit(Limit::Steps));
        assert!(started.elapsed().as_secs() < 2, "{:?}", started.elapsed());
    }

    /// Results of the same expressions in Logseq 0.10.15, read from its own export of a
    /// fixture page whose query views printed each one with `pr-str` (SCI, ClojureScript).
    #[test]
    fn matches_values_recorded_from_logseq_0_10_15() {
        for (expression, recorded) in [
            ("(compare nil \"a\")", "-1"),
            ("(compare \"a\" nil)", "1"),
            ("(compare \"\u{1F600}\" \"\u{FF5E}\")", "-1"),
            ("(compare true false)", "1"),
            ("(compare :a :b)", "-1"),
            ("(compare [1 2 3] [9 9])", "1"),
            (
                "(compare \"2026-01-01T00:30:00+02:00\" \"2025-12-31T23:00:00Z\")",
                "1",
            ),
            (
                "(vec (sort-by (fn [x] x) (fn [a b] (> a b)) [1 3 2]))",
                "[3 2 1]",
            ),
            ("(:a {:a nil} 9)", "nil"),
            ("(get {:a nil} :a 9)", "nil"),
            ("(get-in {:a {:b nil}} [:a :b] 5)", "nil"),
            ("(get-in {:a nil} [:a :b] 5)", "5"),
            (
                "(str nil 1 :k \"x\" 1.5 [1 \"a\"])",
                "\"1:kx1.5[1 \\\"a\\\"]\"",
            ),
            ("(into {} [[:a 1]])", "{:a 1}"),
            ("(into nil [1 2 3])", "(3 2 1)"),
            ("(clojure.string/replace \"a/b\" \"/\" \"$$\")", "\"a$b\""),
            (
                "(clojure.string/replace \"a$b\" \"$\" \"$&$&\")",
                "\"a$$b\"",
            ),
            ("(count \"\u{1F600}\")", "2"),
            ("(vec (take 2 (reverse [1 2 3])))", "[3 2]"),
            ("(contains? [1 2] 1)", "true"),
            ("(contains? [1 2] 2)", "false"),
            ("(clojure.string/join \", \" [1 nil \"a\"])", "\"1, , a\""),
            ("(< \"a\" \"b\")", "true"),
            ("(< \"\u{1F600}\" \"\u{FF5E}\")", "true"),
            ("(< 1 \"a\")", "false"),
            ("(< \"10\" 9)", "false"),
            ("(< nil 1)", "true"),
            (
                "(vec (sort-by :k [{:k \"b\"} {:k nil} {:k \"a\"}]))",
                "[{:k nil} {:k \"a\"} {:k \"b\"}]",
            ),
            ("(first nil)", "nil"),
            ("(last [1 2])", "2"),
            ("(and 1 nil 3)", "nil"),
            ("(or nil false 4)", "4"),
            ("(= 1 1.0)", "true"),
            ("(not= [1] [1])", "false"),
            ("(clojure.string/lower-case \"AbC\")", "\"abc\""),
            ("(vec (filter (fn [x] (> x 1)) [1 2 3]))", "[2 3]"),
            (
                "(vec (map (fn [a b] (str a b)) [\"x\" \"y\"] [1 2 3]))",
                "[\"x1\" \"y2\"]",
            ),
            ("(vec (concat [1] nil [2]))", "[1 2]"),
            ("(vec (take 1 {:a 1 :b 2}))", "[[:a 1]]"),
            ("(str 3.0 \" \" 1e3 \" \" 10)", "\"3 1000 10\""),
            ("(vec (sort-by (fn [x] x) [3 nil 1 2]))", "[nil 1 2 3]"),
            ("(vec (sort [\"b\" \"a\" nil]))", "[nil \"a\" \"b\"]"),
            ("(sort-by count [[1 2] [3]])", "([3] [1 2])"),
        ] {
            let value = run(&format!("(fn [] {expression})"), Vec::new()).unwrap();
            let shown = match &value {
                Value::Str(_) => value.pr_str(),
                other => other.pr_str(),
            };
            assert_eq!(shown, recorded, "{expression}");
        }
        // Logseq's SCI throws the same message for a string against a number.
        let thrown = run("(fn [] (compare \"a\" 1))", Vec::new()).unwrap_err();
        assert_eq!(thrown, Thrown::Error("Cannot compare a to 1".into()));
    }

    #[test]
    fn steps_are_repeatable() {
        let source = "(fn [xs] (vec (sort-by (fn [x] x) (fn [a b] (compare b a)) xs)))";
        let input = Value::vector(
            (0..200)
                .map(|n| Value::Num(f64::from((n * 37) % 101)))
                .collect(),
        );
        let run_once = || {
            let script = parse_script(&read_form(source, 0).unwrap()).unwrap();
            let mut meter = Meter::default();
            let value = script.call(vec![input.clone()], &mut meter).unwrap();
            (value.pr_str(), meter.steps())
        };
        assert_eq!(run_once(), run_once());
        assert!(run_once().1 > 0);
    }
}
