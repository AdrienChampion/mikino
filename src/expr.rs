//! Defines the expression structure used to represent predicates.

crate::prelude!();

use rsmt2::print::{Expr2Smt, Sort2Smt, Sym2Smt};

#[cfg(test)]
mod test;

pub use crate::{build_expr as build, build_typ};

/// A type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Typ {
    /// Bool type.
    Bool,
    /// Integer type.
    Int,
    /// Rational type.
    Rat,
}
impl Typ {
    /// Creates a bool type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::Typ;
    /// let bool_typ = Typ::bool();
    /// assert_eq!(&bool_typ.to_string(), "bool")
    /// ```
    pub fn bool() -> Self {
        Self::Bool
    }
    /// Creates an integer type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::Typ;
    /// let int_typ = Typ::int();
    /// assert_eq!(&int_typ.to_string(), "int")
    /// ```
    pub fn int() -> Self {
        Self::Int
    }
    /// Creates a rational type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::Typ;
    /// let rat_typ = Typ::rat();
    /// assert_eq!(&rat_typ.to_string(), "rat")
    /// ```
    pub fn rat() -> Self {
        Self::Rat
    }

    /// True if the type is an arithmetic one.
    pub fn is_arith(self) -> bool {
        match self {
            Self::Bool => false,
            Self::Int | Self::Rat => true,
        }
    }
}
impl Sort2Smt for Typ {
    fn sort_to_smt2<W: Write>(&self, w: &mut W) -> SmtRes<()> {
        write!(
            w,
            "{}",
            match self {
                Self::Bool => "Bool",
                Self::Int => "Int",
                Self::Rat => "Real",
            }
        )?;
        Ok(())
    }
}

/// Operator precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precedence {
    /// Maximal precedence.
    Max,
    /// Numeric precedence.
    Some(usize),
}
impl Precedence {
    /// Constructor.
    pub const fn new(precedence: usize) -> Self {
        Self::Some(precedence)
    }
    /// Maximal precedence.
    pub const fn max() -> Self {
        Self::Max
    }
}
impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match self {
            Self::Max => {
                if *other == Self::Max {
                    Some(Equal)
                } else {
                    Some(Greater)
                }
            }
            Self::Some(self_prec) => match other {
                Self::Max => Some(Less),
                Self::Some(other_prec) => self_prec.partial_cmp(other_prec),
            },
        }
    }
}

/// Constants.
///
/// Currently only booleans, integers and rationals are supported.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cst {
    /// Bool constant.
    B(bool),
    /// Integer constant.
    I(Int),
    /// Rational constant.
    R(Rat),
}
impl HasTyp for Cst {
    fn typ(&self) -> Typ {
        match self {
            Self::B(_) => Typ::Bool,
            Self::I(_) => Typ::Int,
            Self::R(_) => Typ::Rat,
        }
    }
}
impl Cst {
    /// Creates a boolean constant.
    pub fn bool(b: bool) -> Self {
        Self::B(b)
    }
    /// Creates an integer constant.
    pub fn int<I: Into<Int>>(i: I) -> Self {
        Self::I(i.into())
    }
    /// Creates a rational constant.
    pub fn rat<R: Into<Rat>>(r: R) -> Self {
        Self::R(r.into())
    }

    /// Unwraps a boolean constant.
    pub fn as_bool(&self) -> Res<bool> {
        match self {
            Self::B(b) => Ok(*b),
            _ => bail!("expected boolean, found `{}`", self),
        }
    }
    /// Unwraps an integer constant.
    pub fn as_int(&self) -> Res<&Int> {
        match self {
            Self::I(i) => Ok(i),
            _ => bail!("expect integer, found `{}`", self),
        }
    }
    /// Unwraps a rational constant.
    pub fn as_rat(&self) -> Res<&Rat> {
        match self {
            Self::R(r) => Ok(r),
            _ => bail!("expect integer, found `{}`", self),
        }
    }
}
impl Expr2Smt<()> for Cst {
    fn expr_to_smt2<W: Write>(&self, w: &mut W, _: ()) -> SmtRes<()> {
        match self {
            Self::B(b) => write!(w, "{}", b)?,
            Self::I(i) => write!(w, "{}", i)?,
            Self::R(r) => write!(w, "(/ {} {})", r.numer(), r.denom())?,
        }
        Ok(())
    }
}

/// Operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    /// If-then-else.
    Ite,
    /// Logical implication.
    Implies,
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
    /// Integer division.
    IDiv,
    /// Modulo.
    Mod,
    /// Greater than or equal to.
    Ge,
    /// Less than or equal to.
    Le,
    /// Greater than.
    Gt,
    /// Less than.
    Lt,
    /// Equal to.
    Eq,
    /// Logical negation.
    Not,
    /// Logical conjunction.
    And,
    /// Logical disjunction.
    Or,
}
impl Op {
    /// Tries to parse an operator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::Op;
    /// assert_eq!(Op::of_str("+"), Some(Op::Add));
    /// assert_eq!(Op::of_str("and"), Some(Op::And));
    /// assert_eq!(Op::of_str("⋀"), Some(Op::And));
    /// assert_eq!(Op::of_str("add"), None);
    /// ```
    pub fn of_str<Str: AsRef<str>>(s: Str) -> Option<Self> {
        use Op::*;
        let res = match s.as_ref() {
            "ite" => Ite,
            "=>" | "implies" | "⇒" => Implies,
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "div" => IDiv,
            "mod" => Mod,
            ">=" | "≥" => Ge,
            "<=" | "≤" => Le,
            ">" => Gt,
            "<" => Lt,
            "=" => Eq,
            "not" | "!" | "¬" => Not,
            "and" | "&&" | "⋀" => And,
            "or" | "||" | "⋁" => Or,
            _ => return None,
        };
        Some(res)
    }

