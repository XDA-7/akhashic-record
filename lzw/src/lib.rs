const BIT_MASK_U8: [u8; 8] = [
    0b10000000,
    0b01000000,
    0b00100000,
    0b00010000,
    0b00001000,
    0b00000100,
    0b00000010,
    0b00000001,
];

pub struct EncodingResult {
    pub codes: Vec<Code>,
    pub bytes_processed: usize,
}

pub struct Code {
    pub value: u32,
    pub length: usize,
}

pub struct BitArray {
    pub packed_bits: Vec<u8>,
    pub length: usize,
}
impl BitArray {
    pub fn new(bits: &[u8]) -> Self {
        let mut packed_bits = Vec::new();
        let length = bits.len();
        let mut current_bit = 0;
        while current_bit != length {
            let mut byte: u8 = 0;
            let mut i = 0;
            while current_bit < length && i < 8 {
                if bits[current_bit] != 0 {
                    byte = byte | BIT_MASK_U8[i];
                }
                current_bit = current_bit + 1;
                i = i + 1;
            }
            packed_bits.push(byte);
        }
        BitArray {
            packed_bits,
            length
        }
    }
    // counts the rightmost bit_count bits
    fn from_u32(int: u32, bit_count: usize) -> Self {
        let mut bits = vec![0; bit_count];
        for i in 0..bit_count {
            if int & (0x01 << i) != 0 {
                bits[bit_count - (i + 1)] = 1;
            }
        }
        BitArray::new(&bits)
    }
    fn append(&mut self, from: &BitArray) {
        let byte_offset = self.length % 8;
        self.length = self.length + from.length;
        if byte_offset == 0 {
            self.packed_bits.extend(from.packed_bits.iter());
        }
        else {
            let rev_byte_offset = 8 - byte_offset;
            for byte in from.packed_bits.iter() {
                let byte = *byte;
                let last = self.packed_bits.last_mut().unwrap();
                *last = *last | (byte >> byte_offset);
                let next_byte = byte << rev_byte_offset;
                self.packed_bits.push(next_byte);
            }
            let byte_count = if self.length % 8 == 0 { self.length / 8 } else { (self.length / 8) + 1 };
            if self.packed_bits.len() > byte_count {
                self.packed_bits.pop();
            }
        }
    }
    fn range(&self, start: usize, end: usize) -> Self {
        let mut bits = Vec::new();
        let mut current_byte = start / 8;
        let mut current_offset = start % 8;
        let end_byte = end / 8;
        let end_offset = end % 8;
        while current_byte != end_byte || current_offset != end_offset {
            let bit = (self.packed_bits[current_byte] >> (7 - current_offset)) & 1;
            bits.push(bit);
            if current_offset == 7 {
                current_byte = current_byte + 1;
                current_offset = 0;
            }
            else {
                current_offset = current_offset + 1;
            }
        }
        BitArray::new(&bits)
    }
    fn to_u32(&self) -> u32 {
        let byte_shift = ((32 - self.length) / 8) + 1;
        let bit_shift = (8 - (self.length % 8)) % 8;
        let mut int = 0;
        for i in 0..self.packed_bits.len() {
            let total_offset: i32 = ((4 - (i + byte_shift)) * 8) as i32 - bit_shift as i32;
            let byte = self.packed_bits[i] as u32;
            if total_offset > 0 {
                int = int | byte << total_offset;
            }
            else {
                int = int | byte >> -total_offset;
            }
        }
        int
    }
}

