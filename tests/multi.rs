use miniscript::bitcoin::PublicKey;
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "0245d3b9ce0f54f4d6a17edfe3f9e0993b94d6b299c1a6e5a728ff036ecd9e139f";
const KEY_C: &str = "0257a62b05e99914350ce87639a68d0f3dd588e98afaf6c1131235a855d41962f3";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

fn multi_str(k: usize) -> String {
    format!("multi({k},{KEY_A},{KEY_B},{KEY_C})")
}

#[test]
fn exactly_k_satisfiable() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_key(key(KEY_B));
    let diagnostic = parse_and_evaluate(&multi_str(2), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(
        diagnostic.reason.as_deref().unwrap(),
        "2 of 2 required are satisfied (3 total); requirement met"
    );
}

#[test]
fn fewer_than_k_satisfiable_is_impossible() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&multi_str(2), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Impossible);
    assert!(diagnostic.metadata.iter().any(|(k, _)| k == "required"));
    assert!(diagnostic.metadata.iter().any(|(k, _)| k == "satisfied"));
    assert_eq!(
        diagnostic.reason.as_deref().unwrap(),
        "1 of 2 required are satisfied (3 total); not enough branches can ever be satisfied"
    );
}

#[test]
fn tree_preserves_all_key_children() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&multi_str(2), &context).unwrap();
    assert_eq!(diagnostic.children.len(), 3);
    assert_eq!(diagnostic.children[0].status, Status::Satisfied);
    assert_eq!(diagnostic.children[1].status, Status::Impossible);
    assert_eq!(diagnostic.children[2].status, Status::Impossible);
}

#[test]
fn multi_matches_equivalent_thresh_of_pk() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_key(key(KEY_B));
    let multi = parse_and_evaluate(&multi_str(2), &context).unwrap();
    let thresh = parse_and_evaluate(
        &format!("thresh(2,pk({KEY_A}),s:pk({KEY_B}),s:pk({KEY_C}))"),
        &context,
    )
    .unwrap();
    assert_eq!(multi.status, thresh.status);
}
