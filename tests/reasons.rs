use miniscript::bitcoin::{relative, PublicKey};
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "0245d3b9ce0f54f4d6a17edfe3f9e0993b94d6b299c1a6e5a728ff036ecd9e139f";
const KEY_C: &str = "0257a62b05e99914350ce87639a68d0f3dd588e98afaf6c1131235a855d41962f3";
const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

#[test]
fn and_satisfied_plus_unavailable_names_both_states() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_elapsed_blocks(relative::Height::from(83u16));
    let diagnostic =
        parse_and_evaluate(&format!("and_v(v:pk({KEY_A}),older(144))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with("not currently satisfied"));
    assert!(reason.contains(&format!("pk({KEY_A}) is satisfied")));
    assert!(reason.contains("older(144) is unavailable"));
    assert!(reason.contains("has not yet matured"));
}

#[test]
fn and_impossible_plus_unavailable_distinguishes_the_two() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic =
        parse_and_evaluate(&format!("and_v(v:pk({KEY_A}),older(144))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Impossible);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains(&format!("pk({KEY_A}) is impossible in the current context")));
    assert!(reason.contains(&format!("no key for {KEY_A} was supplied in the context")));
    assert!(reason.contains("older(144) is unavailable"));
    assert!(!reason.contains("cannot be satisfied"));
    assert!(!reason.contains("forged"));
}

#[test]
fn and_unsupported_child_is_not_reported_as_not_satisfied() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate("and_v(v:older(144),1)", &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unsupported);
    let unsupported = diagnostic
        .children
        .iter()
        .find(|child| child.status == Status::Unsupported)
        .expect("fixture must contain an unsupported child");
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with("cannot currently be evaluated"));
    assert!(reason.contains(&format!(
        "{} cannot currently be evaluated",
        unsupported.fragment
    )));
    assert!(reason.contains("not currently supported"));
    assert!(reason.contains("older(144) is unavailable"));
    assert!(!reason.contains("is not satisfied"));
}

#[test]
fn or_unavailable_and_impossible_shows_both_facts() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();

    for policy in [
        format!("or_i(older(100),pk({KEY_A}))"),
        format!("or_i(pk({KEY_A}),older(100))"),
    ] {
        let diagnostic = parse_and_evaluate(&policy, &context).unwrap();
        assert_eq!(diagnostic.status, Status::Unavailable, "{policy}");

        let reason = diagnostic.reason.as_deref().unwrap();
        assert!(
            reason.contains("older(100) could still become available"),
            "{policy}: {reason}"
        );
        assert!(
            reason.contains(&format!("pk({KEY_A}) is impossible in the current context")),
            "{policy}: {reason}"
        );
        assert!(reason.contains("no key for"), "{policy}: {reason}");

        assert!(diagnostic.render().contains(reason));
    }
}

#[test]
fn or_both_impossible_names_both_keys() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic =
        parse_and_evaluate(&format!("or_i(pk({KEY_A}),pk({KEY_B}))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Impossible);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with("no branch is currently satisfied"));
    assert!(reason.contains(&format!("no key for {KEY_A} was supplied in the context")));
    assert!(reason.contains(&format!("no key for {KEY_B} was supplied in the context")));
    assert!(!reason.contains("cannot be satisfied"));
}

#[test]
fn or_unsupported_branch_is_named_and_not_reported_as_pending() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate("or_i(older(144),1)", &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unsupported);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with("cannot currently be evaluated"));
    assert!(reason.contains("not currently supported"));
    assert!(reason.contains("older(144) is unavailable"));
    assert!(!reason.contains("could still become available"));
    assert!(!reason.contains("project"));
    assert!(!reason.contains("MVP"));
}

#[test]
fn or_satisfied_branch_takes_precedence_over_unsupported_branch() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&format!("or_i(pk({KEY_A}),1)"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(
        diagnostic.reason.as_deref().unwrap(),
        format!("satisfied via pk({KEY_A})").as_str()
    );
}

#[test]
fn andor_unsupported_and_branch_names_the_unsupported_child() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic =
        parse_and_evaluate(&format!("andor(pk({KEY_A}),1,pk({KEY_B}))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unsupported);
    let unsupported_child = &diagnostic.children[1];
    assert_eq!(unsupported_child.status, Status::Unsupported);

    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with("cannot currently be evaluated"));
    assert!(reason.contains("and-branch:"));
    assert!(reason.contains(&format!(
        "{} cannot currently be evaluated",
        unsupported_child.fragment
    )));
    assert!(reason.contains("not currently supported"));
    assert!(reason.contains(&format!(
        "else-branch: pk({KEY_B}) is impossible in the current context"
    )));
}

#[test]
fn andor_impossible_and_branch_with_unavailable_else_branch() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic =
        parse_and_evaluate(&format!("andor(pk({KEY_A}),older(1),older(2))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains("older(2) could still become available"));
    assert!(reason.contains("the and-branch is impossible in the current context"));
    assert!(reason.contains(&format!("pk({KEY_A}) is impossible in the current context")));
}

#[test]
fn andor_unavailable_and_branch_with_impossible_else_branch() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(
        &format!("andor(pk({KEY_A}),older(1),pk({KEY_B}))"),
        &context,
    )
    .unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains("could still become available, while the else-branch"));
    assert!(reason.contains(&format!("pk({KEY_B}) is impossible in the current context")));
    assert!(!reason.contains("cannot be satisfied"));
}

#[test]
fn thresh_unavailable_names_the_reachable_condition() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(
        &format!("thresh(2,pk({KEY_A}),s:sha256({HASH32_A}),s:pk({KEY_C}))"),
        &context,
    )
    .unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.starts_with(
        "1 of 3 conditions satisfied; 2 required; 1 more needed and still reachable via: sha256("
    ));
    assert!(!reason.contains("signature"));
    assert!(!reason.contains("branches"));
}

#[test]
fn thresh_impossible_names_the_impossible_conditions() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(
        &format!("thresh(2,pk({KEY_A}),s:pk({KEY_B}),s:pk({KEY_C}))"),
        &context,
    )
    .unwrap();

    assert_eq!(diagnostic.status, Status::Impossible);
    assert_eq!(
        diagnostic.reason.as_deref().unwrap(),
        format!(
            "1 of 3 conditions satisfied; 2 required; not enough remain possible in the current context (impossible: pk({KEY_B}), pk({KEY_C}))"
        )
        .as_str()
    );
}

#[test]
fn json_reason_matches_the_rendered_reason_for_mixed_or() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic =
        parse_and_evaluate(&format!("or_i(older(100),pk({KEY_A}))"), &context).unwrap();

    let json: serde_json::Value = serde_json::to_value(&diagnostic).unwrap();
    let json_reason = json["reason"].as_str().unwrap();

    assert_eq!(Some(json_reason), diagnostic.reason.as_deref());
    assert!(json_reason.contains("older(100) could still become available"));
    assert!(diagnostic.render().contains(json_reason));

    // Child reasons survive serialization too.
    let pk_child = &json["children"][1];
    assert_eq!(pk_child["status"], "impossible");
    assert_eq!(
        pk_child["reason"],
        format!("no key for {KEY_A} was supplied in the context")
    );
}