fn get_code_length(code: u32) -> usize {
    let mut right_shift = 31;
    while ((code >> right_shift) & 1 == 0) && (right_shift != 0) {
        right_shift = right_shift - 1;
    }
    right_shift + 1
}
pub fn encode(data: &[u8], max_encodings: u32, reserved_codes: u32) -> EncodingResult {
    let mut encoding = Vec::new();
    let mut dictionary: std::collections::HashMap<&[u8],u32> = std::collections::HashMap::new();
    let initial_substrings: Vec<u8> = (0..=255).collect();
    for i in 0..=255 {
        dictionary.insert(&initial_substrings[i..=i], i as u32);
    }
    let mut scan_start = 0;
    let mut scan_end = 1;
    let mut code_length = if reserved_codes == 0 { 8 } else { 9 };
    while scan_start != data.len() {
        let scan = &data[scan_start..scan_end];
        if !dictionary.contains_key(scan) {
            let matching_code = dictionary.get(&data[scan_start..(scan_end - 1)]).unwrap();
            encoding.push(Code { value: *matching_code, length: code_length });
            let new_code = dictionary.len() as u32 + reserved_codes;
            if new_code > max_encodings {
                return EncodingResult { codes: encoding, bytes_processed: scan_end - 1 };
            }
            dictionary.insert(scan, new_code);
            code_length = get_code_length(new_code);
            scan_start = scan_end - 1;
        }
        else if scan_end == data.len() {
            let matching_code = dictionary.get(scan).unwrap();
            encoding.push(Code { value: *matching_code, length: code_length });
            scan_start = scan_end;
        }
        else {
            scan_end = scan_end + 1;
        }
    }
    EncodingResult { codes: encoding, bytes_processed: data.len() }
}
pub fn encode_all(data: &Vec<u8>, max_encodings: u32, reserved_codes: u32) -> Vec<EncodingResult> {
    let mut results = Vec::new();
    let mut processed = 0;
    while processed < data.len() {
        let result = encode(&data[processed..], max_encodings, reserved_codes);
        processed += result.bytes_processed;
        results.push(result);
    }
    results
}
pub fn decode(codes: &Vec<Code>, reserved_codes: u32) -> Vec<u8> {
    let mut data = Vec::new();
    let mut dictionary: std::collections::HashMap<u32,Vec<u8>> = std::collections::HashMap::new();
    let initial_substrings: Vec<u8> = (0..=255).collect();
    for i in 0..=255 {
        dictionary.insert(i as u32, vec![initial_substrings[i]]);
    }
    let previous_code = codes.first().unwrap();
    let mut previous_substring = dictionary.get(&previous_code.value).unwrap().clone();
    data.extend(previous_substring.iter());
    for current_code in &codes[1..] {
        // add the substring obtained by the current code to the data
        let (current_substring, new_substring) = match dictionary.get(&current_code.value) {
            Some(current_substring) => {
                // create a new dictionary entry using the previous substring and the first byte of the current substring
                let mut new_substring = previous_substring;
                new_substring.push(current_substring[0]);
                (current_substring.clone(), new_substring)
            }
            None => {
                // create a new dictionary entry using the previous substring with the first byte repeated at the end
                let mut new_substring = previous_substring;
                new_substring.push(new_substring[0]);
                (new_substring.clone(), new_substring)
            }
        };
        
        let new_code = dictionary.len() as u32 + reserved_codes;
        dictionary.insert(new_code, new_substring);
        data.extend(current_substring.iter());
        previous_substring = current_substring;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bitarray_correctly_stores_aligned_bits() {
        let array = BitArray::new(&[1,0,0,1,0,1,1,1,0,1,0,0,0,1,0,1]);
        assert_eq!(array.length, 16);
        assert_eq!(array.packed_bits, vec![0b10010111, 0b01000101]);
    }
    #[test]
    fn bitarray_correctly_stores_unaligned_bits() {
        let array = BitArray::new(&[1,0,0,1,0,1,1,1,0,1,0,0,0,1,0,1,0,1,1,1]);
        assert_eq!(array.length, 20);
        assert_eq!(array.packed_bits, vec![0b10010111, 0b01000101, 0b01110000]);
    }
    #[test]
    fn bitarray_correctly_appends_to_aligned_data() {
        let mut array = BitArray::new(&[1,0,0,1,0,1,1,1,0,1,0,0,0,1,0,1]);
        let oth_array = BitArray::new(&[1,0,0,1,0,0,0,1,1,1,0,1,1,0,0,1,1,0]);
        array.append(&oth_array);
        assert_eq!(array.length, 34);
        assert_eq!(array.packed_bits, vec![0b10010111, 0b01000101, 0b10010001, 0b11011001, 0b10000000]);
    }
    #[test]
    fn bitarray_correctly_appends_to_unaligned_data() {
        let mut array = BitArray::new(&[1,0,0,1,0,1,1,1,0,1,0,0,0,1,0,1,0,1,1,1]);
        let oth_array = BitArray::new(&[1,0,0,1,0,0,0,1,1,1,0,1,1,0,0,1,1,0]);
        array.append(&oth_array);
        assert_eq!(array.length, 38);
        assert_eq!(array.packed_bits, vec![0b10010111,0b01000101,0b01111001,0b00011101,0b10011000]);
    }
    #[test]
    fn bitarray_correctly_converts_from_u32() {
        let int: u32 = 0b00000000000000000100000101001000;
        // rightmost 20 are 00000100000101001000
        let array = BitArray::from_u32(int, 20);
        assert_eq!(array.length, 20);
        assert_eq!(array.packed_bits, vec![0b00000100,0b00010100,0b10000000])
    }
    #[test]
    fn bitarray_ignores_bits_past_bit_count_in_u32() {
        let int: u32 = 0b01111000000000000100000101001000;
        // rightmost 20 are 00000100000101001000
        let array = BitArray::from_u32(int, 20);
        assert_eq!(array.length, 20);
        assert_eq!(array.packed_bits, vec![0b00000100,0b00010100,0b10000000])
    }
    #[test]
    fn bitarray_returns_correct_range() {
        let array = BitArray::new(&[0,1,1,1,1,0,0,1,1,1,0,0,1]);
        let ranges = [
            array.range(0, 13),
            array.range(0, 0),
            array.range(13, 13),
            // on byte boundary
            array.range(0, 8),
            array.range(8, 12),
            // across byte boundary
            array.range(6, 12),
        ];
        assert_eq!(ranges[0].length, 13);
        assert_eq!(ranges[0].packed_bits, vec![0b01111001,0b11001000]);

        assert_eq!(ranges[1].length, 0);
        assert_eq!(ranges[1].packed_bits, vec![]);

        assert_eq!(ranges[2].length, 0);
        assert_eq!(ranges[2].packed_bits, vec![]);

        assert_eq!(ranges[3].length, 8);
        assert_eq!(ranges[3].packed_bits, vec![0b01111001]);

        assert_eq!(ranges[4].length, 4);
        assert_eq!(ranges[4].packed_bits, vec![0b11000000]);

        assert_eq!(ranges[5].length, 6);
        assert_eq!(ranges[5].packed_bits, vec![0b01110000]);
    }
    #[test]
    fn bitarray_converts_to_u32_correctly() {
        let array = BitArray::new(&[]);
        assert_eq!(array.to_u32(), 0b00000000000000000000000000000000);
        let array = BitArray::new(&[0,1,0,0,0,0,0,1,0,1,0,1,0,0,0,1,1,1,0,0,0,1,1,0,0,0,0,0,1,0,0,0]);
        assert_eq!(array.to_u32(), 0b01000001010100011100011000001000);
        let array = BitArray::new(&[1]);
        assert_eq!(array.to_u32(), 0b00000000000000000000000000000001);
        let array = BitArray::new(&[0,1,1,0,1,0]);
        assert_eq!(array.to_u32(), 0b00000000000000000000000000011010);
        let array = BitArray::new(&[1,0,1,1,1,1,1,1,1,1,1,1]);
        assert_eq!(array.to_u32(), 0b00000000000000000000101111111111);
        let array = BitArray::new(&[1,1,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0]);
        assert_eq!(array.to_u32(), 0b00000000000110111111111111111110);
    }

    #[test]
    fn get_code_length_returns_correct_length() {
        let codes: [u32; 8] = [
            // 1
            0b00000000000000000000000000000001,
            // 1 (code is zero)
            0b00000000000000000000000000000000,
            // 32
            0b10000000000000000000000000000000,
            // 19
            0b00000000000001000000000000000000,
            // 12
            0b00000000000000000000100000000000,
            // 16
            0b00000000000000001000000000000000,
            // 22
            0b00000000001000000000000000000000,
            // 13
            0b00000000000000000001000000000000,
        ];
        assert_eq!(get_code_length(codes[0]), 1);
        assert_eq!(get_code_length(codes[1]), 1);
        assert_eq!(get_code_length(codes[2]), 32);
        assert_eq!(get_code_length(codes[3]), 19);
        assert_eq!(get_code_length(codes[4]), 12);
        assert_eq!(get_code_length(codes[5]), 16);
        assert_eq!(get_code_length(codes[6]), 22);
        assert_eq!(get_code_length(codes[7]), 13);
    }

    #[test]
    fn encode_produces_expected_code() {
        let data: Vec<u8> = vec![5,6,7,8,5,6,7,5,6,7,7,6,5,4];
        //expected generation sequence will be
        // [5] -> 5, [5,6] = 256, 8 bits
        // [6] -> 6, [6,7] = 257, 9 bits
        // [7] -> 7, [7,8] = 258
        // [8] -> 8, [8,5] = 259
        // [5,6] -> 256, [5,6,7] = 260
        // [7] -> 7, [7,5] = 261
        // [5,6,7] -> 260, [5,6,7,7] = 262
        // [7] -> 7, [7,6] = 263
        // [6] -> 6, [6,5] = 264
        // [5] -> 5, [5,4] = 265
        // [4] -> 4
        let encoding = encode(&data, 4095, 0);
        assert_eq!(0b00000101, encoding.codes[0].value);
        assert_eq!(8, encoding.codes[0].length);
        assert_eq!(0b000000110, encoding.codes[1].value);
        assert_eq!(9, encoding.codes[1].length);
        assert_eq!(0b000000111, encoding.codes[2].value);
        assert_eq!(9, encoding.codes[2].length);
        assert_eq!(0b000001000, encoding.codes[3].value);
        assert_eq!(9, encoding.codes[3].length);
        assert_eq!(0b100000000, encoding.codes[4].value);
        assert_eq!(9, encoding.codes[4].length);
        assert_eq!(0b000000111, encoding.codes[5].value);
        assert_eq!(9, encoding.codes[5].length);
        assert_eq!(0b100000100, encoding.codes[6].value);
        assert_eq!(9, encoding.codes[6].length);
        assert_eq!(0b000000111, encoding.codes[7].value);
        assert_eq!(9, encoding.codes[7].length);
        assert_eq!(0b000000110, encoding.codes[8].value);
        assert_eq!(9, encoding.codes[8].length);
        assert_eq!(0b000000101, encoding.codes[9].value);
        assert_eq!(9, encoding.codes[9].length);
        assert_eq!(0b000000100, encoding.codes[10].value);
        assert_eq!(9, encoding.codes[10].length);
    }
    #[test]
    fn decode_reproduces_original_data() {
        let data: Vec<u8> = vec![5,6,7,8,5,6,7,5,6,7,7,6,5,4];
        let encoding = encode(&data, 4095, 0);
        let decoding = decode(&encoding.codes, 0);
        assert_eq!(data, decoding);
    }
    #[test]
    fn decode_reproduces_data_for_repetitive_blocks() {
        let data: Vec<u8> = vec![5; 2000];
        let encoding = encode(&data, 4095, 2);
        let decoding = decode(&encoding.codes, 2);
        assert_eq!(data, decoding);
    }
    #[test]
    fn decode_works_for_large_data() {
        let mut i = 0;
        let mut data = Vec::new();
        for _ in 0..30000 {
            data.push(i as u8);
            i = (i + 131) % 256;
        }
        let encoding = encode(&data, 4095, 0);
        let decoding = decode(&encoding.codes, 0);
        assert_eq!(data, decoding);
    }
    #[test]
    fn encode_will_only_work_until_max_code_length_reached() {
        let mut i = 0;
        let mut data = Vec::new();
        for _ in 0..30000 {
            data.push(i as u8);
            i = (i + 131) % 256;
        }
        let encoding = encode(&data, 511, 0);
        assert_ne!(encoding.bytes_processed, 30000);
    }
    #[test]
    fn encode_decode_reproduces_data_when_codes_are_reserved() {
        let mut i = 0;
        let mut data = Vec::new();
        for _ in 0..30000 {
            data.push(i as u8);
            i = (i + 131) % 256;
        }
        let encoding = encode(&data, 4095, 12);
        let decoding = decode(&encoding.codes, 12);
        assert_eq!(data, decoding);
    }
    #[test]
    fn encode_all_does_not_corrupt_data() {
        let mut i = 0;
        let mut data = Vec::new();
        for _ in 0..30000 {
            data.push(i as u8);
            i = (i + 131) % 256;
        }
        let encodings = encode_all(&data, 511, 0);
        let mut decoding = Vec::new();
        for encoding in encodings {
            decoding.extend(decode(&encoding.codes, 0));
        }
        assert_eq!(decoding, data);
    }
}
