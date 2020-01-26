const BIT_MASK: [u8; 8] = [
    0b10000000,
    0b01000000,
    0b00100000,
    0b00010000,
    0b00001000,
    0b00000100,
    0b00000010,
    0b00000001,
];

pub struct BitArray {
    packed_bits: Vec<u8>,
    length: usize,
}
impl BitArray {
    pub fn new(bits: &[u8]) -> Self {
        let mut packed_bits = Vec::new();
        let length = bits.len();
        let mut current_bit = 0;
        while current_bit < length {
            let mut byte: u8 = 0;
            let mut i = 0;
            while current_bit < length && i < 8 {
                if bits[current_bit] != 0 {
                    byte = byte | BIT_MASK[i];
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
    pub fn append(&mut self, from: &BitArray) {
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
}

pub fn encode(data: &Vec<u8>) -> Vec<u8> {
    let mut dictionary: std::collections::HashMap<&[u8],u32> = std::collections::HashMap::new();
    let initial_substrings: Vec<u8> = (0..255).collect();
    for i in 0..255 {
        dictionary.insert(&initial_substrings[i..=i], i as u32);
    }
    vec![]
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
}
