//! Common imports throughout this project.

#[allow(unused_imports)]
pub(crate) use std::{
    collections::{BTreeMap as Map, BTreeSet as Set},
    fmt,
    io::Write,
    marker, mem,
    ops::{self, Deref, DerefMut},
    path::PathBuf,
    time,
};

pub use either::Either;
pub use error_chain::bail;
pub use num::{bigint::Sign, BigInt as Int, BigRational as Rat, One, Zero};
pub use rsmt2::{parse::SmtParser as RSmtParser, SmtConf, SmtRes, Solver as SmtSolver};

pub use crate::{
    ast, build_decls, build_expr, build_trans, build_typ, check,
    err::*,
    expr::{self, HasTyp, Typ},
    parse::{self, Span, Spn},
    script,
    solver::{SFSolver, SLSolver},
    trans,
};

/// Step index.
///
/// In the context of a stateful expression, this is the index of the *current step*. If this index
/// is `7` for instance, then state variable `v` in the current step will be `v_7` and will be `v_8`
/// in the next step.
pub type Unroll = usize;

/// Markdown mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Mode {
    Bold,
    Ita,
    Code,
}
impl Mode {
    fn split(s: &str) -> (&str, Option<(Self, &str)>) {
        let mut chars = s.chars().enumerate();
        while let Some((idx, char)) = chars.next() {
            let (offset, mode) = match char {
                '`' => (1, Self::Code),
                '*' => match chars.next() {
                    Some((_, '*')) => (2, Self::Bold),
                    _ => (1, Self::Ita),
                },
                _ => continue,
            };
            let pref = &s[0..idx];
            let suff = &s[idx + offset..s.len()];
            return (pref, Some((mode, suff)));
        }
        return (s, None);
    }
}

/// Style trait.
pub trait Style {
    /// Output generated by styling functions.
    type Styled: fmt::Display;
    /// Bold styling.
    fn bold(&self, s: &str) -> Self::Styled;
    /// Red styling.
    fn red(&self, s: &str) -> Self::Styled;
    /// Green styling.
    fn green(&self, s: &str) -> Self::Styled;
    /// Underline styling.
    fn under(&self, s: &str) -> Self::Styled;
    /// Gray styling.
    fn gray(&self, s: &str) -> Self::Styled;

    /// Italic.
    fn ita(&self, s: &str) -> Self::Styled;
    /// Code style.
    fn code(&self, s: &str) -> Self::Styled;

    /// Prettifies some markdown, no nesting supported (sorry).
    fn pretty_md(&self, s: impl AsRef<str>, default: impl Fn(&str) -> Self::Styled) -> String {
        let s = s.as_ref();
        let mut current = s;
        let mut current_mode: Option<Mode> = None;
        let mut res = String::with_capacity(s.len() * 2);

        while !current.is_empty() {
            match Mode::split(current) {
                (pref, None) => {
                    if current_mode.is_some() {
                        // unbalanced modes, give up.
                        return s.into();
                    } else {
                        res.push_str(&default(pref).to_string());
                        break;
                    }
                }
                (pref, Some((mode, suff))) => {
                    current = suff;

                    if let Some(omode) = current_mode {
                        current_mode = None;
                        if omode == mode {
                            match mode {
                                Mode::Bold => res.push_str(&self.bold(pref).to_string()),
                                Mode::Ita => res.push_str(&self.ita(pref).to_string()),
                                Mode::Code => res.push_str(&self.code(pref).to_string()),
                            }
                        } else {
                            // nested or unbalanced mode, give up
                            return s.into();
                        }
                    } else {
                        res.push_str(&default(pref).to_string());
                        current_mode = Some(mode);
                    }
                }
            }
        }

        res.shrink_to_fit();
        res
    }
}

impl Style for () {
    type Styled = String;
    fn bold(&self, s: &str) -> String {
        s.into()
    }
    fn red(&self, s: &str) -> String {
        s.into()
    }
    fn green(&self, s: &str) -> String {
        s.into()
    }
    fn under(&self, s: &str) -> String {
        s.into()
    }
    fn gray(&self, s: &str) -> String {
        s.into()
    }
    fn ita(&self, s: &str) -> String {
        s.into()
    }
    fn code(&self, s: &str) -> String {
        s.into()
    }
    fn pretty_md(&self, s: impl AsRef<str>, default: impl Fn(&str) -> String) -> String {
        default(s.as_ref())
    }
}
impl<'a, T> Style for &'a T
where
    T: Style,
{
    type Styled = T::Styled;
    fn bold(&self, s: &str) -> Self::Styled {
        (*self).bold(s)
    }
    fn red(&self, s: &str) -> Self::Styled {
        (*self).red(s)
    }
    fn green(&self, s: &str) -> Self::Styled {
        (*self).green(s)
    }
    fn under(&self, s: &str) -> Self::Styled {
        (*self).under(s)
    }
    fn gray(&self, s: &str) -> Self::Styled {
        (*self).gray(s)
    }

    fn ita(&self, s: &str) -> Self::Styled {
        (*self).ita(s)
    }
    fn code(&self, s: &str) -> Self::Styled {
        (*self).code(s)
    }
}