    /// Human-SMT string representation.
    pub fn hsmt_str(self) -> &'static [&'static str] {
        match self {
            Self::Ite => &["if", "then", "else"],
            Self::Implies => &["⇒"],
            Self::Add => &["+"],
            Self::Sub => &["-"],
            Self::Mul => &["*"],
            Self::Div => &["/"],
            Self::IDiv => &["/"],
            Self::Mod => &["%"],
            Self::Ge => &["≥"],
            Self::Le => &["≤"],
            Self::Gt => &[">"],
            Self::Lt => &["<"],
            Self::Eq => &["="],
            Self::Not => &["¬"],
            Self::And => &["⋀"],
            Self::Or => &["⋁"],
        }
    }

    /// Human-SMT string representation.
    pub fn smt_str(self) -> &'static str {
        match self {
            Self::Ite => "ite",
            Self::Implies => "=>",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::IDiv => "/",
            Self::Mod => "%",
            Self::Ge => "≥",
            Self::Le => "≤",
            Self::Gt => ">",
            Self::Lt => "<",
            Self::Eq => "=",
            Self::Not => "¬",
            Self::And => "⋀",
            Self::Or => "⋁",
        }
    }

    /// True if `self` is an arithmetic relation.
    pub fn is_arith_relation(self) -> bool {
        match self {
            Self::Ge | Self::Le | Self::Gt | Self::Lt => true,
            Self::Ite
            | Self::Implies
            | Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::IDiv
            | Self::Mod
            | Self::Eq
            | Self::Not
            | Self::And
            | Self::Or => false,
        }
    }

    /// Minimal arity of `self`.
    pub fn min_arity(self) -> usize {
        match self {
            Self::Not | Self::Add | Self::Sub | Self::And | Self::Or => 1,
            Self::Mod
            | Self::Mul
            | Self::Div
            | Self::IDiv
            | Self::Implies
            | Self::Eq
            | Self::Le
            | Self::Lt
            | Self::Ge
            | Self::Gt => 2,
            Self::Ite => 3,
        }
    }

    /// Maximal arity for `self`, `None` if infinite.
    pub fn max_arity(self) -> Option<usize> {
        match self {
            Self::Not => Some(1),
            Self::Add
            | Self::Sub
            | Self::Mul
            | Self::And
            | Self::Or
            | Self::Implies
            | Self::Eq
            | Self::Le
            | Self::Lt
            | Self::Ge
            | Self::Gt => None,
            Self::Mod | Self::Div | Self::IDiv => Some(2),
            Self::Ite => Some(3),
        }
    }

    /// True if the operator is left associative.
    pub fn is_left_associative(self) -> bool {
        match self {
            Self::Add
            | Self::Sub
            | Self::Mul
            | Self::And
            | Self::Or
            | Self::Implies
            | Self::Eq
            | Self::Le
            | Self::Lt
            | Self::Ge
            | Self::Gt => true,
            Self::Not | Self::Mod | Self::Div | Self::IDiv | Self::Ite => false,
        }
    }

    /// Type-checks an operator application.
    pub fn type_check<V: HasTyp>(self, args: &[PExpr<V>]) -> Res<Typ> {
        if args.len() < self.min_arity() {
            bail!(
                "`{}` expects at least {} argument(s)",
                self,
                self.min_arity(),
            )
        }
        if let Some(max) = self.max_arity() {
            if args.len() > max {
                bail!("`{}` expects at most {} argument(s)", self, max)
            }
        }
        // From now on we allow ourselves to `unwrap` as we know we have a legal number of
        // arguments.

        let typ = match self {
            Self::Ite => {
                let typ = args[0].typ();
                if typ != Typ::Bool {
                    bail!("expected first argument of type `bool`, got `{}`", typ)
                }

                let thn_typ = args[1].typ();
                let els_typ = args[2].typ();

                if thn_typ != els_typ {
                    bail!(
                        "`{}`'s second and third arguments should have the same type, got `{}` and `{}`",
                        self, thn_typ, els_typ,
                    )
                }

                thn_typ
            }
            Self::Implies | Self::And | Self::Or | Self::Not => {
                if args.iter().any(|e| e.typ() != Typ::Bool) {
                    bail!("`{}`'s arguments must all be boolean expressions", self)
                }
                Typ::Bool
            }

            Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::IDiv
            | Self::Mod
            | Self::Le
            | Self::Ge
            | Self::Lt
            | Self::Gt => {
                let mut typs = args.iter().map(PExpr::typ);
                let first = typs.next().expect("at least one argument");
                if !first.is_arith() {
                    bail!(
                        "`{}`'s arguments must have an arithmetic type, unexpected type `{}`",
                        self,
                        first,
                    )
                }
                for typ in typs {
                    if typ != first {
                        bail!(
                            "`{}`'s arguments must all have the same type, found `{}` and `{}`",
                            self,
                            first,
                            typ,
                        )
                    }
                }
                if (self == Self::IDiv || self == Self::Mod) && first != Typ::Int {
                    bail!(
                        "`{}` can only be applied to integer arguments, found `{}`",
                        self,
                        first,
                    )
                }

                if self == Self::Div {
                    Typ::Rat
                } else if self == Self::Mod {
                    Typ::Int
                } else if self.is_arith_relation() {
                    Typ::Bool
                } else {
                    first
                }
            }

            Self::Eq => {
                let mut typs = args.iter().map(PExpr::typ);
                let first = typs.next().unwrap();
                for typ in typs {
                    if typ != first {
                        bail!(
                            "`{}`'s arguments must all have the same type, found `{}` and `{}`",
                            self,
                            first,
                            typ,
                        )
                    }
                }
                Typ::Bool
            }
        };

        Ok(typ)
    }

    /// Ite evaluation.
    pub fn eval_ite(cnd: Cst, thn: Cst, els: Cst) -> Res<Cst> {
        if cnd.as_bool()? {
            Ok(thn)
        } else {
            Ok(els)
        }
    }
    /// Implication evaluation.
    pub fn eval_implies(first: Cst, mut tail: impl Iterator<Item = Cst>) -> Res<Cst> {
        let mut prev = first;
        loop {
            // Is `prev` the last argument?
            if let Some(next) = tail.next() {
                // No, yield `true` if `prev` is false, implication is trivally true.
                if !next.as_bool()? {
                    break Ok(Cst::bool(true));
                } else {
                    prev = next;
                    continue;
                }
            } else {
                // `prev` is the last argument, previous ones were necessarily `true`.
                // Implication evaluates to `prev`, the last argument.
                break Ok(prev);
            }
        }
    }

    /// Arithmetic operator application.
    pub fn eval_nary_binop(
        binop: impl Fn(Cst, Cst) -> Res<Cst>,
        fst: Cst,
        snd: Cst,
        tail: impl Iterator<Item = Cst>,
    ) -> Res<Cst> {
        let mut res = binop(fst, snd)?;
        for cst in tail {
            res = binop(res, cst)?
        }
        Ok(res)
    }

    /// Applies the operator to a vector of constants.
    pub fn eval(self, args: Vec<Cst>) -> Res<Cst> {
        let arg_count = args.len();
        if arg_count < self.min_arity() {
            bail!(
                "`{}` expects at least {} argument(s)",
                self,
                self.min_arity(),
            )
        }
        if let Some(max) = self.max_arity() {
            if arg_count > max {
                bail!("`{}` expects at most {} argument(s)", self, max)
            }
        }
        // From now on we allow ourselves to `unwrap` as we know we have a legal number of
        // arguments.
        //
        // Also, this runs after type-checking so we're not providing a lot of context on type
        // errors since they should be mikino-level bugs, not user-level error we must provide
        // feedback on.

        let mut args = args.into_iter();

        // Generates an error for an operator.
        macro_rules! op_err {
            (bin $(($real_op:expr))? $lft:ident $op:tt $rgt:ident) => ({
                #[allow(unused_mut, unused_assignments)]
                let mut op = stringify!($op).to_string();
                $(
                    op = $real_op.to_string();
                )?
                bail!(
                    "cannot apply `{}` to `{}: {}` and `{}: {}`",
                    op,
                    $lft,
                    $lft.typ(),
                    $rgt,
                    $rgt.typ(),
                )
            });
            (un $op:tt $cst:expr) => {
                bail!(
                    "cannot apply `{}` to `{}: {}`",
                    stringify!($op),
                    $cst,
                    $cst.typ(),
                )
            };
        }

        // Applies some operator to two arithmetic constants.
        macro_rules! arith_op {
            ($op:tt) => ({
                let (fst, snd) = (args.next().unwrap(), args.next().unwrap());
                Self::eval_nary_binop(
                    |lft, rgt| arith_op!(bin lft $op rgt),
                    fst,
                    snd,
                    args,
                )
            });
            (rel $op:tt) => ({
                let (fst, mut prev) = (args.next().unwrap(), args.next().unwrap());
                let snd = prev.clone();
                let mut res = arith_op!(rel fst <= snd)?;
                if !res.as_bool()? {
                    Ok(Cst::B(false))
                } else {
                    loop {
                        if let Some(next) = args.next() {
                            let current = next.clone();
                            res = arith_op!(rel prev <= current)?;
                            if !res.as_bool()? {
                                break Ok(Cst::B(false));
                            }
                            prev = next;
                            continue;
                        } else {
                            break Ok(Cst::B(true));
                        }
                    }
                }
            });
            (bin $lft:ident $op:tt $rgt:ident) => (
                match ($lft, $rgt) {
                    (Cst::I(lft), Cst::I(rgt)) => Ok(Cst::I(lft $op rgt)),
                    (Cst::R(lft), Cst::R(rgt)) => Ok(Cst::R(lft $op rgt)),
                    (lft, rgt) => op_err!(bin lft $op rgt),
                }
            );
            (rel $lft:ident $op:tt $rgt:ident) => (
                match ($lft, $rgt) {
                    (Cst::I(lft), Cst::I(rgt)) => Res::Ok(Cst::B(lft $op rgt)),
                    (Cst::R(lft), Cst::R(rgt)) => Ok(Cst::B(lft $op rgt)),
                    (lft, rgt) => op_err!(bin lft $op rgt),
                }
            );
            (un $unop:tt $cst:expr) => (
                match $cst {
                    Cst::I(i) => Ok(Cst::I($unop i)),
                    Cst::R(r) => Ok(Cst::R($unop r)),
                    cst => op_err!(un $unop cst),
                }
            );
        }

        match self {
            Self::Ite => Self::eval_ite(
                args.next().unwrap(),
                args.next().unwrap(),
                args.next().unwrap(),
            ),

            Self::Implies => Self::eval_implies(args.next().unwrap(), args),

            Self::Add => arith_op!(+),
            Self::Sub => {
                if arg_count == 1 {
                    arith_op!(un - args.next().unwrap())
                } else {
                    arith_op!(-)
                }
            }
            Self::Mul => arith_op!(*),
            Self::Div => Self::eval_nary_binop(
                |lft, rgt| match (lft, rgt) {
                    (Cst::I(lft), Cst::I(rgt)) => Ok(Cst::R(Rat::new(lft, rgt))),
                    (Cst::R(lft), Cst::R(rgt)) => Ok(Cst::R(lft / rgt)),
                    (Cst::I(lft), Cst::R(rgt)) => Ok(Cst::R(Rat::new(lft, Int::one()) / rgt)),
                    (Cst::R(lft), Cst::I(rgt)) => Ok(Cst::R(lft / Rat::new(rgt, Int::one()))),
                    (lft, rgt) => op_err!(bin lft / rgt),
                },
                args.next().unwrap(),
                args.next().unwrap(),
                args,
            ),
            Self::IDiv => Self::eval_nary_binop(
                |lft, rgt| match (lft, rgt) {
                    (Cst::I(lft), Cst::I(rgt)) => Ok(Cst::I(lft / rgt)),
                    (lft, rgt) => op_err!(bin(Self::IDiv) lft / rgt),
                },
                args.next().unwrap(),
                args.next().unwrap(),
                args,
            ),
            Self::Mod => arith_op!(%),

            Self::Ge => arith_op!(rel >=),
            Self::Le => arith_op!(rel <=),
            Self::Gt => arith_op!(rel >),
            Self::Lt => arith_op!(rel <),

            // We just don't care, not even checking for types.
            Self::Eq => {
                let fst = args.next().unwrap();
                loop {
                    if let Some(next) = args.next() {
                        if fst != next {
                            break Ok(Cst::B(false));
                        } else {
                            continue;
                        }
                    } else {
                        break Ok(Cst::B(true));
                    }
                }
            }

            Self::Not => Ok(Cst::B(!args.next().unwrap().as_bool()?)),

            Self::And => loop {
                if let Some(next) = args.next() {
                    if !next.as_bool()? {
                        break Ok(Cst::B(false));
                    } else {
                        continue;
                    }
                } else {
                    break Ok(Cst::B(true));
                }
            },
            Self::Or => loop {
                if let Some(next) = args.next() {
                    if next.as_bool()? {
                        break Ok(Cst::B(true));
                    } else {
                        continue;
                    }
                } else {
                    break Ok(Cst::B(false));
                }
            },
        }
    }
}
impl Expr2Smt<()> for Op {
    fn expr_to_smt2<W: Write>(&self, w: &mut W, _: ()) -> SmtRes<()> {
        write!(
            w,
            "{}",
            match self {
                Self::Ite => "ite",
                Self::Implies => "=>",
                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",
                Self::IDiv => "div",
                Self::Mod => "mod",
                Self::Ge => ">=",
                Self::Le => "<=",
                Self::Gt => ">",
                Self::Lt => "<",
                Self::Eq => "=",
                Self::Not => "not",
                Self::And => "and",
                Self::Or => "or",
            }
        )?;
        Ok(())
    }
}

