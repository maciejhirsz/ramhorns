/// FNV-1a implementation
pub fn hash<B: AsRef<[u8]>>(bytes: B) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes.as_ref() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
