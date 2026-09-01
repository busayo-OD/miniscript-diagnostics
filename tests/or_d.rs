use miniscript::bitcoin::{relative, PublicKey};
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

#[test]
fn left_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic =
        parse_and_evaluate(&format!("or_d(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children.len(), 2);
    assert_eq!(diagnostic.children[0].status, Status::Satisfied);
    assert_eq!(diagnostic.children[1].status, Status::Unavailable);
}

#[test]
fn right_satisfied() {
    let context: DiagnosticContext<PublicKey> =
        DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(144u16));
    let diagnostic =
        parse_and_evaluate(&format!("or_d(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children[0].status, Status::Impossible);
    assert_eq!(diagnostic.children[1].status, Status::Satisfied);
}

#[test]
fn both_unavailable_or_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();

    let unavailable =
        parse_and_evaluate(&format!("or_d(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(unavailable.status, Status::Unavailable);

    let impossible =
        parse_and_evaluate(&format!("or_d(pk({KEY_A}),pk({KEY_A}))"), &context).unwrap();
    assert_eq!(impossible.status, Status::Impossible);
}

#[test]
fn or_d_matches_or_i_status_semantics() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let or_d = parse_and_evaluate(&format!("or_d(pk({KEY_A}),older(144))"), &context).unwrap();
    let or_i = parse_and_evaluate(&format!("or_i(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(or_d.status, or_i.status);
}
