use std::ffi::CStr;

use pgrx::{pg_sys, varlena};

use crate::{NativeLogicalColumnStatus, NativeLogicalFrameBuilder};

pub(super) unsafe fn append_tuple(
    frame: &mut NativeLogicalFrameBuilder,
    relation: pg_sys::Relation,
    tuple: pg_sys::HeapTuple,
) {
    super::change::or_error(frame.append_tuple_presence(!tuple.is_null()));
    if tuple.is_null() {
        return;
    }
    if relation.is_null() || unsafe { (*relation).rd_att.is_null() } {
        pgrx::error!("Trellara output plugin received a relation without a tuple descriptor");
    }
    let tuple_descriptor = unsafe { (*relation).rd_att };
    let attribute_count = unsafe { (*tuple_descriptor).natts };
    let attribute_count_u16 = u16::try_from(attribute_count)
        .unwrap_or_else(|_| pgrx::error!("relation has invalid attribute count {attribute_count}"));
    super::change::or_error(frame.append_tuple_start(attribute_count_u16));
    for index in 0..attribute_count {
        let attribute = unsafe { &*attribute_at(tuple_descriptor, index as usize) };
        let type_oid = attribute.atttypid.to_u32();
        if attribute.attisdropped {
            super::change::or_error(frame.append_column(
                type_oid,
                NativeLogicalColumnStatus::Dropped,
                &[],
            ));
            continue;
        }
        let attribute_number = index + 1;
        if unsafe { pg_sys::heap_attisnull(tuple, attribute_number, tuple_descriptor) } {
            super::change::or_error(frame.append_column(
                type_oid,
                NativeLogicalColumnStatus::Null,
                &[],
            ));
            continue;
        }
        let datum = unsafe { pg_sys::nocachegetattr(tuple, attribute_number, tuple_descriptor) };
        if unchanged_toast_on_disk(attribute, datum) {
            super::change::or_error(frame.append_column(
                type_oid,
                NativeLogicalColumnStatus::UnchangedToast,
                &[],
            ));
            continue;
        }
        append_value(frame, attribute, type_oid, datum);
    }
}

fn append_value(
    frame: &mut NativeLogicalFrameBuilder,
    attribute: &pg_sys::FormData_pg_attribute,
    type_oid: u32,
    datum: pg_sys::Datum,
) {
    let mut output_function = pg_sys::Oid::INVALID;
    let mut is_varlena = false;
    unsafe { pg_sys::getTypeOutputInfo(attribute.atttypid, &mut output_function, &mut is_varlena) };
    let output = unsafe { pg_sys::OidOutputFunctionCall(output_function, datum) };
    if output.is_null() {
        pgrx::error!("PostgreSQL returned a null textual value for relation attribute");
    }
    let bytes = unsafe { CStr::from_ptr(output) }.to_bytes();
    let result = frame.append_column(type_oid, NativeLogicalColumnStatus::Value, bytes);
    unsafe { pg_sys::pfree(output.cast()) };
    super::change::or_error(result);
}

fn unchanged_toast_on_disk(
    attribute: &pg_sys::FormData_pg_attribute,
    datum: pg_sys::Datum,
) -> bool {
    if attribute.attbyval || attribute.attlen != -1 || datum.is_null() {
        return false;
    }
    let value = datum.cast_mut_ptr::<pg_sys::varlena>();
    unsafe {
        varlena::varatt_is_1b_e(value)
            && varlena::vartag_1b_e(value) as pg_sys::vartag_external::Type
                == pg_sys::vartag_external::VARTAG_ONDISK
    }
}

#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17"))]
unsafe fn attribute_at(
    tuple_descriptor: pg_sys::TupleDesc,
    index: usize,
) -> *const pg_sys::FormData_pg_attribute {
    unsafe { (*tuple_descriptor).attrs.as_ptr().add(index) }
}

#[cfg(feature = "pg18")]
unsafe fn attribute_at(
    tuple_descriptor: pg_sys::TupleDesc,
    index: usize,
) -> *const pg_sys::FormData_pg_attribute {
    let offset = std::mem::offset_of!(pg_sys::TupleDescData, compact_attrs)
        + usize::try_from(unsafe { (*tuple_descriptor).natts })
            .expect("nonnegative attribute count")
            * std::mem::size_of::<pg_sys::CompactAttribute>();
    unsafe {
        tuple_descriptor
            .cast::<u8>()
            .add(offset)
            .cast::<pg_sys::FormData_pg_attribute>()
            .add(index)
    }
}
