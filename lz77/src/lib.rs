const WINDOW_SIZE: u32 = 32 * 1024;
const MIN_MATCH: u32 = 3;
const MAX_MATCH: u32 = 258;

struct StringMatch {
    offset: u32,
    length: u32,
    byte_after: u32,
}

fn find_match(input: &[u8], search: &[u8]) -> StringMatch {
    unimplemented!();
}

fn compress(data: &Vec<u8>) -> Vec<u8> {
    let mut compression = Vec::new();
    compression
}

#[cfg(test)]
mod tests {
}
