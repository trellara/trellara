pub fn redact_database_url(value: &str) -> String {
    if value.to_ascii_lowercase().contains("password=") {
        return redact_keyword_password(value);
    }
    redact_uri_userinfo(value)
}

fn redact_uri_userinfo(value: &str) -> String {
    let Some(scheme_end) = value.find("://") else {
        return value.to_string();
    };
    let authority_start = scheme_end + 3;
    let Some(relative_at) = value[authority_start..].find('@') else {
        return value.to_string();
    };
    let at_index = authority_start + relative_at;
    format!(
        "{}<redacted>@{}",
        &value[..authority_start],
        &value[at_index + 1..]
    )
}

fn redact_keyword_password(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let mut output = String::new();
    let mut index = 0;
    while index < value.len() {
        if lower[index..].starts_with("password=") {
            output.push_str("password=<redacted>");
            index += "password=".len();
            index = skip_password_value(value, index);
        } else {
            let character = value[index..].chars().next().expect("character");
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}

fn skip_password_value(value: &str, mut index: usize) -> usize {
    if let Some(quote) = value[index..]
        .chars()
        .next()
        .filter(|c| *c == '\'' || *c == '"')
    {
        index += quote.len_utf8();
        while index < value.len() {
            let character = value[index..].chars().next().expect("character");
            index += character.len_utf8();
            if character == quote {
                break;
            }
        }
        return index;
    }
    while index < value.len() {
        let character = value[index..].chars().next().expect("character");
        if character.is_whitespace() {
            break;
        }
        index += character.len_utf8();
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_passwords_are_redacted() {
        let redacted =
            redact_database_url("postgresql://user:secret@db.example.com/app?sslmode=require");

        assert_eq!(
            redacted,
            "postgresql://<redacted>@db.example.com/app?sslmode=require"
        );
        assert!(!redacted.contains("secret"));
    }

    #[test]
    fn keyword_passwords_are_redacted() {
        let redacted =
            redact_database_url("host=db user=app password='with spaces' dbname=postgres");

        assert_eq!(
            redacted,
            "host=db user=app password=<redacted> dbname=postgres"
        );
        assert!(!redacted.contains("with spaces"));
    }
}
