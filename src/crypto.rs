use rand::prelude::*;

pub fn challenge() -> [u8; 32] {
    let mut data = [0u8; 32];
    rand::rng().fill_bytes(&mut data);
    data
}
