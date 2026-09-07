use super::super::*;

#[test]
fn pgoutput_decoder_accepts_repeated_matching_relation_metadata() {
    let mut decoder = PgOutputDecoder::default();
    let relation = pgoutput_relation_message(
        16_384,
        "public",
        "sales",
        b'd',
        &[("id", 25, true), ("amount_cents", 20, false)],
    );

    let first = decoder
        .decode(&relation)
        .expect("first relation")
        .expect("relation metadata event");
    let repeated = decoder
        .decode(&relation)
        .expect("repeated relation")
        .expect("repeated relation metadata event");

    match (first, repeated) {
        (
            LogicalEvent::RelationMetadata {
                relation: first_relation,
                schema_fingerprint: first_fingerprint,
            },
            LogicalEvent::RelationMetadata {
                relation: repeated_relation,
                schema_fingerprint: repeated_fingerprint,
            },
        ) => {
            assert_eq!(first_relation, RelationId::new(16_384, "public", "sales"));
            assert_eq!(repeated_relation, first_relation);
            assert_eq!(repeated_fingerprint, first_fingerprint);
        }
        other => panic!("unexpected relation metadata events {other:?}"),
    }
}
