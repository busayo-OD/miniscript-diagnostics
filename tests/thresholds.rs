use miniscript::bitcoin::PublicKey;
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "0245d3b9ce0f54f4d6a17edfe3f9e0993b94d6b299c1a6e5a728ff036ecd9e139f";
const KEY_C: &str = "0257a62b05e99914350ce87639a68d0f3dd588e98afaf6c1131235a855d41962f3";
const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

fn thresh_str(k: usize) -> String {
    let script = format!(
        "thresh({},pk({}),s:pk({}),s:pk({}))",
        k, KEY_A, KEY_B, KEY_C
    );
    script
}

#[test]
fn exactly_k_satisfiable() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_key(key(KEY_B));
    let diagnostic = parse_and_evaluate(&thresh_str(2), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
}

#[test]
fn more_than_k_satisfiable() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_key(key(KEY_B))
        .with_key(key(KEY_C));
    let diagnostic = parse_and_evaluate(&thresh_str(2), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
}

#[test]
fn fewer_than_k_satisfiable_is_impossible() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&thresh_str(2), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Impossible);
    assert!(diagnostic.metadata.iter().any(|(k, _)| k == "required"));
    assert!(diagnostic.metadata.iter().any(|(k, _)| k == "satisfied"));
}

#[test]
fn mixed_statuses_unavailable_when_reachable() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let miniscript = format!("thresh(2,pk({KEY_A}),s:sha256({HASH32_A}),s:pk({KEY_C}))");
    let diagnostic = parse_and_evaluate(&miniscript, &context).unwrap();
    assert_eq!(diagnostic.status, Status::Unavailable);

    let child_statuses: Vec<_> = diagnostic
        .children
        .iter()
        .map(|child| child.status)
        .collect();

    assert!(child_statuses.contains(&Status::Satisfied));
    assert!(child_statuses.contains(&Status::Unavailable));
    assert!(child_statuses.contains(&Status::Impossible));
}

#[test]
fn thresh_tree_preserves_all_children() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&thresh_str(2), &context).unwrap();
    assert_eq!(diagnostic.children.len(), 3);
    assert_eq!(diagnostic.children[0].status, Status::Satisfied);
    assert_eq!(diagnostic.children[1].status, Status::Impossible);
    assert_eq!(diagnostic.children[2].status, Status::Impossible);
}

#[test]
fn thresh_k_equals_n_matches_and_chain() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));

    let thresh_all = parse_and_evaluate(
        &format!("thresh(3,pk({KEY_A}),s:pk({KEY_B}),s:pk({KEY_C}))"),
        &context,
    )
    .unwrap();
    let and_chain = parse_and_evaluate(
        &format!("and_v(v:pk({KEY_A}),and_v(v:pk({KEY_B}),pk({KEY_C})))"),
        &context,
    )
    .unwrap();

    assert_eq!(thresh_all.status, and_chain.status);
}

#[test]
fn thresh_k_equals_1_matches_or_i() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));

    let thresh_any =
        parse_and_evaluate(&format!("thresh(1,pk({KEY_A}),s:pk({KEY_B}))"), &context).unwrap();
    let or_i = parse_and_evaluate(&format!("or_i(pk({KEY_A}),pk({KEY_B}))"), &context).unwrap();

    assert_eq!(thresh_any.status, or_i.status);
}
