const POLYNOMIAL: u32 = 0xedb88320;

struct Crc {
    table: [u32; 256],
}
impl Crc {
    fn new() -> Self {
        let mut table: [u32; 256] = [0; 256];
        for n in 0..256 {
            let mut c: u32 = n;
            for _ in 0..8 {
                if (c & 1) != 0 {
                    c = POLYNOMIAL ^ (c >> 1);
                }
                else {
                    c = c >> 1;
                }
            }
            table[n as usize] = c;
        }
        Crc {
            table
        }
    }
    fn calculate(&self, data: &Vec<u8>) -> u32 {
        let mut c: u32 = 0xffffffff;
        for byte in data {
            let table_index = (c ^ *byte as u32) & 0xff;
            c = self.table[table_index as usize] ^ (c >> 8);
        }
        c ^ 0xffffffff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crc_returns_same_sum_for_same_data() {
        let crc = Crc::new();
        let data_one: Vec<u8> = vec![
            53, 34, 0, 2, 76, 143, 231, 67, 88, 19
        ];
        let data_two: Vec<u8> = vec![
            53, 34, 0, 2, 76, 143, 231, 67, 88, 19
        ];
        assert_eq!(crc.calculate(&data_one), crc.calculate(&data_two));
    }
    #[test]
    fn crc_returns_different_sum_if_data_corrupted() {
        let crc = Crc::new();
        let data_one: Vec<u8> = vec![
            53, 34, 0, 2, 76, 143, 231, 67, 88, 19
        ];
        let data_two: Vec<u8> = vec![
            53, 34, 0, 2, 76, 143, 231, 67, 21, 19
        ];
        // same bytes but different order
        let data_three: Vec<u8> = vec![
            53, 34, 0, 2, 76, 143, 231, 19, 88, 67
        ];
        assert_ne!(crc.calculate(&data_one), crc.calculate(&data_two));
        assert_ne!(crc.calculate(&data_one), crc.calculate(&data_three));
    }
    #[test]
    fn crc_provides_correct_checksum_for_known_values() {
        let crc = Crc::new();
        let values: Vec<Vec<u8>> = vec![
            vec![0x53, 0x34, 0x00, 0x02, 0x76, 0xea, 0xb6, 0x67, 0x88, 0x19],
            vec![0x90, 0x90, 0x23, 0x65, 0x1a, 0xc3],
            vec![0x19, 0x33, 0xa2, 0xc1, 0x64, 0x39, 0x99, 0x02],
            vec![0x55, 0x88, 0x93, 0x61, 0x47, 0x32, 0x36, 0x90, 0x09, 0x71, 0x14],
            vec![0xd1, 0xff, 0xf1, 0x53, 0x88, 0x56, 0x21, 0x45, 0x37, 0x58, 0x04, 0xda, 0x03, 0xc5, 0x38],
        ];
        // correct values obtained through the sample implementation in C from the png specification
        assert_eq!(crc.calculate(&values[0]), 0x8449208);
        assert_eq!(crc.calculate(&values[1]), 0x3379f03a);
        assert_eq!(crc.calculate(&values[2]), 0xce8fd51c);
        assert_eq!(crc.calculate(&values[3]), 0xf5b137);
        assert_eq!(crc.calculate(&values[4]), 0x3d359f36);
    }
}
