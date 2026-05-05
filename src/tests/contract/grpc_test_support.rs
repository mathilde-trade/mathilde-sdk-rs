use flate2::read::GzDecoder;
use prost::Message;
use std::io::{Cursor, Read};

pub fn decode_test_grpc_message<M: Message + Default>(body: &[u8]) -> M {
    assert!(body.len() >= 5, "grpc frame too short");
    let compressed = body[0];
    let len = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
    assert_eq!(body.len(), 5 + len, "grpc frame length mismatch");
    let payload = &body[5..];
    let decoded = match compressed {
        0 => payload.to_vec(),
        1 => decode_compressed_payload(payload),
        other => panic!("unsupported grpc compression flag in test: {other}"),
    };
    M::decode(decoded.as_slice()).expect("decode grpc message")
}

fn decode_compressed_payload(payload: &[u8]) -> Vec<u8> {
    if let Ok(decoded) = zstd::stream::decode_all(Cursor::new(payload)) {
        return decoded;
    }

    let mut decoder = GzDecoder::new(payload);
    let mut decoded = Vec::new();
    if decoder.read_to_end(&mut decoded).is_ok() {
        return decoded;
    }

    panic!("unsupported grpc compressed payload encoding in test");
}
