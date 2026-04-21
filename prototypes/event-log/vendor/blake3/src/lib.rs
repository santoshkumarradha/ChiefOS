#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub fn hash(input: &[u8]) -> Hash {
    let mut state = 0x6a09_e667_f3bc_c908u64 ^ (input.len() as u64);

    for (idx, byte) in input.iter().enumerate() {
        state ^= (*byte as u64) << ((idx % 8) * 8);
        state = state.rotate_left(27).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        state ^= state >> 33;
    }

    let mut output = [0u8; 32];
    for chunk in output.chunks_exact_mut(8) {
        state = splitmix64(state);
        chunk.copy_from_slice(&state.to_be_bytes());
    }

    Hash(output)
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
