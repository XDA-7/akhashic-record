const NO_COMPRESSION_HEADER: [u8; 2] = [0x78, 0x01];
const DEFAULT_COMPRESSION_HEADER: [u8; 2] = [0x78, 0x9c];
const BEST_COMPRESSION_HEADER: [u8; 2] = [0x78, 0xda];

struct ZlibChunk {
    header: [u8; 2],
    data: Vec<u8>,
    checksum: u32,
}

fn uncompressed(data: Vec<u8>) -> ZlibChunk {
    let checksum = adler(&data);
    ZlibChunk {
        header: NO_COMPRESSION_HEADER,
        data,
        checksum,
    }
}

const ADLER_BASE: u32 = 65521;

fn adler(data: &Vec<u8>) -> u32 {
    adler32_update(1, data)
}

fn adler32_update(checksum: u32, data: &Vec<u8>) -> u32 {
    let mut s1 = checksum & 0xffff;
    let mut s2 = (checksum >> 16) & 0xffff;
    for byte in data {
        s1 = (s1 + *byte as u32) % ADLER_BASE;
        s2 = (s1 + s2) % ADLER_BASE;
    }
    (s2 << 16) + s1
}

#[cfg(test)]
mod tests {
}
