//! `miniscript-diagnostics`: explains Miniscript satisfaction state
//! under a supplied spending context.
//!
//! ```
//! use miniscript::bitcoin;
//! use miniscript_diagnostics::{parse, DiagnosticContext, Status};
//!
//! let miniscript = parse::<bitcoin::PublicKey>("older(144)").unwrap();
//! let ctx = DiagnosticContext::new();
//! let diagnostic = miniscript_diagnostics::evaluate(&miniscript, &ctx);
//! assert_eq!(diagnostic.status, Status::Unavailable);
//! ```

mod context;
mod diagnostic;
mod error;
mod evaluator;

pub use context::DiagnosticContext;
pub use diagnostic::{Diagnostic, Status};
pub use error::Error;
pub use evaluator::evaluate;

/// Default script context used by `parse`.
pub use miniscript::Segwitv0;

pub fn parse<Pk>(source: &str) -> Result<miniscript::Miniscript<Pk, Segwitv0>, Error>
where
    Pk: miniscript::FromStrKey,
{
    Ok(miniscript::Miniscript::from_str_insane(source)?)
}

pub fn parse_and_evaluate<Pk>(
    source: &str,
    ctx: &DiagnosticContext<Pk>,
) -> Result<Diagnostic, Error>
where
    Pk: miniscript::FromStrKey,
{
    let miniscript = parse::<Pk>(source)?;
    Ok(evaluate(&miniscript, ctx))
}
