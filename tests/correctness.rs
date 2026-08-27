use miniscript::bitcoin::{absolute, relative, PublicKey};
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext, Status};

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const HASH32_A: &str = "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5330";

#[test]
fn after_matches_bitcoin_absolute_locktime_is_satisfied_by() {
    let cases: &[(u32, u32, bool)] = &[
        (500_000, 500_000, true),
        (500_000, 499_999, false),
        (500_000, 500_001, true),
        (1, 1, true),
    ];
    for &(required, current, expect_satisfied) in cases {
        let context: DiagnosticContext<PublicKey> = DiagnosticContext::new()
            .with_chain_height(absolute::Height::from_consensus(current).unwrap());
        let reference_satisfied = absolute::LockTime::from_height(required)
            .unwrap()
            .is_satisfied_by(
                absolute::Height::from_consensus(current).unwrap(),
                absolute::Time::MIN,
            );
        assert_eq!(
            reference_satisfied, expect_satisfied,
            "test table itself is wrong for required={required} current={current}"
        );

        let diagnostic = parse_and_evaluate(&format!("after({required})"), &context).unwrap();
        let evaluator_satisfied = diagnostic.status == Status::Satisfied;
        assert_eq!(
            evaluator_satisfied, expect_satisfied,
            "evaluator disagreed with bitcoin::absolute::LockTime::is_satisfied_by for required={required} current={current}"
        );
    }
}

#[test]
fn older_matches_bitcoin_relative_locktime_is_satisfied_by() {
    let cases: &[(u16, u16, bool)] = &[
        (144, 144, true),
        (144, 143, false),
        (144, 200, true),
        (1, 1, true),
    ];
    for &(required, elapsed, expect_satisfied) in cases {
        let context: DiagnosticContext<PublicKey> =
            DiagnosticContext::new().with_elapsed_blocks(relative::Height::from(elapsed));
        let reference_satisfied = relative::LockTime::from_height(required)
            .is_satisfied_by(relative::Height::from(elapsed), relative::Time::ZERO);
        assert_eq!(
            reference_satisfied, expect_satisfied,
            "test table itself is wrong for required={required} elapsed={elapsed}"
        );

        let diagnostic = parse_and_evaluate(&format!("older({required})"), &context).unwrap();
        let evaluator_satisfied = diagnostic.status == Status::Satisfied;
        assert_eq!(
            evaluator_satisfied, expect_satisfied,
            "evaluator disagreed with bitcoin::relative::LockTime::is_satisfied_by for required={required} elapsed={elapsed}"
        );
    }
}

#[test]
fn pk_impossible_vs_hash_unavailable_documented_distinction_holds() {
    let context: DiagnosticContext<PublicKey> = DiagnosticContext::new();

    let pk = parse_and_evaluate(&format!("pk({KEY_A})"), &context).unwrap();
    assert_eq!(pk.status, Status::Impossible);

    let pkh = parse_and_evaluate(&format!("pkh({KEY_A})"), &context).unwrap();
    assert_eq!(pkh.status, Status::Impossible);

    let sha256 = parse_and_evaluate(&format!("sha256({HASH32_A})"), &context).unwrap();
    assert_eq!(sha256.status, Status::Unavailable);
}

#[test]
fn and_combinator_dominance_matches_witness_combine() {
    assert_eq!(
        Status::combine_and(Status::Impossible, Status::Unavailable),
        Status::Impossible
    );
    assert_eq!(
        Status::combine_and(Status::Unavailable, Status::Impossible),
        Status::Impossible
    );
    assert_eq!(
        Status::combine_and(Status::Impossible, Status::Satisfied),
        Status::Impossible
    );
    assert_eq!(
        Status::combine_and(Status::Unavailable, Status::Satisfied),
        Status::Unavailable
    );
    assert_eq!(
        Status::combine_and(Status::Satisfied, Status::Satisfied),
        Status::Satisfied
    );
}

#[test]
fn or_combinator_is_the_documented_dual_of_and() {
    assert_eq!(
        Status::combine_or(Status::Satisfied, Status::Impossible),
        Status::Satisfied
    );
    assert_eq!(
        Status::combine_or(Status::Impossible, Status::Satisfied),
        Status::Satisfied
    );
    assert_eq!(
        Status::combine_or(Status::Unavailable, Status::Impossible),
        Status::Unavailable
    );
    assert_eq!(
        Status::combine_or(Status::Impossible, Status::Impossible),
        Status::Impossible
    );
}
