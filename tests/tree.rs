use miniscript::bitcoin::{relative, PublicKey};
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "0245d3b9ce0f54f4d6a17edfe3f9e0993b94d6b299c1a6e5a728ff036ecd9e139f";

fn key(s: &str) -> PublicKey {
    s.parse().expect("valid test pubkey")
}

#[test]
fn and_v_preserves_both_children_even_when_one_fails() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_elapsed_blocks(relative::Height::from(83u16));
    let diagnostic =
        parse_and_evaluate(&format!("and_v(v:pk({KEY_A}),older(144))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    assert_eq!(
        diagnostic.children.len(),
        2,
        "and_v must retain both children"
    );

    let pk_child = &diagnostic.children[0];
    assert_eq!(pk_child.status, Status::Satisfied);
    assert!(pk_child.fragment.starts_with("pk("));

    let older_child = &diagnostic.children[1];
    assert_eq!(older_child.status, Status::Unavailable);
    assert!(older_child.fragment.starts_with("older("));
    assert!(older_child.metadata.iter().any(|(k, _)| k == "required"));
    assert!(older_child.metadata.iter().any(|(k, _)| k == "remaining"));
}

#[test]
fn or_i_preserves_the_losing_branch() {
    let context: DiagnosticContext<PublicKey> =
        DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(144u16));
    let diagnostic =
        parse_and_evaluate(&format!("or_i(pk({KEY_A}),older(144))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Satisfied);
    assert_eq!(diagnostic.children.len(), 2);
    let losing_branch = &diagnostic.children[0];
    assert_eq!(losing_branch.status, Status::Impossible);
    assert!(
        losing_branch.reason.is_some(),
        "losing branch must retain a reason, not be discarded"
    );
}

#[test]
fn nested_combinators_preserve_full_depth() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let miniscript = format!("or_i(and_v(v:pk({KEY_A}),older(144)),pk({KEY_B}))");
    let diagnostic = parse_and_evaluate(&miniscript, &context).unwrap();

    assert_eq!(diagnostic.fragment, "OR_I");
    assert_eq!(diagnostic.children.len(), 2);

    let and_v_branch = &diagnostic.children[0];
    assert_eq!(and_v_branch.fragment, "AND_V");
    assert_eq!(
        and_v_branch.children.len(),
        2,
        "nested and_v must also retain its own children"
    );
    assert_eq!(and_v_branch.children[0].status, Status::Satisfied);
    assert_eq!(and_v_branch.children[1].status, Status::Unavailable);
    assert_eq!(and_v_branch.status, Status::Unavailable);

    let pk_b_branch = &diagnostic.children[1];
    assert_eq!(pk_b_branch.status, Status::Impossible);

    assert_eq!(diagnostic.status, Status::Unavailable);
}

#[test]
fn render_includes_both_branch_labels_and_metadata() {
    let context: DiagnosticContext<_> = DiagnosticContext::new()
        .with_key(key(KEY_A))
        .with_elapsed_blocks(relative::Height::from(83u16));
    let diagnostic =
        parse_and_evaluate(&format!("and_v(v:pk({KEY_A}),older(144))"), &context).unwrap();
    let rendered = diagnostic.render();
    assert!(rendered.contains("AND_V"));
    assert!(rendered.contains("pk("));
    assert!(rendered.contains("older(144)"));
    assert!(rendered.contains("SATISFIED"));
    assert!(rendered.contains("UNAVAILABLE"));
    assert!(rendered.contains("required: 144 blocks"));
    assert!(rendered.contains("remaining: 61 blocks"));
}

#[test]
fn thresh_nested_inside_and_v_preserves_children() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let miniscript = format!("and_v(v:thresh(1,pk({KEY_A}),s:pk({KEY_B})),older(144))");
    let diagnostic = parse_and_evaluate(&miniscript, &context).unwrap();

    assert_eq!(diagnostic.fragment, "AND_V");
    let thresh_branch = &diagnostic.children[0];
    assert!(thresh_branch.fragment.starts_with("THRESH("));
    assert_eq!(
        thresh_branch.children.len(),
        2,
        "nested thresh must retain its own children"
    );
    assert_eq!(thresh_branch.status, Status::Satisfied);
}

#[test]
fn combinator_reasons_are_populated_and_rendered() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let miniscript = format!("or_i(and_v(v:pk({KEY_A}),older(144)),pk({KEY_B}))");
    let diagnostic = parse_and_evaluate(&miniscript, &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    assert!(diagnostic.reason.is_some());
    let or_i_reason = diagnostic.reason.as_deref().unwrap();
    assert!(or_i_reason.contains("could still become available"));

    let and_v_branch = &diagnostic.children[0];
    assert!(and_v_branch.reason.is_some());
    let and_v_reason = and_v_branch.reason.as_deref().unwrap();
    assert!(and_v_reason.contains("older("));

    // reason text must appear in rendered output, not just the struct
    let rendered = diagnostic.render();
    assert!(rendered.contains(or_i_reason));
    assert!(rendered.contains(and_v_reason));
}

#[test]
fn and_v_neither_side_satisfied_names_both_blockers() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic =
        parse_and_evaluate(&format!("and_v(v:pk({KEY_A}),older(144))"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Impossible);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains("pk("));
    assert!(reason.contains("older(144)"));
    assert!(reason.contains("no signature is available"));
    assert!(reason.contains("has not yet matured"));
}

#[test]
fn and_v_with_unsupported_child_names_it_specifically() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate("and_v(v:older(144),1)", &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unsupported);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains("true"));
    assert!(reason.contains("outside this project's supported"));
}

#[test]
fn or_i_both_unavailable_names_both_pending_branches() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate("or_i(older(100),older(200))", &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unavailable);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains("older(100)"));
    assert!(reason.contains("older(200)"));
}

#[test]
fn or_i_with_unsupported_branch_is_not_reported_as_pending() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate("or_i(older(144),1)", &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unsupported);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains("true"));
    assert!(!reason.contains("could still become available"));
}

#[test]
fn andor_with_unsupported_else_branch_names_it() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic =
        parse_and_evaluate(&format!("andor(pk({KEY_A}),older(1),1)"), &context).unwrap();

    assert_eq!(diagnostic.status, Status::Unsupported);
    let reason = diagnostic.reason.as_deref().unwrap();
    assert!(reason.contains("true"));
    assert!(reason.contains("else-branch"));
}
