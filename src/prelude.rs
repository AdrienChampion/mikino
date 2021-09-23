//! Common imports throughout this project.

pub(crate) use std::{
    collections::{BTreeMap as Map, BTreeSet as Set},
    fmt,
    io::Write,
    ops::{Deref, DerefMut},
};

pub use either::Either;
pub use error_chain::bail;
pub use num::{bigint::Sign, BigInt as Int, BigRational as Rat, One, Zero};
pub use rsmt2::SmtRes;

pub use crate::{
    build_decls, build_expr, build_trans, build_typ, check,
    expr::{self, HasTyp},
    parse, trans,
};

/// Step index.
///
/// In the context of a stateful expression, this is the index of the *current step*. If this index
/// is `7` for instance, then state variable `v` in the current step will be `v_7` and will be `v_8`
/// in the next step.
pub type Unroll = usize;

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResExt, Res;
    }

    links {
        Smt2(rsmt2::errors::Error, rsmt2::errors::ErrorKind)
        /// An error from the `rsmt2` crate.
        ;
    }

    foreign_links {
        Io(std::io::Error)
        /// I/O error.
        ;
    }

    errors {
        /// A parse error.
        ///
        /// **NB**: `row` and `col` start at zero.
        ParseErr(prev: Option<String>, row: usize, col: usize, line: String, next: Option<String>, msg: String) {
            description("parse error")
            display("error @{}:{}: `{}`, {}", row, col, line, msg)
        }
    }
}
