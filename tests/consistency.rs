//! Checks the invariant that every diagnostic node's status agrees with the
//! status combination rule for its fragment, applied to its own children.

use miniscript::bitcoin::{hashes, relative, PublicKey};
use miniscript::{Miniscript, Tap};
use miniscript_diagnostics::{evaluate, parse, Diagnostic, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "0245d3b9ce0f54f4d6a17edfe3f9e0993b94d6b299c1a6e5a728ff036ecd9e139f";
const KEY_C: &str = "0257a62b05e99914350ce87639a68d0f3dd588e98afaf6c1131235a855d41962f3";
const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

fn contexts() -> Vec<DiagnosticContext<PublicKey>> {
    let sha256: hashes::sha256::Hash = HASH32_A.parse().unwrap();
    vec![
        DiagnosticContext::new(),
        DiagnosticContext::new().with_key(key(KEY_A)),
        DiagnosticContext::new()
            .with_key(key(KEY_A))
            .with_key(key(KEY_B))
            .with_key(key(KEY_C)),
        DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(144u16)),
        DiagnosticContext::new()
            .with_key(key(KEY_B))
            .with_sha256_preimage(sha256),
    ]
}

fn assert_consistent(diagnostic: &Diagnostic, source: &str) {
    assert!(
        diagnostic.reason.is_some(),
        "{source}: node {} has no reason",
        diagnostic.fragment
    );

    let statuses: Vec<Status> = diagnostic.children.iter().map(|c| c.status).collect();
    let fragment = diagnostic.fragment.as_str();

    let expected = if fragment == "AND_V" || fragment == "AND_B" {
        assert_eq!(statuses.len(), 2, "{source}: {fragment} child count");
        Some(Status::combine_and(statuses[0], statuses[1]))
    } else if matches!(fragment, "OR_I" | "OR_B" | "OR_C" | "OR_D") {
        assert_eq!(statuses.len(), 2, "{source}: {fragment} child count");
        Some(Status::combine_or(statuses[0], statuses[1]))
    } else if fragment == "ANDOR" {
        assert_eq!(statuses.len(), 3, "{source}: ANDOR child count");
        Some(Status::combine_or(
            Status::combine_and(statuses[0], statuses[1]),
            statuses[2],
        ))
    } else if fragment.starts_with("THRESH(")
        || fragment.starts_with("MULTI(")
        || fragment.starts_with("MULTI_A(")
    {
        let k: usize = diagnostic
            .metadata
            .iter()
            .find(|(name, _)| name == "required")
            .map(|(_, value)| value.parse().unwrap())
            .expect("threshold node must record `required`");
        Some(Status::combine_thresh(k, &statuses))
    } else {
        assert!(
            diagnostic.children.is_empty(),
            "{source}: unexpected combinator {fragment}"
        );
        None
    };

    if let Some(expected) = expected {
        assert_eq!(diagnostic.status, expected, "{source}: node {fragment}");
    }

    for child in &diagnostic.children {
        assert_consistent(child, source);
    }
}

#[test]
fn segwit_v0_trees_agree_with_their_final_status() {
    let sources = [
        format!("andor(pk({KEY_A}),pk({KEY_B}),pk({KEY_C}))"),
        format!("andor(pk({KEY_A}),older(1),sha256({HASH32_A}))"),
        format!("andor(pk({KEY_A}),older(144),pk({KEY_B}))"),
        format!("and_n(pk({KEY_A}),older(144))"),
        format!("or_b(pk({KEY_A}),s:pk({KEY_B}))"),
        format!("or_b(pk({KEY_A}),s:sha256({HASH32_A}))"),
        format!("and_v(or_c(pk({KEY_A}),v:older(144)),older(1))"),
        format!("or_d(pk({KEY_A}),older(144))"),
        format!("or_d(pk({KEY_A}),1)"),
        format!("multi(2,{KEY_A},{KEY_B},{KEY_C})"),
        format!("and_v(v:multi(2,{KEY_A},{KEY_B},{KEY_C}),older(144))"),
        format!("thresh(2,pk({KEY_A}),s:sha256({HASH32_A}),s:pk({KEY_C}))"),
        format!("or_i(and_v(v:pk({KEY_A}),older(144)),pk({KEY_B}))"),
    ];
    let contexts = contexts();

    for source in &sources {
        let miniscript =
            parse::<PublicKey>(source).unwrap_or_else(|error| panic!("{source}: {error}"));
        for context in &contexts {
            let diagnostic = evaluate(&miniscript, context);
            assert_consistent(&diagnostic, source);
        }
    }
}

#[test]
fn tap_trees_agree_with_their_final_status() {
    let sources = [
        format!("multi_a(2,{KEY_A},{KEY_B},{KEY_C})"),
        format!("and_v(v:multi_a(2,{KEY_A},{KEY_B},{KEY_C}),older(144))"),
    ];
    let contexts = contexts();

    for source in &sources {
        let miniscript = Miniscript::<PublicKey, Tap>::from_str_insane(source)
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        for context in &contexts {
            let diagnostic = evaluate(&miniscript, context);
            assert_consistent(&diagnostic, source);
        }
    }
}
