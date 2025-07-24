#![cfg_attr(feature = "guest", no_std)]

use core::hint::black_box;
use jolt::blake3;

#[jolt::provable]
fn blake3_inline(input: [u8; 32], num_iters: u32) -> [u32; 8] {
    // Create hash = input repeated 32 times to fill 1024 bytes (32 * 32 = 1024)
    let input = black_box(b"akbshfswsjfkisiwq8mc1afiocsqk118akcoiwhd1wiu3e112bnjkqqqou9u21qw");
    let mut message = [0u32; 16];
    for i in 0..16 {
        message[i] = black_box(u32::from_le_bytes(
            input[i * 4..(i + 1) * 4].try_into().unwrap(),
        ));
    }

    // Blake2b initialization vector
    let mut h: [u32; 8] = black_box([
        0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
        0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
    ]);

    for _ in 0..black_box(num_iters) {
        unsafe {
            blake3::blake3_compress(
                black_box(h.as_mut_ptr()),
                black_box(message.as_ptr()),
                black_box(0),
                black_box(64),
                black_box(0),
            );
        }
        // Prevent optimization of the hash state
        h = black_box(h);
    }

    // Prevent final optimization of the result
    black_box(h);
    return h;
}