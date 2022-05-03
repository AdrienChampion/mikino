//! A simple demo hsmt script.
//!
//! An hsmt script is a Rust-flavored syntax for (a subset of) [SMT-LIB 2][smtlib]. Such scripts
//! allow to perform *satisfiability checks* on formulas; this is done with three basic commands.
//! 
//! All commands can be written function-style `command(args)` or block-style `command { args }`.
//! Some commands also allow `command!(args)` and `command! { args }` such as `check_sat!()`,
//! `get_model!()`, `println!(...)`... and all commands that produce an output or exit the script
//! (including `panic!("message")`).
//! 
//! The script below showcases all commands available. Note that hsmt scripts, unlike SMT-LIB 2
//! scripts, can branch using if-then-else(-otherwise). Conditions in branchings can only be
//! check-sat results. Also unlike SMT-LIB 2, hsmt scripts let you bind (name) check-sat results
//! in a (meta-)variable usable in branchings.
//!
//! [smtlib]: https://smtlib.cs.uiowa.edu/language.shtml
//! (SMT-LIB's official website)


/// First, *declaring variables* (*constants* in SMT-LIB 2) is necessary to later write formulas
/// that use these variables.
vars {
	/// Current counter value.
	cnt: int,
	/// Next counter value.
	next_cnt: int,
	/// True if resetting.
	reset: bool,
	/// True if counting.
	counting: bool,
}

/// Next, we write formulas and *assert* them. This tells the solver *"I want this formula to be
/// true"*.
assert {
	// This `=` is **NOT** an assignment. It is a **constraint** comparing the value of `next_cnt`
	// to the value of the right-hand side. It does **NOT** "set" `next_cnt`'s value. In a context
	// where `next_cnt`'s value is `7` and `reset` is true for instance, then this formula becomes
	// `7 = 0`, which is `false`.
	//       v
	next_cnt = if reset {
		0 // resetting
	} else if counting {
		cnt + 1 // incrementing
	} else {
		cnt // stuttering
	}
}

/// `assert { ... }` actually accepts a comma-separated list (with optional trailing comma) of
/// formulas, making it convenient to assert more than one formula.
assert {
	// `cnt` is positive
	cnt ≥ 0,
	// `next_cnt` is not positive
	¬(next_cnt ≥ 0),
}


/// So far, we have asserted what `next_cnt` is based on `cnt`, `reset` and `counting`. We have
/// also asserted that `cnt` is positive or zero, and that `next_cnt` is strictly negative. We now
/// *check sat* to ask the solver *"can all of these formulas be true for valuation of the
/// variables?".
check_sat!()
// outputs `unsat`


/// Solver says it's not possible: we just proved that with this definition of `next_cnt`, it is
/// not possible to get a strictly negative value assuming `cnt` is itself positive.
/// 
/// Interestingly, hsmt scripts provides *branching* (if-then-else) on check sat results, so we
/// can communicate this result explicitly.
if check_sat!() {
	// sat case
	echo!("`next_cnt` can be *strictly* negative 😿")
	// produce a model
	get_model!()
} else {
	// unsat case
	echo!("`next_cnt` can only be **positive or zero** 😸")
} otherwise {
	// this *optional* `otherwise` branch triggers when the check sat returns `unknown` or
	// `timeout`. Here, we decide to *panic* with a message, which ends the script with an error.
	panic!("solver was not able to decide satisfiability of this easy check 🙀")
}

/// Hsmt scripts even let you (meta-)bind check sat results to meta-variables using `let ... =
/// ...;`.
let is_sat = check_sat!();

/// Meta-variables can then be used in branching.
if is_sat {
	// Can use `println!` instead of `echo!`.
	println!("sat 😿")
	get_model!()
} else {
	println!("unsat 😸")
}
// No `otherwise` branch, will panic if the check sat was inconclusive.


echo!("**reset**ting solver")
reset!()

// This just echoes a newline.
echo!()
echo!("nothing declared or asserted at this point")

/// Nothing there, should be sat.
if check_sat!() {
	echo!("*of course* it's **sat**")
} else {
	panic!("no way this is **unsat**")
} otherwise {
	echo!("epic fail, this check was so easy")
	/// We can exit with a unix-style signed integer exit code.
	exit!(2)
}


/// Let's declare/assert almost the same things as previously.
vars {
	cnt next_cnt: int,
	reset counting: bool,
}

assert {
	next_cnt =
		if reset {
			0
		} else if counting {
			cnt + 1
		} else {
			cnt
		}
	,
	// `cnt` is **strictly** positive
	cnt > 0,
	// `next_cnt` is not **strictly** positive
	¬(next_cnt > 0),
}

/// What do you think?
let not_strictly_positive = check_sat!();

/// Let's take a look.
if not_strictly_positive {
	echo!("yeah, `next_cnt` can actually be not **strictly** positive if `reset`:")
	get_model!()
} else {
	panic!("unreachable")
}


/// Let's get some values.
eval! {
	cnt,
	cnt + 2,
	cnt ≥ 0,
	if ¬reset {
		11
	} else {
		7 * (cnt + next_cnt)
	},
}

/// You can also use `get_values` or `get_value` for that.
get_values! {
	cnt * (next_cnt + 7),
}

/// Let's force `reset` to be false.
echo!("if we forbid `reset`ting, `next_cnt` should always be strictly positive")
assert {
	¬reset
}

/// This time `next_cnt` can only be strictly positive.
if check_sat!() {
	panic!("unreachable")
} else {
	echo!("indeed it is")
}


echo!()
echo!("all done here")

/// We can omit the `exit!` code, which is the same as `exit!(0)`.
exit!()
