use std::fs;
use std::io::Write;


trait Image {
    fn write(self, path: &str) -> Result<fs::File, std::io::Error>;
}

trait Chunk {
    fn append(self, bytes: &mut Vec<u8>);
}

fn int_to_bytes(val: u32) -> [u8; 4] {
    [
        (val >> 24) as u8,
        ((val >> 16) & 0xff) as u8,
        ((val >> 8) & 0xff) as u8,
        (val & 0xff) as u8,
    ]
}

fn bytes_to_int(val: [u8; 4]) -> u32 {
    (val[0] as u32) << 24 |
    (val[1] as u32) << 16 |
    (val[2] as u32) <<  8 |
    (val[3] as u32)
}
    
mod png {
    use super::*;
    const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    const HEADER_SIGNATURE: [u8; 4] = [0x49, 0x48, 0x44, 0x52];
    const PALETTE_SIGNATURE: [u8; 4] = [0x50, 0x4c, 0x54, 0x45];
    const DATA_SIGNATURE: [u8; 4] = [0x49, 0x44, 0x41, 0x54];
    const END_SIGNATURE: [u8; 4] = [0x49, 0x45, 0x4e, 0x44];
    fn get_chunk_bytes(signature: &[u8; 4], data: &Vec<u8>) -> Vec<u8> {
        let mut chunk_data: Vec<u8> = Vec::new();
        //Length
        chunk_data.extend(&int_to_bytes(data.len() as u32));
        //Chunk Type
        chunk_data.extend(signature);
        //Chunk Data
        chunk_data.extend(data);
        //CRC
        let crc = crc::Crc::new();
        let checksum = crc.calculate(&Vec::from(chunk_data.split_at(4).1));
        chunk_data.extend(&int_to_bytes(checksum));
        chunk_data
    }

    struct PngImage {
        header: HeaderChunk,
        palette: Option<PaletteChunk>,
        data: Vec<DataChunk>,
        end: EndChunk,
    }
    impl Image for PngImage {
        fn write(self, path: &str) -> Result<fs::File, std::io::Error> {
            let file_image = match fs::File::create(path) {
                Ok(file_image) => file_image,
                Err(e) => return Err(e),
            };
            let mut bytes: Vec<u8> = Vec::new();
            bytes.extend(&PNG_SIGNATURE);
            self.header.append(&mut bytes);
            match self.palette {
                Some(palette) => palette.append(&mut bytes),
                None => (),
            }
            for data_chunk in self.data {
                data_chunk.append(&mut bytes);
            }
            self.end.append(&mut bytes);
            Ok(file_image)
        }
    }

    enum ColorType {
        GrayScale,
        TrueColor,
        Palette,
        GrayScaleAlpha,
        TrueColorAlpha,
    }
    
    enum InterlaceType {
        None,
        Adam7,
    }
    
    struct HeaderChunk {
        width: u32,
        height: u32,
        color_type: ColorType,
        interlace_method: InterlaceType,
    }
    impl Chunk for HeaderChunk {
        fn append(self, bytes: &mut Vec<u8>) {
            let mut data = Vec::new();
            data.extend(&int_to_bytes(self.width));
            data.extend(&int_to_bytes(self.height));
            // bit depth will be fixed to 8 for now
            data.push(8);
            data.push(match self.color_type {
                ColorType::GrayScale => 0,
                ColorType::TrueColor => 2,
                ColorType::Palette => 3,
                ColorType::GrayScaleAlpha => 4,
                ColorType::TrueColorAlpha => 6,
            });
            // the only compression supported by the spec is type 0
            data.push(0);
            // the same goes for the filter method
            data.push(0);
            data.push(match self.interlace_method {
                InterlaceType::None => 0,
                InterlaceType::Adam7 => 1,
            });
            bytes.extend(get_chunk_bytes(&HEADER_SIGNATURE, &data));
        }
    }
    
    struct PaletteValue {
        red: u8,
        green: u8,
        blue: u8,
    }
    
    struct PaletteChunk {
        entries: Vec<PaletteValue>,
    }
    impl Chunk for PaletteChunk {
        fn append(self, bytes: &mut Vec<u8>) {
            let mut data = Vec::new();
            for entry in self.entries {
                data.push(entry.red);
                data.push(entry.green);
                data.push(entry.blue);
            }
            bytes.extend(get_chunk_bytes(&PALETTE_SIGNATURE, &data));
        }
    }
    
