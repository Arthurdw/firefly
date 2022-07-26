use crate::ascii_optimisation::{compress, compress_slice, decompress, decompress_slice};

#[test]
fn test_compress() {
    assert_eq!(compress(0b0011_1110, 0b0111_1111, 2).unwrap(), 0b1111_1011);
    assert_eq!(
        compress(255, 255, 1),
        Err("The current and followup bytes must be ascii.".to_string())
    );
}

#[test]
fn test_decompress() {
    assert_eq!(decompress(0b1111_1100, 0b1111_1100, 2), 0b0011_1111);
}

#[test]
fn test_compress_slice() {
    let uncompressed: Vec<u8> = vec![0b0111_1111, 0b0110_0001, 0b0111_1111];
    let expected: Vec<u8> = vec![0b1111_1111, 0b1000_0111, 0b1111_1000];

    assert_eq!(compress_slice(&uncompressed).unwrap(), expected);
}

#[test]
fn test_decompress_slice() {
    let compressed: Vec<u8> = vec![0b1111_1111, 0b1000_0111, 0b1111_1000];
    let expected: Vec<u8> = vec![0b0111_1111, 0b0110_0001, 0b0111_1111];

    assert_eq!(decompress_slice(&compressed), expected);
}