/// Trait implemented by all variables.
pub trait HasTyp: fmt::Display {
    /// Type accessor.
    fn typ(&self) -> Typ;
}

/// A stateless variable.
///
/// This type of variable is used in stateless expressions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Var {
    /// Variable identifier.
    id: String,
    /// Type of the variable.
    typ: Typ,
}
impl Var {
    /// Constructor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{Var, Typ};
    /// # #[allow(dead_code)]
    /// let var = Var::new("v_1", Typ::Bool);
    /// ```
    pub fn new<S: Into<String>>(id: S, typ: Typ) -> Self {
        Self { id: id.into(), typ }
    }

    /// Identifier accessor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{Var, Typ};
    /// let var = Var::new("v_1", Typ::Bool);
    /// assert_eq!(var.id(), "v_1");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }
}
impl HasTyp for Var {
    /// Type accessor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{Var, Typ, HasTyp};
    /// let var = Var::new("v_1", Typ::Bool);
    /// assert_eq!(var.typ(), Typ::Bool);
    /// ```
    fn typ(&self) -> Typ {
        self.typ
    }
}
impl Sym2Smt<Unroll> for Var {
    fn sym_to_smt2<W: Write>(&self, w: &mut W, step: Unroll) -> SmtRes<()> {
        write!(w, "{}@{}", self.id, step)?;
        Ok(())
    }
}
impl Sym2Smt<()> for Var {
    fn sym_to_smt2<W: Write>(&self, w: &mut W, _step: ()) -> SmtRes<()> {
        write!(w, "{}", self.id)?;
        Ok(())
    }
}

