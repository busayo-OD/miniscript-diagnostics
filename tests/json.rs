use miniscript::bitcoin;
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const KEY_B: &str = "0245d3b9ce0f54f4d6a17edfe3f9e0993b94d6b299c1a6e5a728ff036ecd9e139f";
const KEY_C: &str = "0257a62b05e99914350ce87639a68d0f3dd588e98afaf6c1131235a855d41962f3";
const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";

fn key(s: &str) -> bitcoin::PublicKey {
    s.parse().expect("valid test pubkey")
}

#[test]
fn satisfied_leaf_serializes() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&format!("pk({KEY_A})"), &context).unwrap();

    let json: serde_json::Value = serde_json::to_value(&diagnostic).unwrap();
    assert_eq!(json["status"], "satisfied");
    assert_eq!(json["fragment"], diagnostic.fragment);
    assert_eq!(
        json["reason"],
        "the required signing key is available in the supplied context"
    );
    assert_eq!(json["children"], serde_json::json!([]));
}

#[test]
fn unavailable_leaf_serializes() {
    let context: DiagnosticContext<miniscript::bitcoin::PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate("older(144)", &context).unwrap();

    let json: serde_json::Value = serde_json::to_value(&diagnostic).unwrap();
    assert_eq!(json["status"], "unavailable");
    assert_eq!(json["fragment"], "older(144)");
    // metadata preserves the Vec<(String, String)> shape: array of pairs.
    let metadata = json["metadata"].as_array().unwrap();
    assert!(metadata.iter().any(|pair| pair[0] == "required"));
}

#[test]
fn impossible_leaf_serializes() {
    let context: DiagnosticContext<miniscript::bitcoin::PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate(&format!("pk({KEY_A})"), &context).unwrap();

    let json: serde_json::Value = serde_json::to_value(&diagnostic).unwrap();
    assert_eq!(json["status"], "impossible");
    assert_eq!(
        json["reason"],
        "no signature is available for the required key, and signatures cannot be forged"
    );
}

#[test]
fn nested_tree_preserves_all_children_in_json() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let miniscript = format!("or_i(and_v(v:pk({KEY_A}),older(144)),pk({KEY_B}))");
    let diagnostic = parse_and_evaluate(&miniscript, &context).unwrap();

    let json: serde_json::Value = serde_json::to_value(&diagnostic).unwrap();
    assert_eq!(json["fragment"], "OR_I");
    let children = json["children"].as_array().unwrap();
    assert_eq!(children.len(), 2);

    let and_v = &children[0];
    assert_eq!(and_v["fragment"], "AND_V");
    let and_v_children = and_v["children"].as_array().unwrap();
    assert_eq!(
        and_v_children.len(),
        2,
        "nested and_v must keep both children in JSON too"
    );
    assert_eq!(and_v_children[0]["status"], "satisfied");
    assert_eq!(and_v_children[1]["status"], "unavailable");
}

#[test]
fn threshold_json_preserves_child_structure() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let miniscript = format!("thresh(2,pk({KEY_A}),s:sha256({HASH32_A}),s:pk({KEY_C}))");
    let diagnostic = parse_and_evaluate(&miniscript, &context).unwrap();

    let json: serde_json::Value = serde_json::to_value(&diagnostic).unwrap();
    assert!(json["fragment"].as_str().unwrap().starts_with("THRESH("));
    let metadata = json["metadata"].as_array().unwrap();
    assert!(metadata
        .iter()
        .any(|pair| pair[0] == "required" && pair[1] == "2"));

    let children = json["children"].as_array().unwrap();
    assert_eq!(children.len(), 3);
    assert_eq!(children[0]["status"], "satisfied");
    assert_eq!(children[1]["status"], "unavailable");
    assert_eq!(children[2]["status"], "impossible");
}

#[test]
fn round_trips_through_status_display_values() {
    // Sanity check that all four Status variants serialize to the expected
    // lowercase strings (the JSON contract), independent of any one fragment.
    assert_eq!(
        serde_json::to_value(Status::Satisfied).unwrap(),
        "satisfied"
    );
    assert_eq!(
        serde_json::to_value(Status::Unavailable).unwrap(),
        "unavailable"
    );
    assert_eq!(
        serde_json::to_value(Status::Impossible).unwrap(),
        "impossible"
    );
    assert_eq!(
        serde_json::to_value(Status::Unsupported).unwrap(),
        "unsupported"
    );
}
