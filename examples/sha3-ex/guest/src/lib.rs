#![cfg_attr(feature = "guest", no_std)]

#[jolt::provable]
fn sha3(input: &[u8]) -> [u8; 32] {
    jolt::keccak256::Keccak256::digest(input)
}