/// A stateful variable.
///
/// This type of variable is used in stateful expressions: expressions that span over two steps.
/// Typically, the transition relation of a system is stateful. A stateful variable is essentially a
/// [Var] with a *next* flag that specifies whether the stateful variable refers to the current or
/// next version of the underlying variable.
///
/// [Var]: struct.Var.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SVar {
    /// Underlying variable.
    var: Var,
    /// True if the variable refers to the next state version of the variable.
    nxt: bool,
}
impl SVar {
    /// State variable constructor with a `next` flag.
    pub fn new(var: Var, nxt: bool) -> Self {
        Self { var, nxt }
    }
    /// Constructor for next state variables.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{Var, SVar, Typ, HasTyp};
    /// let var = Var::new("v_1", Typ::Bool);
    /// let svar = SVar::new_next(var);
    /// assert_eq!(&svar.to_string(), "v_1@1");
    /// assert_eq!(svar.id(), "v_1");
    /// assert_eq!(svar.typ(), Typ::Bool);
    /// ```
    pub fn new_next(var: Var) -> Self {
        Self::new(var, true)
    }

    /// Constructor for current state variables.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{Var, SVar, Typ, HasTyp};
    /// let var = Var::new("v_1", Typ::Bool);
    /// let svar = SVar::new_curr(var);
    /// assert_eq!(&svar.to_string(), "v_1@0");
    /// assert_eq!(svar.id(), "v_1");
    /// assert_eq!(svar.typ(), Typ::Bool);
    /// ```
    pub fn new_curr(var: Var) -> Self {
        Self::new(var, false)
    }

    /// True if the state variable is a next state variable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{Var, SVar, Typ};
    /// let var = Var::new("v_1", Typ::Bool);
    /// let svar = SVar::new_next(var.clone());
    /// assert!(svar.is_next());
    /// let svar = SVar::new_curr(var);
    /// assert!(!svar.is_next());
    /// ```
    pub fn is_next(&self) -> bool {
        self.nxt
    }
}
impl Sym2Smt<Unroll> for SVar {
    fn sym_to_smt2<W: Write>(&self, w: &mut W, step: Unroll) -> SmtRes<()> {
        write!(w, "{}@{}", self.id, if self.nxt { step + 1 } else { step })?;
        Ok(())
    }
}
impl HasTyp for SVar {
    fn typ(&self) -> Typ {
        self.typ
    }
}

