use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endianness {
    Big,
    Little,
}

// Little-endian hexadecimal string
#[derive(Debug, PartialEq, Eq)]
pub struct HexString {
    hex_bytes: Vec<HexByte>,
    endianness: Endianness,
}

impl HexString {
    // Creates a `HexString` with the appropriate `Endianness`
    pub fn from_hex_bytes(
        mut hex_bytes: Vec<HexByte>,
        source_endianness: Endianness,
        target_endianness: Endianness,
    ) -> Self {
        if source_endianness != target_endianness {
            hex_bytes.reverse();
        }

        Self {
            hex_bytes,
            endianness: target_endianness,
        }
    }

    pub fn from_hex_str(
        s: &str,
        source_endianness: Endianness,
        target_endianness: Endianness,
    ) -> Result<Self, String> {
        let mut string_of_hex_chars = String::new();

        // Prefix '0' if odd numebr of characters to pad into bytes
        if s.len() % 2 != 0 {
            string_of_hex_chars.push('0');
        }

        string_of_hex_chars.push_str(s);

        let mut hex_bytes = Vec::with_capacity((string_of_hex_chars.len()) / 2);

        for i in 0..hex_bytes.capacity() {
            // Push indices 0..=1, then 2..=3, etc. in blocks of up to 2 nibbles, making up a byte (or padded to it when making a `HexByte`)
            let start_index = i * 2;
            let end_index = i * 2 + 1;
            let hex_byte = HexByte::from_hex_str(&string_of_hex_chars[start_index..=end_index])?;
            hex_bytes.push(hex_byte);
        }

        Ok(Self::from_hex_bytes(
            hex_bytes,
            source_endianness,
            target_endianness,
        ))
    }

    // Converts each character of `s` into a `HexByte`, constructing a `HexString`
    pub fn from_str(s: &str, source_endianness: Endianness, target_endianness: Endianness) -> Self {
        let mut hex_bytes = Vec::with_capacity(s.len());

        for c in s.chars() {
            hex_bytes.push(HexByte::from(c));
        }

        Self::from_hex_bytes(hex_bytes, source_endianness, target_endianness)
    }

    // Changes the endianness and associated contents
    pub fn as_endianness(self, endianness: Endianness) -> Self {
        let mut hex_bytes = self.hex_bytes.clone();

        if self.endianness != endianness {
            hex_bytes.reverse();
        }

        Self {
            hex_bytes,
            endianness,
        }
    }

    // Returns a vector of the indices at which `needle` is found in `self.hex_bytes`
    pub fn get_offsets(&self, mut needle: Self) -> Vec<usize> {
        let mut matches = Vec::new();

        // Ensure that the needle is not empty and not larger than the search content
        if needle.hex_bytes.len() == 0 || self.hex_bytes.len() < needle.hex_bytes.len() {
            return matches;
        }

        if needle.endianness != self.endianness {
            needle = needle.as_endianness(self.endianness);
        }

        // Use a `VecDeque` as a FIFO which contains content equivalent to the needle size for comparison
        let mut current_hex_bytes = VecDeque::with_capacity(needle.hex_bytes.len());

        for i in 0..self.hex_bytes.len() {
            // Not enough remaining content to update `current_hex_bytes` without going OOB
            if i + needle.hex_bytes.len() > self.hex_bytes.len() {
                break;
            }

            // Update the FIFO
            if i == 0 {
                // Setup the FIFO on the first iteration
                for i in 0..needle.hex_bytes.len() {
                    // `unwrap` as we have previously checked that `contents` has at least the same length as `needle`
                    current_hex_bytes.push_back(self.hex_bytes.get(i).unwrap());
                }
            } else {
                // Update the FIFO on each iteration after the first, `unwrap` because we've already protected against OOB
                current_hex_bytes.pop_front();
                current_hex_bytes
                    .push_back(self.hex_bytes.get(i + needle.hex_bytes.len() - 1).unwrap());
            }

            let mut matched = true;

            // Check whether `current_hex_bytes` and `needle` match
            for (&a, b) in current_hex_bytes.iter().zip(needle.hex_bytes.iter()) {
                if a != b {
                    matched = false;
                    break;
                }
            }

            if matched {
                matches.push(i);
            }
        }

        return matches;
    }

    pub fn as_hex_string(self, endianness: Endianness) -> String {
        let mut result = String::with_capacity(self.hex_bytes.len() * 2);

        let hex_string = match endianness == self.endianness {
            true => self,
            false => self.as_endianness(endianness),
        };

        hex_string
            .hex_bytes
            .into_iter()
            .for_each(|hex_byte| result.push_str(&hex_byte.contents));

        result
    }

    pub fn as_usize(self) -> Option<usize> {
        match usize::from_str_radix(&self.as_hex_string(Endianness::Big), 16) {
            Ok(i) => Some(i),
            Err(_) => None,
        }
    }
}

// Two lower case hexadecimal characters, making up one byte
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexByte {
    contents: String,
}