    struct Scanline {
        samples: Vec<u8>,
    }
    
    struct DataChunk {
        scanlines: Vec<Scanline>,
    }
    impl Chunk for DataChunk {
        fn append(self, bytes: &mut Vec<u8>) {}
    }
    
    struct EndChunk {}
    impl Chunk for EndChunk {
        fn append(self, bytes: &mut Vec<u8>) {}
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn get_chunk_bytes_returns_correct_results() {
            let signature: [u8; 4] = [50,21,33,2];
            let data: Vec<u8> = vec![32,22,65,156,43,229,186,70,82,68,41,55,90,66,21];
            let checked_data: Vec<u8> = vec![50,21,33,2, 32,22,65,156,43,229,186,70,82,68,41,55,90,66,21];
    
            let length = int_to_bytes(15);
            let checksum = crc::Crc::new().calculate(&checked_data);
            let checksum = int_to_bytes(checksum);
    
            let chunk_bytes = get_chunk_bytes(&signature, &data);
            for i in 0..4 {
                assert_eq!(chunk_bytes[i], length[i]);
            }
            for i in 0..4 {
                assert_eq!(chunk_bytes[i + 4], signature[i]);
            }
            for i in 0..15 {
                assert_eq!(chunk_bytes[i + 8], data[i]);
            }
            for i in 0..4 {
                assert_eq!(chunk_bytes[i + 23], checksum[i]);
            }
        }
    }
}

mod gif {
    use super::*;
    use std::fs;
    use image;
    use lzw;
    const HEADER_SIGNATURE: [u8; 6] = [0x47,0x49,0x46,0x38,0x39,0x61];
    // 256 RGB colours in the palette
    const COLOR_TABLE_FIELDS: u8 = 0xf7;
    const IMAGE_DATA_SENTINEL: u8 = 0x2c;
    const EOF_SENTINEL: u8 = 0x3b;

    fn insert_color_table(palette: &image::Palette, data: &mut Vec<u8>) {
        for i in 0..256 {
            match palette.color(i) {
                Some(color) => {
                    data.push(color.red);
                    data.push(color.green);
                    data.push(color.blue);
                },
                None => {
                    data.push(0);
                    data.push(0);
                    data.push(0);
                },
            }
        }
    }
    fn insert_color_data(pixels: &Vec<u8>, height: &[u8; 2], width: &[u8; 2], data: &mut Vec<u8>) {
        data.push(IMAGE_DATA_SENTINEL);
        // top right is the origin
        data.extend(&[0; 4]);
        data.extend(width);
        data.extend(height);
        // no local color table
        data.push(0);
        // minimum code length of 8
        data.push(8);
        let encoding = lzw::encode(pixels);
        data.push(encoding.packed_bits.len() as u8);
        data.extend(encoding.packed_bits);
    }
    fn truncate_usize_vec(data: &Vec<usize>) -> Vec<u8> {
        let mut result = Vec::new();
        for byte in data {
            result.push(*byte as u8);
        }
        result
    }

    struct GifImage {
        image: image::ColorImage,
    }
    impl Image for GifImage {
        fn write(self, path: &str) -> Result<fs::File, std::io::Error> {
            let mut file_image = match fs::File::create(path) {
                Ok(file_image) => file_image,
                Err(e) => return Err(e),
            };
            let mut data: Vec<u8> = vec![];
            data.extend(&HEADER_SIGNATURE);
            let height = self.image.canvas.height as u16;
            let height = [(height >> 8) as u8, height as u8];
            data.extend(&height);
            let width = self.image.canvas.width as u16;
            let width = [(width >> 8) as u8, width as u8];
            data.extend(&width);
            data.push(COLOR_TABLE_FIELDS);
            // background color: 0
            data.push(0);
            // default pixel aspect ratio
            data.push(0);
            insert_color_table(&self.image.palette, &mut data);
            // skip Graphic Control Extension
            let truncated_vec = truncate_usize_vec(&self.image.canvas.pixels);
            insert_color_data(&truncated_vec, &height, &width, &mut data);
            data.push(EOF_SENTINEL);
            file_image.write_all(&data).unwrap();
            Ok(file_image)
        }
    }
}
