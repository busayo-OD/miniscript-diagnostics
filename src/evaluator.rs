//! Walks the public Miniscript AST and produces a `Diagnostic` tree.

use std::sync::Arc;

use miniscript::bitcoin::{absolute, relative};
use miniscript::{Miniscript, MiniscriptKey, ScriptContext, Terminal, Threshold};

use crate::context::DiagnosticContext;
use crate::diagnostic::{Diagnostic, Status};

pub fn evaluate<Pk, Ctx>(
    miniscript: &Miniscript<Pk, Ctx>,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    evaluate_term(&miniscript.node, ctx)
}

fn evaluate_term<Pk, Ctx>(term: &Terminal<Pk, Ctx>, ctx: &DiagnosticContext<Pk>) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    match term {
        Terminal::Alt(child)
        | Terminal::Swap(child)
        | Terminal::Check(child)
        | Terminal::Verify(child)
        | Terminal::NonZero(child)
        | Terminal::ZeroNotEqual(child) => evaluate_term(&child.node, ctx),

        Terminal::PkK(pk) => evaluate_pk("pk", pk, ctx),
        Terminal::PkH(pk) => evaluate_pk("pkh", pk, ctx),

        Terminal::Sha256(hash) => evaluate_hash("sha256", hash, ctx.has_sha256_preimage(hash)),
        Terminal::Hash256(hash) => evaluate_hash("hash256", hash, ctx.has_hash256_preimage(hash)),
        Terminal::Ripemd160(hash) => {
            evaluate_hash("ripemd160", hash, ctx.has_ripemd160_preimage(hash))
        }
        Terminal::Hash160(hash) => evaluate_hash("hash160", hash, ctx.has_hash160_preimage(hash)),

        Terminal::After(lock_time) => evaluate_after(*lock_time, ctx),
        Terminal::Older(lock_time) => evaluate_older(*lock_time, ctx),

        Terminal::AndV(left, right) => evaluate_and("AND_V", left, right, ctx),
        Terminal::AndB(left, right) => evaluate_and("AND_B", left, right, ctx),
        Terminal::AndOr(a, b, c) => evaluate_andor(a, b, c, ctx),
        Terminal::OrI(left, right) => evaluate_or_i(left, right, ctx),
        Terminal::OrD(left, right) => evaluate_or("OR_D", left, right, ctx),
        Terminal::OrC(left, right) => evaluate_or("OR_C", left, right, ctx),
        Terminal::OrB(left, right) => evaluate_or("OR_B", left, right, ctx),
        Terminal::Thresh(thresh) => evaluate_thresh(thresh, ctx),
        Terminal::Multi(thresh) => evaluate_multi("MULTI", thresh, ctx),
        Terminal::MultiA(thresh) => evaluate_multi("MULTI_A", thresh, ctx),

        other => unsupported(other),
    }
}

fn unsupported<Pk, Ctx>(terminal: &Terminal<Pk, Ctx>) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    let name = terminal_kind_name(terminal);
    Diagnostic::leaf(
        name.clone(),
        Status::Unsupported,
        format!("the `{name}` fragment is not currently supported"),
    )
}