impl HexByte {
    // Creates a new `HexByte`, consisting of lower case characters, ensuring it's a valid hexadecimal value
    pub fn from_hex_str(hex_byte: &str) -> Result<Self, String> {
        if hex_byte.len() > 2 {
            return Err(format!(
                "HexByte contents must be of most length 2, provided: {}",
                hex_byte.len()
            ));
        }

        let hex_byte = format!("{:0>2}", hex_byte.to_owned());

        // Validate is 0-9, a-f, or A-F
        for c in hex_byte.chars() {
            if !((c as u8 >= 48 && c as u8 <= 57)
                || (c as u8 >= 65 && c as u8 <= 70)
                || (c as u8 >= 97 && c as u8 <= 102))
            {
                return Err(format!(
                    "cannot instantiate a HexByte with a non-hexadecimal character: {}",
                    c
                ));
            }
        }

        Ok(Self {
            contents: hex_byte.to_lowercase(),
        })
    }
}

impl ToString for HexByte {
    fn to_string(&self) -> String {
        self.contents.clone()
    }
}

impl From<char> for HexByte {
    fn from(c: char) -> Self {
        // `unwrap` as we are passing an arugment which should never fail
        Self::from_hex_str(&format!("{:x}", c as u8)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_hex_bytes() -> Vec<HexByte> {
        vec![
            HexByte::from_hex_str("a0").unwrap(),
            HexByte::from_hex_str("00").unwrap(),
            HexByte::from_hex_str("ff").unwrap(),
            HexByte::from_hex_str("90").unwrap(),
            HexByte::from_hex_str("7b").unwrap(),
        ]
    }

    #[test]
    fn hex_string_from_hex_bytes() {
        let hex_bytes = setup_hex_bytes();

        let correct_hex_bytes = hex_bytes.clone();

        let hex_string =
            HexString::from_hex_bytes(hex_bytes, Endianness::Little, Endianness::Little);

        assert_eq!(correct_hex_bytes, hex_string.hex_bytes);
    }

    #[test]
    fn hex_string_from_hex_bytes_changing_endianness() {
        let hex_bytes = setup_hex_bytes();

        let mut correct_hex_bytes = hex_bytes.clone();
        correct_hex_bytes.reverse();

        let hex_string = HexString::from_hex_bytes(hex_bytes, Endianness::Big, Endianness::Little);

        assert_eq!(correct_hex_bytes, hex_string.hex_bytes);
    }

    #[test]
    fn hex_string_from_hex_str() {
        let hex_string =
            HexString::from_hex_str("a000ff907b", Endianness::Little, Endianness::Little).unwrap();

        assert_eq!(setup_hex_bytes(), hex_string.hex_bytes);
    }

    #[test]
    fn hex_string_from_hex_str_changing_endianness() {
        let hex_string =
            HexString::from_hex_str("7b90ff00a0", Endianness::Little, Endianness::Big).unwrap();

        assert_eq!(setup_hex_bytes(), hex_string.hex_bytes);
    }

    #[test]
    fn hex_string_from_hex_str_nibble() {
        let hex_string =
            HexString::from_hex_str("fff", Endianness::Little, Endianness::Little).unwrap();

        assert_eq!(
            HexByte::from_hex_str("0f").unwrap(),
            hex_string.hex_bytes[0]
        );
    }

    #[test]
    fn hex_string_from_invalid_hex_str() {
        let result = HexString::from_hex_str("x0", Endianness::Little, Endianness::Little);

        assert!(result.is_err());
    }

    fn setup_ascii_hex_bytes() -> Vec<HexByte> {
        vec![
            HexByte::from_hex_str("41").unwrap(), // A
            HexByte::from_hex_str("61").unwrap(), // a
            HexByte::from_hex_str("30").unwrap(), // 0
            HexByte::from_hex_str("41").unwrap(), // A
            HexByte::from_hex_str("61").unwrap(), // a
            HexByte::from_hex_str("31").unwrap(), // 1
            HexByte::from_hex_str("41").unwrap(), // A
            HexByte::from_hex_str("61").unwrap(), // a
            HexByte::from_hex_str("32").unwrap(), // 2
        ]
    }

    #[test]
    fn hex_string_from_str() {
        let hex_string = HexString::from_str("Aa0Aa1Aa2", Endianness::Little, Endianness::Little);

        assert_eq!(hex_string.hex_bytes, setup_ascii_hex_bytes());
    }

    #[test]
    fn hex_string_from_str_chainging_endianness() {
        let hex_string = HexString::from_str("Aa0Aa1Aa2", Endianness::Big, Endianness::Little);

        let mut correct_hex_bytes = setup_ascii_hex_bytes();

        correct_hex_bytes.reverse();

        assert_eq!(hex_string.hex_bytes, correct_hex_bytes);
    }

    #[test]
    fn hex_string_as_literal_hex_string() {
        let hex_string =
            HexString::from_hex_str("00112233", Endianness::Big, Endianness::Big).unwrap();

        assert_eq!(hex_string.as_hex_string(Endianness::Big), "00112233");
    }

    #[test]
    fn hex_string_as_literal_hex_string_with_opposite_endianness() {
        let hex_string =
            HexString::from_hex_str("00112233", Endianness::Big, Endianness::Big).unwrap();

        assert_eq!(hex_string.as_hex_string(Endianness::Little), "33221100");
    }

    #[test]
    fn hex_string_into_usize() {
        let hex_string =
            HexString::from_hex_str("00112233", Endianness::Big, Endianness::Big).unwrap();

        assert_eq!(hex_string.as_usize(), Some(1122867));
    }

    #[test]
    fn hex_string_too_large_for_usize() {
        let hex_string =
            HexString::from_hex_str("fffffffffffffffff", Endianness::Big, Endianness::Big).unwrap();

        assert_eq!(hex_string.as_usize(), None);
    }

    fn create_le_hex_string() -> HexString {
        HexString::from_hex_str("Aa0Aa1Aa2", Endianness::Little, Endianness::Little).unwrap()
    }

    #[test]
    fn set_same_endianness() {
        let mut hex_string = create_le_hex_string();

        hex_string = hex_string.as_endianness(Endianness::Little);

        assert_eq!(hex_string.endianness, Endianness::Little);
        assert_eq!(hex_string.hex_bytes, create_le_hex_string().hex_bytes);
    }

    #[test]
    fn set_different_endianness() {
        let mut hex_string = create_le_hex_string();

        hex_string = hex_string.as_endianness(Endianness::Big);

        let mut correct_hex_bytes = create_le_hex_string().hex_bytes;
        correct_hex_bytes.reverse();

        assert_eq!(hex_string.endianness, Endianness::Big);
        assert_eq!(hex_string.hex_bytes, correct_hex_bytes);
    }

    #[test]
    fn get_single_offset() {
        let haystack =
            HexString::from_hex_str("0011223344", Endianness::Little, Endianness::Little).unwrap();

        let needle =
            HexString::from_hex_str("2233", Endianness::Little, Endianness::Little).unwrap();

        let offsets = haystack.get_offsets(needle);

        assert_eq!(offsets.len(), 1);
        assert_eq!(*offsets.get(0).unwrap(), 2);
    }

    #[test]
    fn get_single_offset_with_swapped_endian() {
        let haystack =
            HexString::from_hex_str("0011223344", Endianness::Little, Endianness::Little).unwrap();

        let needle = HexString::from_hex_str("3322", Endianness::Big, Endianness::Little).unwrap();

        let offsets = haystack.get_offsets(needle);

        assert_eq!(offsets.len(), 1);
        assert_eq!(*offsets.get(0).unwrap(), 2);
    }

    #[test]
    fn get_multiple_offsets() {
        let haystack = HexString::from_hex_str(
            "00112233440011223344",
            Endianness::Little,
            Endianness::Little,
        )
        .unwrap();

        let needle =
            HexString::from_hex_str("2233", Endianness::Little, Endianness::Little).unwrap();

        let offsets = haystack.get_offsets(needle);

        assert_eq!(offsets.len(), 2);
        assert_eq!(*offsets.get(0).unwrap(), 2);
        assert_eq!(*offsets.get(1).unwrap(), 7);
    }

    #[test]
    fn get_no_matching_offsets() {
        let haystack =
            HexString::from_hex_str("0011223344", Endianness::Little, Endianness::Little).unwrap();

        let needle = HexString::from_hex_str("55", Endianness::Little, Endianness::Little).unwrap();

        let offsets = haystack.get_offsets(needle);

        assert_eq!(offsets.len(), 0);
    }

    #[test]
    fn hex_byte_from_hex_str() {
        assert_eq!(HexByte::from_hex_str("AB").unwrap().contents, "ab");
    }

    #[test]
    fn hex_byte_from_hex_str_nibble() {
        assert_eq!(HexByte::from_hex_str("1").unwrap().contents, "01");
    }

    #[test]
    fn hex_byte_from_empty_hex_str() {
        assert_eq!(HexByte::from_hex_str("").unwrap().contents, "00");
    }

    #[test]
    fn hex_byte_from_too_long_hex_str() {
        assert!(HexByte::from_hex_str("FFFF").is_err());
    }

    #[test]
    fn hex_byte_from_invalid_hex_str() {
        assert!(HexByte::from_hex_str("F0J5").is_err());
    }

    #[test]
    fn hex_byte_to_string() {
        let hex_byte = HexByte::from_hex_str("AB").unwrap();

        assert_eq!(hex_byte.to_string(), String::from("ab"));
    }

    #[test]
    fn hex_byte_from_char() {
        assert_eq!(HexByte::from('A').contents, "41");
    }

    #[test]
    fn hex_byte_from_unicode_char() {
        assert_eq!(HexByte::from('é').contents, "e9");
    }
}
