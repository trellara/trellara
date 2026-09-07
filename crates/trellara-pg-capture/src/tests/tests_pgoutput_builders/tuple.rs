use super::primitives::{put_sized, put_u16};

pub(in crate::tests) enum TupleValue<'a> {
    Null,
    Unchanged,
    Text(&'a str),
    Binary(&'a [u8]),
}

impl<'a> TupleValue<'a> {
    pub(in crate::tests) fn text(value: &'a str) -> Self {
        Self::Text(value)
    }

    pub(in crate::tests) fn unchanged() -> Self {
        Self::Unchanged
    }
}

pub(super) fn put_tuple(message: &mut Vec<u8>, values: &[TupleValue<'_>]) {
    put_u16(message, values.len() as u16);
    for value in values {
        match value {
            TupleValue::Null => message.push(b'n'),
            TupleValue::Unchanged => message.push(b'u'),
            TupleValue::Text(value) => {
                message.push(b't');
                put_sized(message, value.as_bytes());
            }
            TupleValue::Binary(value) => {
                message.push(b'b');
                put_sized(message, value);
            }
        }
    }
}
