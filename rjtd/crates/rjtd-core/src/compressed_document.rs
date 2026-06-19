use crate::lha::decompress_lh5_member;
use crate::{Error, Result};

pub const JUST_COMPRESSED_DOCUMENT_MAGIC: &[u8] = b"\x26\0JustCompressedDocument";
const LH5_METHOD: &[u8; 5] = b"-lh5-";

pub fn is_just_compressed_document(data: &[u8]) -> bool {
    data.starts_with(JUST_COMPRESSED_DOCUMENT_MAGIC)
}

pub fn decompress_just_compressed_document(data: &[u8]) -> Result<Vec<u8>> {
    if !is_just_compressed_document(data) {
        return Err(Error::InvalidData(
            "missing JustCompressedDocument marker".into(),
        ));
    }

    let method_offset = data
        .windows(LH5_METHOD.len())
        .position(|window| window == LH5_METHOD)
        .ok_or_else(|| Error::InvalidData("missing -lh5- member marker".into()))?;
    let member_start = method_offset
        .checked_sub(2)
        .ok_or_else(|| Error::InvalidData("invalid -lh5- member marker offset".into()))?;
    let member = decompress_lh5_member(&data[member_start..])?;
    Ok(member.bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::is_just_compressed_document;

    #[test]
    fn detects_just_compressed_document_payload() {
        assert!(is_just_compressed_document(
            b"\x26\0JustCompressedDocument\0payload"
        ));
        assert!(!is_just_compressed_document(b"DocumentText"));
    }
}
