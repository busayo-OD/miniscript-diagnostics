use miniscript::bitcoin::PublicKey;
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

#[test]
fn falls_through_to_else_branch_when_and_branch_blocked() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let expr = format!("andor(pk({KEY_A}),older(1),pk({KEY_A}))");
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.children.len(), 3);
    assert!(diagnostic
        .reason
        .as_deref()
        .unwrap()
        .contains("satisfied via the else-branch"));
}

#[test]
fn c_branch_satisfied() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new().with_key(key(KEY_B));
    let expr = format!("andor(pk({KEY_A}),older(1),pk({KEY_B}))");
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.children[0].status, Status::Impossible);
    assert_eq!(diagnostic.children[1].status, Status::Unavailable);
    assert_eq!(diagnostic.children[2].status, Status::Satisfied);
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert!(diagnostic
        .reason
        .as_deref()
        .unwrap()
        .contains("satisfied via the else-branch"));
}

#[test]
fn both_paths_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let expr = format!("andor(pk({KEY_A}),pk({KEY_A}),pk({KEY_A}))");
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.status, Status::Impossible);
    assert_eq!(
        diagnostic.reason.as_deref().unwrap(),
        "neither the and-branch nor the else-branch can be satisfied"
    );
}

#[test]
fn children_preserved_in_order() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let expr = format!("andor(pk({KEY_A}),older(1),older(2))");
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.children.len(), 3);
    assert!(diagnostic.children[0].fragment.starts_with("pk("));
    assert!(diagnostic.children[1].fragment.starts_with("older("));
    assert!(diagnostic.children[2].fragment.starts_with("older("));
}
