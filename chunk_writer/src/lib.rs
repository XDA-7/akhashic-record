use std::fs;

trait Chunk {
    fn append(self, file: &fs::File);
}

trait Image {
    fn write(self, path: &str) -> Result<fs::File, std::io::Error>;
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
        self.header.append(&file_image);
        match self.palette {
            Some(palette) => palette.append(&file_image),
            None => (),
        }
        for data_chunk in self.data {
            data_chunk.append(&file_image);
        }
        self.end.append(&file_image);
        Ok(file_image)
    }
}

struct HeaderChunk {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

impl Chunk for HeaderChunk {
    fn append(self, file: &fs::File) {}
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
    fn append(self, file: &fs::File) {}
}

struct Scanline {
    samples: Vec<u8>,
}

struct DataChunk {
    scanlines: Vec<Scanline>,
}

impl Chunk for DataChunk {
    fn append(self, file: &fs::File) {}
}

struct EndChunk {}

impl Chunk for EndChunk {
    fn append(self, file: &fs::File) {}
}