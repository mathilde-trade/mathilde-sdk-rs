use crate::core::error::SdkError;
use prost::Message;

pub(crate) const OUTPUT_ROW_COMPUTED_FIRST_FIELD_NUMBER: u32 = 21;

pub(crate) fn extract_numeric_computed_fields_from_message<M: Message>(
    message: &M,
    field_names: &[&str],
    first_field_number: u32,
    context: &'static str,
) -> Result<serde_json::Map<String, serde_json::Value>, SdkError> {
    let bytes = message.encode_to_vec();
    let mut fields = serde_json::Map::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        let key = decode_varint(&bytes, &mut cursor, context)?;
        let field_number = (key >> 3) as u32;
        let wire_type = (key & 0x07) as u8;

        let index = field_number.checked_sub(first_field_number);
        if let Some(index) = index.filter(|index| (*index as usize) < field_names.len()) {
            if wire_type != 1 {
                return Err(SdkError::contract_drift(format!(
                    "{context} computed field `{}` used unsupported wire type {wire_type}",
                    field_names[index as usize]
                )));
            }

            if cursor + 8 > bytes.len() {
                return Err(SdkError::contract_drift(format!(
                    "{context} truncated fixed64 payload for computed field `{}`",
                    field_names[index as usize]
                )));
            }

            let bits = u64::from_le_bytes(
                bytes[cursor..cursor + 8]
                    .try_into()
                    .expect("fixed64 slice length"),
            );
            cursor += 8;

            let value = f64::from_bits(bits);
            let number = serde_json::Number::from_f64(value).ok_or_else(|| {
                SdkError::contract_drift(format!(
                    "{context} computed field `{}` was non-finite",
                    field_names[index as usize]
                ))
            })?;
            fields.insert(
                field_names[index as usize].to_string(),
                serde_json::Value::Number(number),
            );
            continue;
        }

        skip_wire_value(&bytes, &mut cursor, wire_type, context)?;
    }

    Ok(fields)
}

fn decode_varint(bytes: &[u8], cursor: &mut usize, context: &'static str) -> Result<u64, SdkError> {
    let mut value = 0u64;
    let mut shift = 0u32;

    while *cursor < bytes.len() && shift < 64 {
        let byte = bytes[*cursor];
        *cursor += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
    }

    Err(SdkError::contract_drift(format!(
        "{context} contained an invalid protobuf varint"
    )))
}

fn skip_wire_value(
    bytes: &[u8],
    cursor: &mut usize,
    wire_type: u8,
    context: &'static str,
) -> Result<(), SdkError> {
    match wire_type {
        0 => {
            let _ = decode_varint(bytes, cursor, context)?;
            Ok(())
        }
        1 => {
            if *cursor + 8 > bytes.len() {
                return Err(SdkError::contract_drift(format!(
                    "{context} contained a truncated fixed64 field"
                )));
            }
            *cursor += 8;
            Ok(())
        }
        2 => {
            let len = decode_varint(bytes, cursor, context)? as usize;
            if *cursor + len > bytes.len() {
                return Err(SdkError::contract_drift(format!(
                    "{context} contained a truncated length-delimited field"
                )));
            }
            *cursor += len;
            Ok(())
        }
        5 => {
            if *cursor + 4 > bytes.len() {
                return Err(SdkError::contract_drift(format!(
                    "{context} contained a truncated fixed32 field"
                )));
            }
            *cursor += 4;
            Ok(())
        }
        other => Err(SdkError::contract_drift(format!(
            "{context} used unsupported protobuf wire type {other}"
        ))),
    }
}
