pub struct ByteCharIter<I> {
    iter: I,
}

pub fn expected_byte_length_from_starting(starting_byte: u8) -> Option<u8> {
    if starting_byte & 0b1000_0000 == 0 {
        Some(1)
    } else if starting_byte & 0b1100_0000 == 0b1000_0000 {
        // Found "following byte" 2 byte starting code
        None
    } else if starting_byte & 0b1110_0000 == 0b1100_0000 {
        Some(2)
    } else if starting_byte & 0b1111_0000 == 0b1110_0000 {
        Some(3)
    } else if starting_byte & 0b1111_1000 == 0b1111_0000 {
        Some(4)
    } else {
        // Found byte starting with 5 or more leading 1's, invalid utf8 byte
        None
    }
}

impl<I> ByteCharIter<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'a, I> Iterator for ByteCharIter<I>
where
    I: Iterator<Item = &'a u8>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut byte_buf = vec![];

        let next_byte = self.iter.next()?;

        let Some(char_byte_length) = expected_byte_length_from_starting(*next_byte) else {
            panic!(
                "Invalid utf8 encoding. Found invalid first byte for utf8 encoding: {:x}",
                next_byte
            );
        };

        byte_buf.push(*next_byte);
        for _ in 0..char_byte_length-1 {
            let Some(following_byte) = self.iter.next() else {
                panic!("Invalid utf8 encoding. Reached end of byte stream with partial utf8 char parsed");
            };
            byte_buf.push(*following_byte);
        }

        let decoded_string = std::str::from_utf8(byte_buf.as_slice())
            .expect("Failed to decide utf8 character from byte iterator");

        let mut chars = decoded_string.chars();
        let Some(char) = chars.next() else {
            panic!("Failed to convert byte iterator to valid utf8 character.");
        };
        if chars.next().is_some() {
            panic!("Found multiple characters while attempting to parse a single character from utf8 byte iterator.");
        }

        Some(char)
    }
}
