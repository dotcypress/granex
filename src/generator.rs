use crypto::digest::Digest;
use crypto::ed25519::keypair;
use crypto::sha3::Sha3;
use rand::{thread_rng, Rng};
use std::fs::{create_dir_all, File};
use std::io::{Result, Write};
use std::path::PathBuf;

const ALPHABET: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz234567";
const SECRET_KEY_PREFIX: &'static [u8] = b"== ed25519v1-secret: type0 ==\0\0\0";
const CHECKSUM_PREFIX: &'static [u8] = b".onion checksum";
const VERSION_SUFFIX: [u8; 1] = [3];

pub fn start(prefix: &str, output_dir: PathBuf) -> Result<()> {
    let mut rng = thread_rng();
    let mut seed = [0u8; 20];
    loop {
        rng.fill(&mut seed[..]);
        let (secret, address) = generate_address(&seed);
        if address.starts_with(prefix) {
            save_key(&output_dir, secret, &address)?;
            return Ok(());
        }
    }
}

fn generate_address(seed: &[u8]) -> ([u8; 64], String) {
    let (secret, pub_key) = keypair(seed);
    let mut hasher = Sha3::sha3_256();
    let mut checksum = [0; 32];
    hasher.input(&CHECKSUM_PREFIX);
    hasher.input(&pub_key);
    hasher.input(&VERSION_SUFFIX);
    hasher.result(&mut checksum);
    let address = base32encode(&[&pub_key[..], &checksum[..2], &[3]].concat());
    (secret, address)
}

fn save_key(output_dir: &PathBuf, secret: [u8; 64], address: &str) -> Result<()> {
    println!("address found: {:}.onion", address);

    let mut secret_path = output_dir.clone();
    secret_path.push(&address);
    create_dir_all(&secret_path)?;

    secret_path.push("hs_ed25519_secret_key");
    let mut file = File::create(&secret_path)?;
    file.write_all(&SECRET_KEY_PREFIX)?;
    file.write_all(&secret)?;
    Ok(())
}

fn base32encode(data: &[u8]) -> String {
    let mut res = Vec::with_capacity((data.len() + 3) / 4 * 5);
    for chunk in data.chunks(5) {
        let buf = {
            let mut buf = [0u8; 5];
            for (i, &b) in chunk.iter().enumerate() {
                buf[i] = b;
            }
            buf
        };
        res.push(ALPHABET[((buf[0] & 0xF8) >> 3) as usize]);
        res.push(ALPHABET[(((buf[0] & 0x07) << 2) | ((buf[1] & 0xC0) >> 6)) as usize]);
        res.push(ALPHABET[((buf[1] & 0x3E) >> 1) as usize]);
        res.push(ALPHABET[(((buf[1] & 0x01) << 4) | ((buf[2] & 0xF0) >> 4)) as usize]);
        res.push(ALPHABET[(((buf[2] & 0x0F) << 1) | (buf[3] >> 7)) as usize]);
        res.push(ALPHABET[((buf[3] & 0x7C) >> 2) as usize]);
        res.push(ALPHABET[(((buf[3] & 0x03) << 3) | ((buf[4] & 0xE0) >> 5)) as usize]);
        res.push(ALPHABET[(buf[4] & 0x1F) as usize]);
    }

    if data.len() % 5 != 0 {
        let len = res.len();
        let num_extra = 8 - (data.len() % 5 * 8 + 4) / 5;
        res.truncate(len - num_extra);
    }
    String::from_utf8(res).unwrap()
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;

    #[bench]
    fn bench_address_generate(b: &mut test::Bencher) {
        let mut rng = thread_rng();
        let mut seed = [0u8; 20];
        b.iter(move || {
            rng.fill(&mut seed[..]);
            generate_address(&seed);
        });
    }
}
