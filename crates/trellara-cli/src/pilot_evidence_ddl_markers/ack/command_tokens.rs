pub(super) fn clean_arg(command: &str, flag: &str) -> bool {
    command_arg(command, flag).is_some_and(|value| clean_arg_value(&value))
}

pub(super) fn clean_arg_value(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !value.starts_with("--")
}

pub(super) fn command_args(command: &str, flag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut tokens = command.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == flag {
            if let Some(value) = tokens.next() {
                values.push(value.to_string());
            }
        } else if let Some(value) = token.strip_prefix(&format!("{flag}=")) {
            values.push(value.to_string());
        }
    }
    values
}

pub(super) fn command_arg(command: &str, flag: &str) -> Option<String> {
    let mut tokens = command.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == flag {
            return tokens.next().map(ToString::to_string);
        }
        if let Some(value) = token.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_string());
        }
    }
    None
}