fn terminal_kind_name<Pk: MiniscriptKey, Ctx: ScriptContext>(
    terminal: &Terminal<Pk, Ctx>,
) -> String {
    match terminal {
        Terminal::True => "true".into(),
        Terminal::False => "false".into(),
        Terminal::PkK(_) => "pk (key check)".into(),
        Terminal::PkH(_) => "pkh (key hash check)".into(),
        Terminal::RawPkH(_) => "raw pkh (hash160 only)".into(),
        Terminal::After(_) => "after (absolute timelock)".into(),
        Terminal::Older(_) => "older (relative timelock)".into(),
        Terminal::Sha256(_) => "sha256".into(),
        Terminal::Hash256(_) => "hash256".into(),
        Terminal::Ripemd160(_) => "ripemd160".into(),
        Terminal::Hash160(_) => "hash160".into(),
        Terminal::Alt(_) => "a: (Alt wrapper)".into(),
        Terminal::Swap(_) => "s: (Swap wrapper)".into(),
        Terminal::Check(_) => "c: (Check wrapper)".into(),
        Terminal::DupIf(_) => "d: (DupIf wrapper)".into(),
        Terminal::Verify(_) => "v: (Verify wrapper)".into(),
        Terminal::NonZero(_) => "j: (NonZero wrapper)".into(),
        Terminal::ZeroNotEqual(_) => "n: (ZeroNotEqual wrapper)".into(),
        Terminal::AndV(_, _) => "and_v".into(),
        Terminal::AndB(_, _) => "and_b".into(),
        Terminal::AndOr(_, _, _) => "andor".into(),
        Terminal::OrB(_, _) => "or_b".into(),
        Terminal::OrD(_, _) => "or_d".into(),
        Terminal::OrC(_, _) => "or_c".into(),
        Terminal::OrI(_, _) => "or_i".into(),
        Terminal::Thresh(_) => "thresh".into(),
        Terminal::Multi(_) => "multi".into(),
        Terminal::MultiA(_) => "multi_a".into(),
    }
}

fn evaluate_pk<Pk: MiniscriptKey>(label: &str, pk: &Pk, ctx: &DiagnosticContext<Pk>) -> Diagnostic {
    let available = ctx.is_key_available(pk);

    let (status, reason) = if available {
        (
            Status::Satisfied,
            "the required signing key is available in the supplied context",
        )
    } else {
        (
            Status::Impossible,
            "no signature is available for the required key, and signatures cannot be forged",
        )
    };

    Diagnostic::leaf(format!("{label}({pk})"), status, reason)
}

fn evaluate_hash<H: std::fmt::Display>(label: &str, hash: &H, available: bool) -> Diagnostic {
    let (status, reason) = if available {
        (
            Status::Satisfied,
            "the required preimage is available in the supplied context",
        )
    } else {
        (
            Status::Unavailable,
            "the required preimage was not supplied",
        )
    };

    Diagnostic::leaf(format!("{label}({hash})"), status, reason).with_meta("hash", hash.to_string())
}

