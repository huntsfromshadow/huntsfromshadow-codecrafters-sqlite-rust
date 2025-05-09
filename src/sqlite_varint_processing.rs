macro_rules! format_binary_spaced {
    ($num:expr) => {
        {
            let num_value = $num;
            let bits = std::mem::size_of_val(&num_value) * 8;
            let binary_string = format!("{:0>1$b}", num_value, bits);
            let mut formatted_binary = String::new();
            for (i, bit) in binary_string.chars().enumerate() {
                formatted_binary.push(bit);
                if (i + 1) % 4 == 0 && i != binary_string.len() - 1 {
                    formatted_binary.push(' ');
                }
            }
            formatted_binary
        }
    };
}

#[derive(Debug, PartialEq, Eq)]
pub enum VarintError {
    IncompleteVarint
}

pub fn process_sqlite_varint(data: Vec<u8>) -> Result<(i64, usize), VarintError> {
    let mut result: i64 = 0;
    let mut bytes_read: usize = 0;

    if data.is_empty() {
        return Err(VarintError::IncompleteVarint);
    }

    for i in 0..9 {
        // A varint is at most 9 bytes long
        if i >= data.len() {
            // We need more bytes, but the input slice is exhausted.
            // This happens if a previous byte had its MSB set, indicating continuation.
            return Err(VarintError::IncompleteVarint);
        }

        let byte = data[i];
        bytes_read += 1;

        if i < 8 {
            // For the first 8 bytes, the lower 7 bits are payload.
            let payload = byte & 0x7F;
            result = (result << 7) | (payload as i64);

            if (byte & 0x80) == 0 {
                // MSB is 0, so this is the last byte of the varint.
                return Ok((result as i64, bytes_read));
            }
            // MSB is 1, continue to the next byte.
        } else {
            // This is the 9th byte. It uses all 7 bits for payload,
            // as the MSB of the 9th byte for a valid i64 range will be 0.
            // note if msb is 1 it's bad data
            if(byte >= 0x80) {
                return Err(VarintError::IncompleteVarint)
            }
            
            let payload = byte & 0x7F;
            result = (result << 7) | (payload as i64);
            return Ok((result as i64, bytes_read));
        }
    }

    // This part should ideally not be reached if the varint is correctly formed
    // and fits within 9 bytes, as the loop's conditions and returns cover all cases.
    // If the 8th byte had MSB set, the 9th byte path is taken and returns.
    // If any byte < 8th had MSB cleared, it would have returned earlier.
    // For safety, though practically unreachable with current logic:
    Err(VarintError::IncompleteVarint) // Or a more specific error if possible
}


#[cfg(test)]
mod tests {
    use itertools::assert_equal;
    use crate::sqlite_varint_processing::{process_sqlite_varint, VarintError};

    // Single Bytes self-contained list
    #[test]
    fn test_single_byte_zero() {
        assert_eq!(process_sqlite_varint(vec![0u8]), Ok((0,1)));
    }

    #[test]
    fn test_small_positive_integer() {
        assert_eq!(process_sqlite_varint(vec![0x5a]), Ok((90,1)));
    }

    #[test]
    fn test_max_single_byte_value() {
        assert_eq!(process_sqlite_varint(vec![0x7f]), Ok((127,1)));
    }

    // Multi Bytes but still self-contained list
    #[test]
    fn test_small_two_byte_value() {
        assert_eq!(process_sqlite_varint(vec![0x81,0x00]), Ok((128,2)));
    }

    #[test]
    fn mid_range_two_byte() {
        assert_eq!(process_sqlite_varint(vec![0x8a,0x3b]), Ok((1339,2)));
    }

    #[test]
    fn maximum_two_byte_value() {
        assert_eq!(process_sqlite_varint(vec![0xff,0x7f]), Ok((16383,2)));
    }

    #[test]
    fn small_three_byte_value() {
        assert_eq!(process_sqlite_varint(vec![0x81,0x80,0x00]), Ok((16384,3)));
    }

    // Edge cases still self-contained
    #[test]
    fn maximum_valid_varint() {
        let res = process_sqlite_varint(
                vec![0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0x7F]);
        let r = res.unwrap();
        assert_eq!( r, (i64::MAX, 9));
    }

    #[test]
    fn single_byte_zero() {
        let res = process_sqlite_varint(vec![0x00]);
        assert_eq!(res, Ok((0, 1)));
    }

    #[test]
    fn single_byte_positive() {
        let res = process_sqlite_varint(vec![0x5A]);
        assert_eq!(res, Ok((90, 1)));
    }

    #[test]
    fn single_byte_max() {
        let res = process_sqlite_varint(vec![0x7F]);
        assert_eq!(res, Ok((127, 1)));
    }

    #[test]
    fn two_bytes_small() {
        let res = process_sqlite_varint(vec![0x81, 0x00]);
        assert_eq!(res, Ok((128, 2)));
    }

    #[test]
    fn two_bytes_mid() {
        let res = process_sqlite_varint(vec![0x8A, 0x3B]);
        assert_eq!(res, Ok((1339, 2)));
    }

    #[test]
    fn two_bytes_max() {
        let res = process_sqlite_varint(vec![0xFF, 0x7F]);
        assert_eq!(res, Ok((16383, 2)));
    }

    #[test]
    fn three_bytes_small() {
        let res = process_sqlite_varint(vec![0x81, 0x80, 0x00]);
        assert_eq!(res, Ok((16384, 3)));
    }

    #[test]
    fn empty_input() {
        let res = process_sqlite_varint(vec![]);
        assert_eq!(res, Err(VarintError::IncompleteVarint));
    }

    #[test]
    fn incomplete_two_byte() {
        let res = process_sqlite_varint(vec![0x81]);
        assert_eq!(res, Err(VarintError::IncompleteVarint));
    }

    #[test]
    fn incomplete_multi_byte() {
        let res = process_sqlite_varint(vec![0x81, 0x80]);
        assert_eq!(res, Err(VarintError::IncompleteVarint));
    }

    #[test]
    fn eight_bytes_with_continuation() {
        let res = process_sqlite_varint(vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(res, Err(VarintError::IncompleteVarint));
    }

    #[test]
    fn nine_bytes_continuation_on_last() {
        let res = process_sqlite_varint(
            vec![0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81]);
        // While this is technically 9 bytes, the last byte has its MSB set,
        // indicating more data which isn't there.
        assert_eq!(res, Err(VarintError::IncompleteVarint));
    }

    #[test]
    fn single_byte_followed_by_extra() {
        let data = vec![0x5A, 0xBB, 0xCC];
        let res = process_sqlite_varint(data);
        assert_eq!(res, Ok((90, 1)));
    }

    #[test]
    fn two_bytes_followed_by_extra() {
        let data = vec![0x8A, 0x3B, 0xCC, 0xDD];
        let res = process_sqlite_varint(data);
        assert_eq!(res, Ok((1339, 2)));
    }

    #[test]
    fn four_bytes_followed_by_extra() {
        let data = vec![0x81, 0x82, 0x83, 0x04, 0xEE, 0xFF];
        let res = process_sqlite_varint(data);
        assert_eq!(res, Ok(((((1 << 7 | 2) << 7 | 3) << 7 | 4) as i64, 4)));
    }
}