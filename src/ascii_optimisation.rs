/// Apply the ascii compression to a current byte.
/// The compression makes use of the fact that ascii only uses 7 bits.
/// So the first bit is always 0, which is a big waste of memory.
/// What we can do is to use the last bit from every byte.
/// This allows us to store 8 characters in 7 bytes instead of 8.
///
/// A visual representation:
/// ```
/// 0011 0001 -> 0110 0010
/// 0011 0010 -> 1100 1001
/// 0011 0011 -> 1001 1011
/// 0011 0100 -> 0100 ....
/// ```
///
/// # Arguments
///
/// * `current` - The current byte to compress.
/// * `followup` - The next byte to compress.
/// * `iteration` - The amount of bits (from the left) that are irrelevant.
///                 This should always be a minimum of 1.
/// # Example
///
/// ```
/// // We are able to compress 16 bytes into 14:
/// let mut compressed = Vec::with_capacity(14);
/// let mut current_byte = 1;
///
/// for i in 0..17 {
///     // Ignore the 8'th byte, as this is fully compressed into the 7'th byte.
///     if current_byte == 8 {
///         current_byte = 1;
///         continue;
///     }
///
///     compressed.push(compress(i, i + 1, current_byte).unwrap());
///     current_byte += 1;
/// }
/// ```
pub fn compress(current: u8, followup: u8, iteration: u8) -> Result<u8, String> {
    if !current.is_ascii() || !followup.is_ascii() {
        return Err("The current and followup bytes must be ascii.".to_string());
    }

    let curr_bits = 8 - iteration;
    let prev_bits = iteration;

    let bits_in_last = (current >> curr_bits) << curr_bits;
    let cleared_from_last = current ^ bits_in_last;

    let moved_current = cleared_from_last << prev_bits;

    let included_next_bits = followup >> (curr_bits - 1);

    return Ok(moved_current | included_next_bits);
}

/// Decompress the ascii compression of a byte.
/// This reverts the output of the compression function.
///
/// A visual representation:
/// ```
/// 0110 0010 -> 0011 0001
/// 1100 1001 -> 0011 0010
/// 1001 1011 -> 0011 0011
/// 0100 .... -> 0011 0100
/// ```
///
/// # Arguments
///
/// * `previous` - The previous byte, if it is the first this should be 0.
/// * `current` - The byte to decompress.
/// * `iteration` - The amount of bits (from the left) that are in the previous
///                 byte.
///
/// # Example
///
/// ```
/// // This example decompresses from the example of the compress function.
/// // So, our 14 bytes are decompressed into 16 bytes.
/// let mut decompressed = Vec::with_capacity(16);
/// let mut current_byte = 1;
/// for (idx, byte) in compressed.iter().enumerate() {
///   let comp = byte.to_owned();
///   let mut previous_byte = 0;
///
///   if idx != 0 {
///     previous_byte = compressed[idx - 1];
///   }
///
///   if current_byte == 8 {
///     current_byte = 1;
///   }
///
///   decompressed.push(decompress(previous_byte, comp, current_byte).unwrap());
///
///   // Our 7'th byte contains our 8'th byte
///   if current_byte == 7 {
///     let first_byte = (comp >> 7) << 7;
///     decompressed.push((first_byte as u8) ^ (comp as u8));
///   }
///
///   current_byte += 1;
/// }
/// ```
pub fn decompress(previous: u8, current: u8, iteration: u8) -> u8 {
    let prev_bits = iteration - 1;

    let bits_to_ignore_in_last = (previous >> prev_bits) << prev_bits;
    let bits_in_last = previous ^ bits_to_ignore_in_last;
    let bits_from_last = bits_in_last << (8 - iteration);

    let current_bits = current >> iteration;

    return bits_from_last | current_bits;
}

/// Apply the ascii compression to a slice of bytes.
/// This is a wrapper around the compress function.
///
/// # Arguments
///
/// * `slice` - The bytes to compress.
pub fn compress_slice(slice: &[u8]) -> Result<Vec<u8>, String> {
    let mut compressed = Vec::with_capacity(slice.len());
    let mut current_bit = 1;

    for i in 0..slice.len() {
        let mut next_bit = 0;

        // Get the next bit if available
        if i != slice.len() - 1 {
            next_bit = slice[i + 1];
        }

        // Our last bit is always already compressed in the previous bit.
        if current_bit == 8 {
            current_bit = 1;
            continue;
        }

        compressed.push(compress(slice[i], next_bit, current_bit)?);
        current_bit += 1;
    }

    return Ok(compressed);
}

/// Decompress a slice of bytes.
/// This is a wrapper around the decompress function.
///
/// # Arguments
///
/// * `slice` - The bytes to decompress.
pub fn decompress_slice(slice: &[u8]) -> Vec<u8> {
    let mut decompressed = Vec::with_capacity(slice.len());
    let mut current_bit = 1;

    for (idx, byte) in slice.iter().enumerate() {
        let comp = byte.to_owned();
        let mut prev_bit = 0;

        if idx != 0 {
            prev_bit = slice[idx - 1];
        }

        if current_bit == 8 {
            current_bit = 1;
        }

        decompressed.push(decompress(prev_bit, comp, current_bit));

        if current_bit == 7 {
            let first_bit = (comp >> 7) << 7;
            decompressed.push((first_bit as u8) ^ (comp as u8));
        }

        current_bit += 1;
    }

    return decompressed;
}
