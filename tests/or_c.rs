use miniscript::bitcoin::{relative, PublicKey};
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

fn wrap(inner: &str) -> String {
    format!("and_v({inner},1)")
}

#[test]
fn left_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic =
        parse_and_evaluate(&wrap(&format!("or_c(pk({KEY_A}),v:older(144))")), &context).unwrap();
    let or_c = &diagnostic.children[0];
    assert_eq!(or_c.status, Status::Satisfied);
    assert_eq!(or_c.children.len(), 2);
    assert_eq!(or_c.children[0].status, Status::Satisfied);
    assert_eq!(or_c.children[1].status, Status::Unavailable);
    assert!(or_c.reason.as_deref().unwrap().contains("satisfied via"));
}

#[test]
fn right_satisfied() {
    let context: DiagnosticContext<PublicKey> =
        DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(144u16));
    let diagnostic =
        parse_and_evaluate(&wrap(&format!("or_c(pk({KEY_A}),v:older(144))")), &context).unwrap();
    let or_c = &diagnostic.children[0];
    assert_eq!(or_c.status, Status::Satisfied);
    assert_eq!(or_c.children[0].status, Status::Impossible);
    assert_eq!(or_c.children[1].status, Status::Satisfied);
}

#[test]
fn both_unavailable_or_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();

    let unavailable =
        parse_and_evaluate(&wrap(&format!("or_c(pk({KEY_A}),v:older(144))")), &context).unwrap();
    assert_eq!(unavailable.children[0].status, Status::Unavailable);

    let impossible =
        parse_and_evaluate(&wrap(&format!("or_c(pk({KEY_A}),v:pk({KEY_A}))")), &context).unwrap();
    assert_eq!(impossible.children[0].status, Status::Impossible);
}

#[test]
fn or_c_matches_or_i_status_semantics() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let or_c_diagnostic =
        parse_and_evaluate(&wrap(&format!("or_c(pk({KEY_A}),v:older(144))")), &context).unwrap();
    let or_i_diagnostic =
        parse_and_evaluate(&format!("or_i(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(or_c_diagnostic.children[0].status, or_i_diagnostic.status);
}

const KEY_B: &str = "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";

// `or_c` has type V, so it cannot be a root. These tests wrap it as
// `and_v(or_c(..), older(1))`: the sibling is a supported fragment, so the
// root status is meaningful, unlike the `and_v(.., 1)` wrapper above.
fn with_sibling(inner: &str) -> String {
    format!("and_v({inner},older(1))")
}

#[test]
fn root_is_satisfied_when_or_c_and_sibling_are_satisfied() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_elapsed_blocks(relative::Height::from(1u16));
    let expr = with_sibling(&format!("or_c(pk({KEY_A}),v:older(144))"));
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.fragment, "AND_V");
    assert_eq!(diagnostic.status, Status::Satisfied);

    let or_c = &diagnostic.children[0];
    assert_eq!(or_c.fragment, "OR_C");
    assert_eq!(or_c.status, Status::Satisfied);
    assert_eq!(
        or_c.reason.as_deref().unwrap(),
        format!("satisfied via pk({KEY_A})").as_str()
    );
    assert_eq!(diagnostic.children[1].fragment, "older(1)");
    assert_eq!(diagnostic.children[1].status, Status::Satisfied);
}

#[test]
fn impossible_or_c_makes_the_root_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let expr = with_sibling(&format!("or_c(pk({KEY_A}),v:pk({KEY_B}))"));
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    let or_c = &diagnostic.children[0];
    assert_eq!(or_c.status, Status::Impossible);
    assert!(or_c
        .reason
        .as_deref()
        .unwrap()
        .starts_with("no branch is currently satisfied"));
    assert_eq!(diagnostic.children[1].status, Status::Unavailable);
    assert_eq!(diagnostic.status, Status::Impossible);
}
