use std::io::Write;

use flate2::{write::DeflateEncoder, Compression};

pub const PUBLIC_SERVER: &str = "https://www.plantuml.com/plantuml";

pub fn svg_url(source: &str) -> String {
    format!("{PUBLIC_SERVER}/svg/{}", encode(source))
}

fn encode(source: &str) -> String {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(source.as_bytes())
        .expect("writing to an in-memory buffer cannot fail");
    let compressed = encoder
        .finish()
        .expect("finishing an in-memory buffer cannot fail");

    let mut encoded = String::with_capacity((compressed.len() + 2) / 3 * 4);
    for chunk in compressed.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or_default();
        let third = chunk.get(2).copied().unwrap_or_default();
        encoded.push_str(&encode_triplet(first, second, third));
    }
    encoded
}

fn encode_triplet(first: u8, second: u8, third: u8) -> String {
    let c1 = first >> 2;
    let c2 = ((first & 0x3) << 4) | (second >> 4);
    let c3 = ((second & 0xF) << 2) | (third >> 6);
    let c4 = third & 0x3F;

    [c1, c2, c3, c4].into_iter().map(encode_six_bits).collect()
}

fn encode_six_bits(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=35 => char::from(b'A' + value - 10),
        36..=61 => char::from(b'a' + value - 36),
        62 => '-',
        63 => '_',
        _ => unreachable!("six-bit values cannot exceed 63"),
    }
}

#[cfg(test)]
mod tests {
    use super::{encode_six_bits, svg_url, PUBLIC_SERVER};

    #[test]
    fn six_bit_boundaries_then_use_plantuml_alphabet() {
        // Arrange
        let values = [0, 9, 10, 35, 36, 61, 62, 63];

        // Act
        let encoded: String = values.into_iter().map(encode_six_bits).collect();

        // Assert
        assert_eq!(encoded, "09AZaz-_");
    }

    #[test]
    fn plantuml_source_then_generates_public_svg_url() {
        // Arrange
        let source = "@startuml\nAlice -> Bob: hello\n@enduml";

        // Act
        let url = svg_url(source);

        // Assert
        assert!(url.starts_with(&format!("{PUBLIC_SERVER}/svg/")));
        assert!(url
            .trim_start_matches(&format!("{PUBLIC_SERVER}/svg/"))
            .chars()
            .all(|character| character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'));
    }
}
