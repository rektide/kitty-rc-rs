//! A valid 1×1 RGBA PNG fixture generated without external codecs.
//!
//! The upload conformance journey needs an honest, independently valid PNG.
//! zlib stored (uncompressed) deflate blocks are fully valid, so the IDAT
//! stream is assembled by hand with a computed Adler-32 checksum and every
//! chunk carries a computed CRC-32; unit tests decode the result back.

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

/// The fixture pixel: RGB (64, 128, 255), fully opaque.
const PIXEL: [u8; 4] = [64, 128, 255, 255];

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (index, entry) in table.iter_mut().enumerate() {
        let mut value = index as u32;
        for _ in 0..8 {
            value = if value & 1 != 0 {
                0xEDB8_8320 ^ (value >> 1)
            } else {
                value >> 1
            };
        }
        *entry = value;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for byte in data {
        crc = table[((crc ^ *byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data {
        a = (a + *byte as u32) % 0xFFF1;
        b = (b + a) % 0xFFF1;
    }
    (b << 16) | a
}

/// A zlib stream carrying `data` as a single stored deflate block.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 11);
    out.extend_from_slice(&[0x78, 0x01]); // deflate 32K window, stored blocks
    out.push(0x01); // final block, stored
    let length = u16::try_from(data.len()).expect("stored block below 64 KiB");
    out.extend_from_slice(&length.to_le_bytes());
    out.extend_from_slice(&(!length).to_le_bytes());
    out.extend_from_slice(data);
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn push_chunk(kind: &[u8; 4], data: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// The complete independent PNG fixture bytes.
pub fn one_pixel_rgba_png() -> Vec<u8> {
    let mut scanline = Vec::with_capacity(5);
    scanline.push(0); // filter: none
    scanline.extend_from_slice(&PIXEL);
    let mut png = Vec::new();
    png.extend_from_slice(PNG_SIGNATURE);
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&1u32.to_be_bytes()); // width
    ihdr.extend_from_slice(&1u32.to_be_bytes()); // height
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA
    push_chunk(b"IHDR", &ihdr, &mut png);
    push_chunk(b"IDAT", &zlib_stored(&scanline), &mut png);
    push_chunk(b"IEND", &[], &mut png);
    png
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk_at(png: &[u8], offset: &mut usize) -> ([u8; 4], Vec<u8>) {
        let length = u32::from_be_bytes(png[*offset..*offset + 4].try_into().unwrap()) as usize;
        let kind: [u8; 4] = png[*offset + 4..*offset + 8].try_into().unwrap();
        let data = png[*offset + 8..*offset + 8 + length].to_vec();
        let crc = u32::from_be_bytes(
            png[*offset + 8 + length..*offset + 12 + length]
                .try_into()
                .unwrap(),
        );
        let mut covered = kind.to_vec();
        covered.extend_from_slice(&data);
        assert_eq!(
            crc,
            crc32(&covered),
            "chunk {} CRC mismatch",
            String::from_utf8_lossy(&kind)
        );
        *offset += 12 + length;
        (kind, data)
    }

    /// Decode a stored-block zlib stream back to its bytes.
    fn inflate_stored(zlib: &[u8]) -> Vec<u8> {
        assert_eq!(&zlib[..2], &[0x78, 0x01]);
        let length = u16::from_le_bytes(zlib[3..5].try_into().unwrap()) as usize;
        let body = &zlib[7..7 + length];
        assert_eq!(
            adler32(body),
            u32::from_be_bytes(zlib[7 + length..11 + length].try_into().unwrap())
        );
        body.to_vec()
    }

    #[test]
    fn png_has_valid_signature_chunks_and_pixel_scanline() {
        let png = one_pixel_rgba_png();
        assert_eq!(&png[..8], PNG_SIGNATURE.as_slice());
        let mut offset = 8;
        let (ihdr_kind, ihdr) = chunk_at(&png, &mut offset);
        assert_eq!(&ihdr_kind, b"IHDR");
        assert_eq!(&ihdr[..8], &1u32.to_be_bytes().repeat(2));
        assert_eq!(&ihdr[8..13], &[8, 6, 0, 0, 0]);
        let (idat_kind, idat) = chunk_at(&png, &mut offset);
        assert_eq!(&idat_kind, b"IDAT");
        assert_eq!(inflate_stored(&idat), [0, 64, 128, 255, 255]);
        let (iend_kind, _) = chunk_at(&png, &mut offset);
        assert_eq!(&iend_kind, b"IEND");
        assert_eq!(offset, png.len());
    }
}
