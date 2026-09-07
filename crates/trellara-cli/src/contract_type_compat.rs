pub(crate) fn is_safe_postgres_type_widening(source: &str, target: &str) -> bool {
    matches!(
        (
            normalize_postgres_type_name(source).as_str(),
            normalize_postgres_type_name(target).as_str()
        ),
        ("int2", "int4")
            | ("int2", "int8")
            | ("int2", "numeric")
            | ("int4", "int8")
            | ("int4", "numeric")
            | ("int8", "numeric")
            | ("float4", "float8")
            | ("varchar", "text")
            | ("bpchar", "text")
            | ("bpchar", "varchar")
            | ("date", "timestamp")
            | ("date", "timestamptz")
            | ("timestamp", "timestamptz")
    )
}

fn normalize_postgres_type_name(type_name: &str) -> String {
    let lower = type_name.trim().to_ascii_lowercase();
    let base = lower.split('(').next().unwrap_or(&lower).trim();
    match base {
        "smallint" | "int2" => "int2".to_string(),
        "integer" | "int" | "int4" => "int4".to_string(),
        "bigint" | "int8" => "int8".to_string(),
        "decimal" | "numeric" => "numeric".to_string(),
        "real" | "float4" => "float4".to_string(),
        "double precision" | "float8" => "float8".to_string(),
        "character varying" | "varchar" => "varchar".to_string(),
        "character" | "char" | "bpchar" => "bpchar".to_string(),
        "timestamp without time zone" | "timestamp" => "timestamp".to_string(),
        "timestamp with time zone" | "timestamptz" => "timestamptz".to_string(),
        other => other.to_string(),
    }
}
