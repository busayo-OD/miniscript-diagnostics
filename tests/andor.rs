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

    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with("no branch is currently satisfied"));
    assert!(reason.contains("and-branch"));
    assert!(reason.contains("else-branch"));
    assert!(reason.contains(&format!("no key for {KEY_A} was supplied in the context")));
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

const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";

#[test]
fn and_branch_satisfied_takes_precedence_over_satisfied_else_branch() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_key(key(KEY_B));
    let expr = format!("andor(pk({KEY_A}),pk({KEY_B}),pk({KEY_A}))");
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.fragment, "ANDOR");
    assert_eq!(diagnostic.status, Status::Satisfied);
    assert!(diagnostic
        .children
        .iter()
        .all(|child| child.status == Status::Satisfied));
    assert_eq!(
        diagnostic.reason.as_deref().unwrap(),
        format!("satisfied via pk({KEY_A}) and pk({KEY_B})").as_str()
    );
}

#[test]
fn and_branch_and_else_branch_both_unavailable() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new().with_key(key(KEY_A));
    let expr = format!("andor(pk({KEY_A}),older(1),sha256({HASH32_A}))");
    let diagnostic = parse_and_evaluate(&expr, &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    assert_eq!(diagnostic.children[0].status, Status::Satisfied);
    assert_eq!(diagnostic.children[1].status, Status::Unavailable);
    assert_eq!(diagnostic.children[2].status, Status::Unavailable);

    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with("not currently satisfied, but the and-branch (pk("));
    assert!(reason.contains("older(1)"));
    assert!(reason.contains("or the else-branch sha256("));
    assert!(reason.ends_with("could still become available"));
}

#[test]
fn and_n_is_unsupported_unless_its_and_branch_is_satisfied() {
    // Known limitation, pinned deliberately: `and_n(X,Y)` is `andor(X,Y,0)` and
    // the `0` fragment is not supported. Unless the and-branch is satisfied,
    // the result is UNSUPPORTED rather than UNAVAILABLE or IMPOSSIBLE.
    // If `0` support is added later, update this test with that change.
    let expr = format!("and_n(pk({KEY_A}),older(1))");

    let satisfied_context: DiagnosticContext<PublicKey> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_elapsed_blocks(miniscript::bitcoin::relative::Height::from(1u16));
    let satisfied = parse_and_evaluate(&expr, &satisfied_context).unwrap();
    assert_eq!(satisfied.status, Status::Satisfied);

    let empty_context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let unsupported = parse_and_evaluate(&expr, &empty_context).unwrap();
    assert_eq!(unsupported.status, Status::Unsupported);
    assert_eq!(unsupported.children[2].fragment, "false");
    assert_eq!(unsupported.children[2].status, Status::Unsupported);
}