/// The polymorphic expression structure.
///
/// This structure is polymorphic in the type of variables. This allows to create two types, [Expr]
/// and [SExpr] for stateless and stateful expressions respectively. The former is `PExpr<Var>`
/// while the latter is `PExpr<SVar>`.
///
/// [Expr]: type.Expr.html
/// [SExpr]: type.SExpr.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PExpr<V> {
    /// A constant.
    Cst(Cst),
    /// A variable.
    Var(V),
    /// An operator application.
    App {
        /// The operator.
        op: Op,
        /// The arguments.
        args: Vec<PExpr<V>>,
    },
}
impl<V> PExpr<V> {
    /// Variable constructor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{PExpr, Var, SVar, Typ};
    /// let var = Var::new("v_1", Typ::Bool);
    /// let expr: PExpr<Var> = PExpr::new_var(var.clone());
    /// assert_eq!(expr, PExpr::Var(var.clone()));
    /// let svar = SVar::new_next(var);
    /// let expr: PExpr<SVar> = PExpr::new_var(svar.clone());
    /// assert_eq!(expr, PExpr::Var(svar));
    /// ```
    pub fn new_var(var: V) -> Self {
        Self::Var(var)
    }

    /// Constant constructor.
    pub fn new_cst(cst: Cst) -> Self {
        Self::Cst(cst)
    }

    /// Operator application constructor.
    /// Variable constructor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use mikino_api::expr::{PExpr, Var, SVar, Typ, Op};
    /// let var = Var::new("v_1", Typ::Bool);
    /// let expr: PExpr<Var> = PExpr::new_var(var.clone());
    /// assert_eq!(expr, PExpr::Var(var.clone()));
    /// let svar = SVar::new_next(var);
    /// let expr: PExpr<SVar> = PExpr::new_var(svar.clone());
    /// assert_eq!(expr, PExpr::Var(svar));
    /// ```
    pub fn new_op(op: Op, args: Vec<Self>) -> Res<Self>
    where
        V: HasTyp,
    {
        op.type_check(&args)?;
        Ok(Self::simplify_app(op, args))
    }

    /// Simplifies the application of `op` to `args`, **non-recursively**.
    fn simplify_app(op: Op, mut args: Vec<Self>) -> Self {
        match (op, args.len()) {
            (Op::Sub, 1) if args[0].is_cst() => match &args[0] {
                Self::Cst(Cst::I(i)) => Cst::I(-i).into(),
                Self::Cst(Cst::R(r)) => Cst::R(-r).into(),
                Self::Cst(Cst::B(_)) => panic!("trying to apply `{}` to a boolean", op),
                _ => Self::App { op, args },
            },
            (Op::Div, 2) if args[0].is_cst() && args[1].is_cst() => match (&args[0], &args[1]) {
                (Self::Cst(Cst::I(lft)), Self::Cst(Cst::I(rgt))) => Cst::I(lft - rgt).into(),
                (Self::Cst(Cst::R(lft)), Self::Cst(Cst::R(rgt))) => Cst::R(lft - rgt).into(),
                _ => panic!("trying to apply `{}` to arguments of unexpected type", op),
            },
            (Op::Add, 1) | (Op::And, 1) | (Op::Or, 1) => {
                args.pop().expect("[unreachable] pop on vec of len `1`")
            }
            _ => Self::App { op, args },
        }
    }

    /// True if `self` is a constant.
    pub fn is_cst(&self) -> bool {
        match self {
            Self::Cst(_) => true,
            Self::Var(_) | Self::App { .. } => false,
        }
    }
    /// True if `self` is a variable.
    pub fn is_var(&self) -> bool {
        match self {
            Self::Var(_) => true,
            Self::Cst(_) | Self::App { .. } => false,
        }
    }
    /// True if `self` is an application.
    pub fn is_app(&self) -> bool {
        match self {
            Self::App { .. } => true,
            Self::Cst(_) | Self::Var(_) => false,
        }
    }

    /// Cleans a string representation.
    ///
    /// This is useful for get-values/evals commands as they store the user representation for
    /// expressions to evaluate. This function basically trims the string and collapses all
    /// consecutive whitespaces in a single space.
    pub fn clean_repr(s: impl AsRef<str>) -> String {
        let s = s.as_ref();
        let s = s.trim();

        let mut res = String::with_capacity(s.len());
        let mut ws_mode = false;
        for char in s.chars() {
            match (char.is_whitespace(), ws_mode) {
                (true, true) => (),
                (true, false) => {
                    ws_mode = true;
                    res.push(' ')
                }
                (false, _) => {
                    ws_mode = false;
                    res.push(char)
                }
            }
        }
        res.shrink_to_fit();
        res
    }

    /// Negation of a reference to an expression.
    ///
    /// This is mostly useful in cases when we have a reference to an expression we don't want to
    /// clone, and want to assert the negation.
    pub fn negated(&self) -> NotPExpr<V> {
        self.into()
    }

