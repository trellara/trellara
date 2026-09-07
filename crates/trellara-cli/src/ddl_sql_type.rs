pub(crate) fn is_safe_postgres_type(sql_type: &str) -> bool {
    let Some((base, args)) = split_type_args(sql_type.trim()) else {
        return false;
    };
    let base = base.trim();
    if base.is_empty() || !base_is_safe(base) {
        return false;
    }
    match args {
        Some(args) => type_args_are_safe(args),
        None => true,
    }
}

fn split_type_args(sql_type: &str) -> Option<(&str, Option<&str>)> {
    if let Some(open) = sql_type.find('(') {
        let close = sql_type.strip_suffix(')')?;
        if close[open + 1..].contains(['(', ')']) {
            return None;
        }
        return Some((&close[..open], Some(&close[open + 1..])));
    }
    (!sql_type.contains(')')).then_some((sql_type, None))
}

fn base_is_safe(base: &str) -> bool {
    if base.contains(' ') {
        return matches!(
            base,
            "character varying"
                | "double precision"
                | "timestamp with time zone"
                | "timestamp without time zone"
                | "time with time zone"
                | "time without time zone"
        );
    }
    base.split('.').all(is_safe_postgres_ident)
}

fn type_args_are_safe(args: &str) -> bool {
    !args.trim().is_empty()
        && args.split(',').all(|arg| {
            arg.trim()
                .chars()
                .all(|character| character.is_ascii_digit())
        })
}

pub(crate) fn is_safe_postgres_ident(identifier: &str) -> bool {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_type_grammar_allows_common_column_types() {
        for sql_type in [
            "text",
            "numeric(10, 2)",
            "character varying(255)",
            "timestamp without time zone",
            "public.money_amount",
        ] {
            assert!(is_safe_postgres_type(sql_type), "{sql_type}");
        }
    }

    #[test]
    fn safe_type_grammar_rejects_extra_clauses() {
        for sql_type in [
            "text default now()",
            "numeric(10, 2) not null",
            "text check (value <> '')",
            "text collate \"C\"",
        ] {
            assert!(!is_safe_postgres_type(sql_type), "{sql_type}");
        }
    }
}