fn evaluate_after<Pk: MiniscriptKey>(
    lock_time: miniscript::AbsLockTime,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic {
    let satisfied = ctx.check_after(lock_time.into());

    let status = if satisfied {
        Status::Satisfied
    } else {
        Status::Unavailable
    };

    let reason = if satisfied {
        "the absolute timelock requirement is currently met"
    } else {
        "the absolute timelock has not yet matured"
    };

    let mut diagnostic = Diagnostic::leaf(
        format!("after({})", lock_time.to_consensus_u32()),
        status,
        reason,
    );

    let lock_time: absolute::LockTime = lock_time.into();

    match lock_time {
        absolute::LockTime::Blocks(required_height) => {
            diagnostic =
                diagnostic.with_meta("required", format!("{required_height} (block height)"));

            match ctx.chain_height {
                Some(current_height) => {
                    diagnostic = diagnostic
                        .with_meta("available", format!("{current_height} (block height)"));

                    let remaining = required_height
                        .to_consensus_u32()
                        .saturating_sub(current_height.to_consensus_u32());

                    diagnostic = diagnostic.with_meta("remaining", format!("{remaining} blocks"));
                }
                None => {
                    diagnostic = diagnostic
                        .with_meta("note", "no chain height supplied; cannot compute remaining");
                }
            }
        }

        absolute::LockTime::Seconds(required_time) => {
            diagnostic = diagnostic.with_meta("required", format!("{required_time} (unix time)"));

            match ctx.chain_time {
                Some(current_time) => {
                    diagnostic =
                        diagnostic.with_meta("available", format!("{current_time} (unix time)"));

                    let remaining = required_time
                        .to_consensus_u32()
                        .saturating_sub(current_time.to_consensus_u32());

                    diagnostic = diagnostic.with_meta("remaining", format!("{remaining} seconds"));
                }
                None => {
                    diagnostic = diagnostic
                        .with_meta("note", "no chain time supplied; cannot compute remaining");
                }
            }
        }
    }

    diagnostic
}

fn evaluate_older<Pk: MiniscriptKey>(
    lock_time: miniscript::RelLockTime,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic {
    let satisfied = ctx.check_older(lock_time.into());

    let status = if satisfied {
        Status::Satisfied
    } else {
        Status::Unavailable
    };

    let reason = if satisfied {
        "the relative timelock requirement is currently met"
    } else {
        "the relative timelock has not yet matured"
    };

    let mut diagnostic = Diagnostic::leaf(
        format!("older({})", lock_time.to_consensus_u32()),
        status,
        reason,
    );

    let lock_time: relative::LockTime = lock_time.into();

    match lock_time {
        relative::LockTime::Blocks(required_blocks) => {
            diagnostic = diagnostic.with_meta("required", format!("{required_blocks} blocks"));

            match ctx.elapsed_blocks {
                Some(elapsed_blocks) => {
                    diagnostic =
                        diagnostic.with_meta("available", format!("{elapsed_blocks} blocks"));

                    let remaining = required_blocks
                        .value()
                        .saturating_sub(elapsed_blocks.value());

                    diagnostic = diagnostic.with_meta("remaining", format!("{remaining} blocks"));
                }
                None => {
                    diagnostic = diagnostic.with_meta(
                        "note",
                        "no elapsed-block count supplied; cannot compute remaining",
                    );
                }
            }
        }

        relative::LockTime::Time(required_intervals) => {
            let required_seconds = u32::from(required_intervals.value()) * 512;

            diagnostic = diagnostic.with_meta(
                "required",
                format!("{required_seconds} seconds ({required_intervals} x 512s intervals)"),
            );

            match ctx.elapsed_time {
                Some(elapsed_time) => {
                    let elapsed_seconds = u32::from(elapsed_time.value()) * 512;

                    diagnostic = diagnostic.with_meta(
                        "available",
                        format!("{elapsed_seconds} seconds ({elapsed_time} x 512s intervals)"),
                    );

                    let remaining = required_seconds.saturating_sub(elapsed_seconds);

                    diagnostic = diagnostic.with_meta("remaining", format!("{remaining} seconds"));
                }
                None => {
                    diagnostic = diagnostic
                        .with_meta("note", "no elapsed-time supplied; cannot compute remaining");
                }
            }
        }
    }

    diagnostic
}

fn evaluate_and<Pk, Ctx>(
    label: &str,
    left: &Arc<Miniscript<Pk, Ctx>>,
    right: &Arc<Miniscript<Pk, Ctx>>,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    let left_diagnostic = evaluate(left, ctx);
    let right_diagnostic = evaluate(right, ctx);
    let status = Status::combine_and(left_diagnostic.status, right_diagnostic.status);
    let reason = and_reason(&left_diagnostic, &right_diagnostic);

    Diagnostic::combinator(label, status, vec![left_diagnostic, right_diagnostic])
        .with_reason(reason)
}

fn and_reason(left: &Diagnostic, right: &Diagnostic) -> String {
    match (left.status, right.status) {
        (Status::Satisfied, Status::Satisfied) => {
            "both required branches are currently satisfied".to_string()
        }
        (Status::Satisfied, _) => format!(
            "{} is satisfied, but {} is not: {}",
            left.fragment,
            right.fragment,
            right.reason.as_deref().unwrap_or("its status is blocking")
        ),
        (_, Status::Satisfied) => format!(
            "{} is satisfied, but {} is not: {}",
            right.fragment,
            left.fragment,
            left.reason.as_deref().unwrap_or("its status is blocking")
        ),
        _ => format!(
            "neither required branch is currently satisfied: {} is not satisfied ({}), and {} is not satisfied ({})",
            left.fragment,
            left.reason.as_deref().unwrap_or("its status is blocking"),
            right.fragment,
            right.reason.as_deref().unwrap_or("its status is blocking"),
        ),
    }
}

/// Evaluates `andor(A, B, C)`: satisfied via `(A AND B)` or via `C`.
fn evaluate_andor<Pk, Ctx>(
    a: &Arc<Miniscript<Pk, Ctx>>,
    b: &Arc<Miniscript<Pk, Ctx>>,
    c: &Arc<Miniscript<Pk, Ctx>>,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    let a_diagnostic = evaluate(a, ctx);
    let b_diagnostic = evaluate(b, ctx);
    let c_diagnostic = evaluate(c, ctx);

    let and_status = Status::combine_and(a_diagnostic.status, b_diagnostic.status);
    let status = Status::combine_or(and_status, c_diagnostic.status);
    let reason = andor_reason(and_status, &a_diagnostic, &b_diagnostic, &c_diagnostic);

    Diagnostic::combinator(
        "ANDOR",
        status,
        vec![a_diagnostic, b_diagnostic, c_diagnostic],
    )
    .with_reason(reason)
}

fn andor_reason(and_status: Status, a: &Diagnostic, b: &Diagnostic, c: &Diagnostic) -> String {
    match (and_status, c.status) {
        (Status::Satisfied, _) => format!("satisfied via {} and {}", a.fragment, b.fragment),
        (_, Status::Satisfied) => format!("satisfied via the else-branch {}", c.fragment),

        (Status::Unsupported, Status::Unsupported) => format!(
            "cannot currently be evaluated: both the and-branch ({} and {}) and the else-branch {} are unsupported",
            a.fragment, b.fragment, c.fragment
        ),
        (Status::Unsupported, _) => format!(
            "cannot currently be evaluated: the and-branch ({} and {}) includes an unsupported fragment",
            a.fragment, b.fragment
        ),
        (_, Status::Unsupported) => format!(
            "cannot currently be evaluated: the else-branch {} is unsupported",
            c.fragment
        ),

        (Status::Impossible, Status::Impossible) => {
            "neither the and-branch nor the else-branch can be satisfied".to_string()
        }
        (Status::Unavailable, Status::Unavailable) => format!(
            "not currently satisfied, but the and-branch ({} and {}) or the else-branch {} could still become available",
            a.fragment, b.fragment, c.fragment
        ),
        (Status::Unavailable, Status::Impossible) => format!(
            "not currently satisfied, but the and-branch ({} and {}) could still become available",
            a.fragment, b.fragment
        ),
        (Status::Impossible, Status::Unavailable) => format!(
            "not currently satisfied, but the else-branch {} could still become available",
            c.fragment
        ),
    }
}

fn evaluate_or_i<Pk, Ctx>(
    left: &Arc<Miniscript<Pk, Ctx>>,
    right: &Arc<Miniscript<Pk, Ctx>>,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    let left_diagnostic = evaluate(left, ctx);
    let right_diagnostic = evaluate(right, ctx);
    let status = Status::combine_or(left_diagnostic.status, right_diagnostic.status);
    let reason = or_reason(&left_diagnostic, &right_diagnostic);

    Diagnostic::combinator("OR_I", status, vec![left_diagnostic, right_diagnostic])
        .with_reason(reason)
}

fn or_reason(left: &Diagnostic, right: &Diagnostic) -> String {
    match (left.status, right.status) {
        (Status::Satisfied, Status::Satisfied) => {
            format!(
                "satisfied via either {} or {}",
                left.fragment, right.fragment
            )
        }
        (Status::Satisfied, _) => format!("satisfied via {}", left.fragment),
        (_, Status::Satisfied) => format!("satisfied via {}", right.fragment),

        (Status::Unsupported, Status::Unsupported) => format!(
            "cannot currently be evaluated: both {} and {} are unsupported fragments",
            left.fragment, right.fragment
        ),
        (Status::Unsupported, _) => format!(
            "cannot currently be evaluated: {} is unsupported ({})",
            left.fragment,
            left.reason.as_deref().unwrap_or("not currently supported")
        ),
        (_, Status::Unsupported) => format!(
            "cannot currently be evaluated: {} is unsupported ({})",
            right.fragment,
            right.reason.as_deref().unwrap_or("not currently supported")
        ),

        (Status::Impossible, Status::Impossible) => "neither branch can be satisfied".to_string(),

        (Status::Unavailable, Status::Unavailable) => format!(
            "no branch is currently satisfied, but {} or {} could still become available",
            left.fragment, right.fragment
        ),
        (Status::Unavailable, Status::Impossible) => format!(
            "no branch is currently satisfied, but {} could still become available",
            left.fragment
        ),
        (Status::Impossible, Status::Unavailable) => format!(
            "no branch is currently satisfied, but {} could still become available",
            right.fragment
        ),
    }
}

/// Evaluates disjunctive fragments that require at least one side to be
/// satisfiable.
fn evaluate_or<Pk, Ctx>(
    label: &str,
    left: &Arc<Miniscript<Pk, Ctx>>,
    right: &Arc<Miniscript<Pk, Ctx>>,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    let left_diagnostic = evaluate(left, ctx);
    let right_diagnostic = evaluate(right, ctx);
    let status = Status::combine_or(left_diagnostic.status, right_diagnostic.status);
    let reason = or_reason(&left_diagnostic, &right_diagnostic);

    Diagnostic::combinator(label, status, vec![left_diagnostic, right_diagnostic])
        .with_reason(reason)
}

fn evaluate_thresh<Pk, Ctx>(
    thresh: &Threshold<Arc<Miniscript<Pk, Ctx>>, 0>,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic
where
    Pk: MiniscriptKey,
    Ctx: ScriptContext,
{
    let k = thresh.k();
    let n = thresh.n();

    let children: Vec<Diagnostic> = thresh
        .iter()
        .map(|child| evaluate(child.as_ref(), ctx))
        .collect();

    let statuses: Vec<Status> = children
        .iter()
        .map(|diagnostic| diagnostic.status)
        .collect();

    let status = Status::combine_thresh(k, &statuses);
    let satisfied = statuses
        .iter()
        .filter(|status| **status == Status::Satisfied)
        .count();
    let reason = thresh_reason(k, n, satisfied, status);

    Diagnostic::combinator(format!("THRESH({k}, {n})"), status, children)
        .with_meta("required", k.to_string())
        .with_meta("satisfied", satisfied.to_string())
        .with_reason(reason)
}

fn thresh_reason(k: usize, n: usize, satisfied: usize, status: Status) -> String {
    let detail = match status {
        Status::Satisfied => "requirement met".to_string(),
        Status::Impossible => "not enough branches can ever be satisfied".to_string(),
        Status::Unavailable => {
            format!(
                "{} more needed and still reachable",
                k.saturating_sub(satisfied)
            )
        }
        Status::Unsupported => "blocked by an unsupported branch".to_string(),
    };
    format!("{satisfied} of {k} required are satisfied ({n} total); {detail}")
}

fn evaluate_multi<Pk, const MAX: usize>(
    label: &str,
    thresh: &Threshold<Pk, MAX>,
    ctx: &DiagnosticContext<Pk>,
) -> Diagnostic
where
    Pk: MiniscriptKey,
{
    let k = thresh.k();
    let n = thresh.n();

    let children: Vec<Diagnostic> = thresh.iter().map(|pk| evaluate_pk("pk", pk, ctx)).collect();

    let statuses: Vec<Status> = children
        .iter()
        .map(|diagnostic| diagnostic.status)
        .collect();

    let status = Status::combine_thresh(k, &statuses);
    let satisfied = statuses
        .iter()
        .filter(|status| **status == Status::Satisfied)
        .count();
    let reason = thresh_reason(k, n, satisfied, status);

    Diagnostic::combinator(format!("{label}({k}, {n})"), status, children)
        .with_meta("required", k.to_string())
        .with_meta("satisfied", satisfied.to_string())
        .with_reason(reason)
}