    /// Bottom-up fold over an expression.
    pub fn fold<'me, Acc>(
        &'me self,
        mut var_action: impl FnMut(&'me V) -> Acc,
        mut cst_action: impl FnMut(&'me Cst) -> Acc,
        mut app_action: impl FnMut(Op, Vec<Acc>) -> Acc,
    ) -> Acc {
        // Stores *frames*, *i.e.* info about an application:
        // - the accumulator value when we reached this
        // - its operator,
        // - a vector of `Acc` corresponding to the fold result on the application's first
        //   arguments,
        // - an iterator over the remaining arguments.
        //
        // If frame `(op, res, todo)` is on the `stack`, it means we have fold-ed over the first
        // `res.length()` arguments and `res` stores these results, we are currently going down the
        // `res.length()+1`th argument, and the remaining arguments are given by the `todo`
        // iterator.
        let mut stack: Vec<(Op, Vec<Acc>, std::slice::Iter<PExpr<V>>)> = Vec::with_capacity(12);
        let mut current = self;

        'go_down: loop {
            // If the current expression has kids, we will recursively fold them so `acc` below is
            // not used. Otherwise, we call the relevant user-provided function to generate a value
            // for `acc` and use it to `'go_up` below.
            let mut acc = match current {
                Self::Var(var) => var_action(var),
                Self::Cst(cst) => cst_action(cst),
                Self::App { op, args } => {
                    let mut todo = args.iter();

                    // At least one argument?
                    if let Some(next) = todo.next() {
                        // Need to go down `next` now.
                        current = next;
                        // Empty-for-now vector of accumulators.
                        let res = Vec::with_capacity(args.len());
                        // Push frame on the stack for when we go up.
                        stack.push((*op, res, todo));

                        continue 'go_down;
                    } else {
                        // No argument, this should actually not happen, but it's not a problem for
                        // folding so we might as well just handle it.
                        app_action(*op, vec![])
                    }
                }
            };

            // Go up the stack recursively, usind and updating `acc`. We `continue 'go_down`
            // whenever we find a frame for an application that still has kids to fold.
            'go_up: while let Some((op, mut res, mut todo)) = stack.pop() {
                // `acc` is result obtained for the `res.length() + 1`th argument of the
                // application.
                res.push(acc);

                // Any argument left to go down into?
                if let Some(next) = todo.next() {
                    // Need to handle it.
                    current = next;
                    // Don't forget to add the frame back.
                    stack.push((op, res, todo));

                    continue 'go_down;
                } else {
                    // Update current accumulator and keep going up.
                    acc = app_action(op, res);

                    continue 'go_up;
                }
            }

            // Stack is empty, `acc` contains the result of folding over the whole expression
            // (`self`).
            return acc;
        }
    }
}
impl<V: HasTyp> HasTyp for PExpr<V> {
    fn typ(&self) -> Typ {
        match self {
            Self::Var(var) => var.typ(),
            Self::Cst(cst) => cst.typ(),
            Self::App { op, args } => match op.type_check(args) {
                Ok(typ) => typ,
                Err(e) => panic!("illegal operator application `{}`: {}", self, e),
            },
        }
    }
}
impl<Info: Copy, V: Sym2Smt<Info>> Expr2Smt<Info> for PExpr<V> {
    fn expr_to_smt2<W: Write>(&self, w: &mut W, i: Info) -> SmtRes<()> {
        match self {
            Self::Cst(cst) => cst.expr_to_smt2(w, ()),
            Self::Var(var) => var.sym_to_smt2(w, i),
            Self::App { op, args } => {
                write!(w, "(")?;
                op.expr_to_smt2(w, ())?;
                for arg in args {
                    write!(w, " ")?;
                    arg.expr_to_smt2(w, i)?
                }
                write!(w, ")")?;
                Ok(())
            }
        }
    }
}

/// A simple (stateless) expression.
pub type Expr = PExpr<Var>;

/// A stateful expression.
pub type SExpr = PExpr<SVar>;

/// Represents the negation of a borrowed expression.
///
/// This is mostly useful in cases when we have a reference to an expression we don't want to clone,
/// and want to assert the negation.
///
/// # Examples
///
/// ```rust
/// # use mikino_api::expr::{self, NotPExpr, Expr, Var};
/// use mikino_api::rsmt2::print::Expr2Smt;
/// let expr = expr::build!(
///     (and (>= (v_1: int) 0) (v_2: bool))
/// );
/// let expr = &expr;
///
/// let not_expr: NotPExpr<Var> = expr.negated();
///
/// use std::io::Write;
/// let mut buff = vec![];
/// not_expr.expr_to_smt2(&mut buff, 0);
/// let s = String::from_utf8_lossy(&buff);
/// assert_eq!(&s, "(not (and (>= v_1@0 0) v_2@0))")
/// ```
pub struct NotPExpr<'a, V> {
    expr: &'a PExpr<V>,
}
impl<'a, V> From<&'a PExpr<V>> for NotPExpr<'a, V> {
    fn from(expr: &'a PExpr<V>) -> Self {
        Self { expr }
    }
}
impl<'a, Info: Copy, V: Sym2Smt<Info>> Expr2Smt<Info> for NotPExpr<'a, V> {
    fn expr_to_smt2<W: Write>(&self, w: &mut W, i: Info) -> SmtRes<()> {
        write!(w, "(not ")?;
        self.expr.expr_to_smt2(w, i)?;
        write!(w, ")")?;
        Ok(())
    }
}

