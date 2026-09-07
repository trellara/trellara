use super::tests_pgoutput_builders::*;
use super::*;

mod invalid_tuple_tags;
mod key_identity;

fn decoder_with_sales_relation() -> PgOutputDecoder {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true)],
        ))
        .expect("relation");
    decoder
}
