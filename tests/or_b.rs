use miniscript::bitcoin::PublicKey;
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

fn wrap_right(inner: &str) -> String {
    format!("s:{inner}")
}

#[test]
fn left_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let expr = format!("or_b(pk({KEY_A}),{})", wrap_right(&format!("pk({KEY_B})")));
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children.len(), 2);
    assert_eq!(diagnostic.children[0].status, Status::Satisfied);
    assert_eq!(diagnostic.children[1].status, Status::Impossible);
}

#[test]
fn right_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_B));
    let expr = format!("or_b(pk({KEY_A}),{})", wrap_right(&format!("pk({KEY_B})")));
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children[0].status, Status::Impossible);
    assert_eq!(diagnostic.children[1].status, Status::Satisfied);
}

#[test]
fn both_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_key(key(KEY_B));
    let expr = format!("or_b(pk({KEY_A}),{})", wrap_right(&format!("pk({KEY_B})")));
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children[0].status, Status::Satisfied);
    assert_eq!(diagnostic.children[1].status, Status::Satisfied);
}

#[test]
fn both_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let expr = format!("or_b(pk({KEY_A}),{})", wrap_right(&format!("pk({KEY_B})")));
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.status, Status::Impossible);
}

#[test]
fn or_b_matches_or_i_status_semantics() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let or_b_expr = format!("or_b(pk({KEY_A}),{})", wrap_right(&format!("pk({KEY_B})")));
    let or_b = parse_and_evaluate(&or_b_expr, &context).unwrap();
    let or_i = parse_and_evaluate(&format!("or_i(pk({KEY_A}),pk({KEY_B}))"), &context).unwrap();

    assert_eq!(or_b.status, or_i.status);
}