/// Packs basic trait implementations.
mod trait_impls {
    use super::*;

    impl fmt::Display for Typ {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Bool => write!(fmt, "bool"),
                Self::Int => write!(fmt, "int"),
                Self::Rat => write!(fmt, "rat"),
            }
        }
    }

    impl fmt::Display for Op {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Ite => write!(fmt, "ite"),
                Self::Implies => write!(fmt, "=>"),
                Self::Add => write!(fmt, "+"),
                Self::Sub => write!(fmt, "-"),
                Self::Mul => write!(fmt, "*"),
                Self::Div => write!(fmt, "/"),
                Self::IDiv => write!(fmt, "div"),
                Self::Mod => write!(fmt, "%"),
                Self::Ge => write!(fmt, ">="),
                Self::Le => write!(fmt, "<="),
                Self::Gt => write!(fmt, ">"),
                Self::Lt => write!(fmt, "<"),
                Self::Eq => write!(fmt, "="),
                Self::Not => write!(fmt, "not"),
                Self::And => write!(fmt, "and"),
                Self::Or => write!(fmt, "or"),
            }
        }
    }

    impl fmt::Display for Cst {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::B(b) => b.fmt(fmt),
                Self::I(i) => {
                    if i.sign() == Sign::Minus {
                        write!(fmt, "(- {})", -i)
                    } else {
                        i.fmt(fmt)
                    }
                }
                Self::R(r) => {
                    let (num, den) = (r.numer(), r.denom());
                    match (num.sign(), den.sign()) {
                        (Sign::Minus, Sign::Minus) => write!(fmt, "(/ {} {})", -num, -den),
                        (Sign::Minus, _) => write!(fmt, "(- (/ {} {}))", -num, den),
                        (_, Sign::Minus) => write!(fmt, "(- (/ {} {}))", num, -den),
                        _ => write!(fmt, "(/ {} {})", num, den),
                    }
                }
            }
        }
    }
    impl From<bool> for Cst {
        fn from(b: bool) -> Self {
            Self::B(b)
        }
    }
    impl From<Int> for Cst {
        fn from(i: Int) -> Self {
            Self::I(i)
        }
    }
    impl From<usize> for Cst {
        fn from(n: usize) -> Self {
            Int::from_bytes_be(Sign::Plus, &n.to_be_bytes()).into()
        }
    }
    impl From<(usize, usize)> for Cst {
        fn from((num, den): (usize, usize)) -> Self {
            let (num, den): (Int, Int) = (num.into(), den.into());
            Rat::new(num, den).into()
        }
    }
    impl From<Rat> for Cst {
        fn from(r: Rat) -> Self {
            Self::R(r)
        }
    }

    impl fmt::Display for Var {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "{}", self.id)
        }
    }

    impl Deref for SVar {
        type Target = Var;
        fn deref(&self) -> &Var {
            &self.var
        }
    }
    impl fmt::Display for SVar {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "{}@{}", self.id, if self.nxt { 1 } else { 0 })
        }
    }

    impl<V: fmt::Display> fmt::Display for PExpr<V> {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Cst(cst) => cst.fmt(fmt),
                Self::Var(var) => var.fmt(fmt),
                Self::App { op, args } => {
                    write!(fmt, "({}", op)?;
                    for arg in args {
                        write!(fmt, " {}", arg)?
                    }
                    write!(fmt, ")")
                }
            }
        }
    }
    impl<C, V> From<C> for PExpr<V>
    where
        C: Into<Cst>,
    {
        fn from(cst: C) -> Self {
            Self::Cst(cst.into())
        }
    }
    impl<V> From<(Op, Vec<PExpr<V>>)> for PExpr<V> {
        fn from((op, args): (Op, Vec<PExpr<V>>)) -> Self {
            Self::App { op, args }
        }
    }
}

/// A meta-variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaVar {
    /// Variable identifier.
    pub ident: String,
}
impl From<String> for MetaVar {
    fn from(ident: String) -> Self {
        Self { ident }
    }
}
impl MetaVar {
    /// Constructor.
    pub fn new(ident: impl Into<String>) -> Self {
        Self {
            ident: ident.into(),
        }
    }
}
/// A meta-expression.
pub type MExpr = PExpr<MetaVar>;
