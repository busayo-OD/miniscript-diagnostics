use miniscript::bitcoin::{self, absolute, hashes, relative, PublicKey};
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";
const HASH32_B: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH20_A: &str = "1111111111111111111111111111111111111111";
const HASH20_B: &str = "2222222222222222222222222222222222222222";

fn key(s: &str) -> bitcoin::PublicKey {
    s.parse().expect("valid test pubkey")
}

#[test]
fn satisfied_key() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&format!("pk({KEY_A})"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
}

#[test]
fn unavailable_key_is_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate(&format!("pk({KEY_A})"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Impossible);
}

#[test]
fn pkh_mirrors_pk() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let satisfied = parse_and_evaluate(&format!("pkh({KEY_A})"), &context).unwrap();
    assert_eq!(satisfied.status, Status::Satisfied);

    let empty_context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let impossible = parse_and_evaluate(&format!("pkh({KEY_A})"), &empty_context).unwrap();
    assert_eq!(impossible.status, Status::Impossible);
}

#[test]
fn satisfied_hashlock() {
    let hash: hashes::sha256::Hash = HASH32_A.parse().unwrap();
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new().with_sha256_preimage(hash);
    let diagnostic = parse_and_evaluate(&format!("sha256({HASH32_A})"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
}

#[test]
fn missing_preimage_is_unavailable() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate(&format!("sha256({HASH32_A})"), &context).unwrap();
    assert_eq!(diagnostic.status, Status::Unavailable);
}

#[test]
fn all_four_hash_types_available() {
    let sha256: hashes::sha256::Hash = HASH32_A.parse().unwrap();
    let hash256: miniscript::hash256::Hash = HASH32_B.parse().unwrap();
    let ripemd160: hashes::ripemd160::Hash = HASH20_A.parse().unwrap();
    let hash160: hashes::hash160::Hash = HASH20_B.parse().unwrap();

    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new()
        .with_sha256_preimage(sha256)
        .with_hash256_preimage(hash256)
        .with_ripemd160_preimage(ripemd160)
        .with_hash160_preimage(hash160);

    assert_eq!(
        parse_and_evaluate(&format!("sha256({HASH32_A})"), &context)
            .unwrap()
            .status,
        Status::Satisfied
    );
    assert_eq!(
        parse_and_evaluate(&format!("hash256({HASH32_B})"), &context)
            .unwrap()
            .status,
        Status::Satisfied
    );
    assert_eq!(
        parse_and_evaluate(&format!("ripemd160({HASH20_A})"), &context)
            .unwrap()
            .status,
        Status::Satisfied
    );
    assert_eq!(
        parse_and_evaluate(&format!("hash160({HASH20_B})"), &context)
            .unwrap()
            .status,
        Status::Satisfied
    );
}

#[test]
fn satisfied_absolute_timelock_block_height() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new()
        .with_chain_height(absolute::Height::from_consensus(500_000).unwrap());
    let diagnostic = parse_and_evaluate("after(499000)", &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
}

#[test]
fn unsatisfied_absolute_timelock_block_height() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new()
        .with_chain_height(absolute::Height::from_consensus(400_000).unwrap());
    let diagnostic = parse_and_evaluate("after(500000)", &context).unwrap();
    assert_eq!(diagnostic.status, Status::Unavailable);
    assert!(diagnostic
        .metadata
        .iter()
        .any(|(k, v)| k == "remaining" && v == "100000 blocks"));
}

#[test]
fn satisfied_relative_timelock_blocks() {
    let context: DiagnosticContext<PublicKey> =
        DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(200u16));
    let diagnostic = parse_and_evaluate("older(144)", &context).unwrap();
    assert_eq!(diagnostic.status, Status::Satisfied);
}

#[test]
fn unsatisfied_relative_timelock_blocks() {
    let context: DiagnosticContext<PublicKey> =
        DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(83u16));
    let diagnostic = parse_and_evaluate("older(144)", &context).unwrap();
    assert_eq!(diagnostic.status, Status::Unavailable);
    assert!(diagnostic
        .metadata
        .iter()
        .any(|(k, v)| k == "remaining" && v == "61 blocks"));
}

#[test]
fn timelock_leaf_never_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let after = parse_and_evaluate("after(500000)", &context).unwrap();
    let older = parse_and_evaluate("older(144)", &context).unwrap();
    assert_eq!(after.status, Status::Unavailable);
    assert_eq!(older.status, Status::Unavailable);
}

#[test]
fn missing_context_is_unavailable_not_a_lie() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate("after(500000)", &context).unwrap();
    assert_eq!(diagnostic.status, Status::Unavailable);
    assert!(!diagnostic.metadata.iter().any(|(k, _)| k == "remaining"));
    assert!(diagnostic.metadata.iter().any(|(k, _)| k == "note"));
}

#[test]
fn reason_text_pk_satisfied() {
    let context: DiagnosticContext<_> = DiagnosticContext::new().with_key(key(KEY_A));
    let diagnostic = parse_and_evaluate(&format!("pk({KEY_A})"), &context).unwrap();
    assert_eq!(
        diagnostic.reason.as_deref(),
        Some("the required signing key is available in the supplied context")
    );
}

#[test]
fn reason_text_pk_impossible() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let diagnostic = parse_and_evaluate(&format!("pk({KEY_A})"), &context).unwrap();
    assert_eq!(
        diagnostic.reason.as_deref(),
        Some("no signature is available for the required key, and signatures cannot be forged")
    );
}

#[test]
fn reason_text_sha256_satisfied_and_unavailable() {
    let hash: hashes::sha256::Hash = HASH32_A.parse().unwrap();
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new().with_sha256_preimage(hash);

    let satisfied_diagnostic =
        parse_and_evaluate(&format!("sha256({HASH32_A})"), &context).unwrap();
    assert_eq!(
        satisfied_diagnostic.reason.as_deref(),
        Some("the required preimage is available in the supplied context")
    );

    let empty_context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let unavailable_diagnostic =
        parse_and_evaluate(&format!("sha256({HASH32_A})"), &empty_context).unwrap();
    assert_eq!(
        unavailable_diagnostic.reason.as_deref(),
        Some("the required preimage was not supplied")
    );
}

#[test]
fn reason_text_after_and_older() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();
    let after = parse_and_evaluate("after(500000)", &context).unwrap();
    assert_eq!(
        after.reason.as_deref(),
        Some("the absolute timelock has not yet matured")
    );

    let older = parse_and_evaluate("older(144)", &context).unwrap();
    assert_eq!(
        older.reason.as_deref(),
        Some("the relative timelock has not yet matured")
    );
}
