use miniscript::bitcoin::{self, relative, PublicKey};
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";

fn key(s: &str) -> bitcoin::PublicKey {
    s.parse().expect("valid test pubkey")
}

#[test]
fn and_v_both_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic =
        parse_and_evaluate(&format!("and_v(v:pk({KEY_A}),pk({KEY_A}))"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children.len(), 2);
    assert!(diagnostic
        .children
        .iter()
        .all(|child| child.status == Status::Satisfied));
}

#[test]
fn and_v_with_unavailable_child() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate(
        &format!("and_v(v:sha256({HASH32_A}),after(500000))"),
        &context,
    )
    .unwrap();
    assert_eq!(diagnostic.status, Status::Unavailable);
    assert_eq!(diagnostic.children[0].status, Status::Unavailable);
    assert_eq!(diagnostic.children[1].status, Status::Unavailable);
}

#[test]
fn and_v_with_impossible_child_dominates() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic =
        parse_and_evaluate(&format!("and_v(v:pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Impossible);
    assert_eq!(diagnostic.children[0].status, Status::Impossible);
    assert_eq!(diagnostic.children[1].status, Status::Unavailable);
}

#[test]
fn and_b_mirrors_and_v_semantics() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let satisfied =
        parse_and_evaluate(&format!("and_b(pk({KEY_A}),s:pk({KEY_A}))"), &context).unwrap();
    assert_eq!(satisfied.status, Status::Satisfied);

    let empty_context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let impossible =
        parse_and_evaluate(&format!("and_b(pk({KEY_A}),s:pk({KEY_A}))"), &empty_context).unwrap();
    assert_eq!(impossible.status, Status::Impossible);
}

#[test]
fn or_i_left_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic =
        parse_and_evaluate(&format!("or_i(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children.len(), 2);
    assert_eq!(diagnostic.children[0].status, Status::Satisfied);
    assert_eq!(diagnostic.children[1].status, Status::Unavailable);
}

#[test]
fn or_i_right_satisfied() {
    let context: DiagnosticContext<PublicKey> =
        DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(144u16));
    let diagnostic =
        parse_and_evaluate(&format!("or_i(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children[0].status, Status::Impossible);
    assert_eq!(diagnostic.children[1].status, Status::Satisfied);
}

#[test]
fn or_i_both_unavailable_or_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();

    let unavailable_diagnostic =
        parse_and_evaluate(&format!("or_i(pk({KEY_A}),older(144))"), &context).unwrap();
    assert_eq!(unavailable_diagnostic.status, Status::Unavailable);

    let impossible_diagnostic =
        parse_and_evaluate(&format!("or_i(pk({KEY_A}),pk({KEY_A}))"), &context).unwrap();
    assert_eq!(impossible_diagnostic.status, Status::Impossible);
}
