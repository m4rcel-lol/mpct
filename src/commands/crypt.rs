use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use argon2::Argon2;
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};

use crate::{
    cli::{CryptArgs, CryptCommand, GlobalOptions},
    error, output,
};

const MAGIC: &[u8] = b"MPCTCRYPT1\0";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

pub fn run(cmd: &CryptCommand, _global: &GlobalOptions) -> Result<()> {
    match cmd {
        CryptCommand::Encrypt(args) => encrypt(args),
        CryptCommand::Decrypt(args) => decrypt(args),
    }
}

fn encrypt(args: &CryptArgs) -> Result<()> {
    let passphrase = read_passphrase(args.passphrase_stdin, true)?;
    let input = fs::read(&args.file)
        .with_context(|| format!("failed to read input file {}", args.file.display()))?;
    let out = args
        .out
        .clone()
        .unwrap_or_else(|| default_encrypt_path(&args.file));
    output::overwrite_confirm(&out)?;

    let mut salt = [0_u8; SALT_LEN];
    let mut nonce = [0_u8; NONCE_LEN];
    rand::fill(&mut salt);
    rand::fill(&mut nonce);
    let key = derive_key(passphrase.as_bytes(), &salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), input.as_ref())
        .map_err(|_| error::msg("encryption failed"))?;

    let mut container = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    container.extend_from_slice(MAGIC);
    container.extend_from_slice(&salt);
    container.extend_from_slice(&nonce);
    container.extend_from_slice(&ciphertext);
    fs::write(&out, container).with_context(|| format!("failed to write {}", out.display()))?;
    println!("{}", out.display());
    Ok(())
}

fn decrypt(args: &CryptArgs) -> Result<()> {
    let passphrase = read_passphrase(args.passphrase_stdin, false)?;
    let input = fs::read(&args.file)
        .with_context(|| format!("failed to read encrypted file {}", args.file.display()))?;
    let (salt, nonce, ciphertext) = parse_container(&input)?;
    let key = derive_key(passphrase.as_bytes(), salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| error::msg("decryption failed: wrong passphrase or tampered ciphertext"))?;

    let out = args
        .out
        .clone()
        .unwrap_or_else(|| default_decrypt_path(&args.file));
    output::overwrite_confirm(&out)?;
    fs::write(&out, plaintext).with_context(|| format!("failed to write {}", out.display()))?;
    println!("{}", out.display());
    Ok(())
}

fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    let mut key = [0_u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(passphrase, salt, &mut key)
        .map_err(|err| error::msg(format!("key derivation failed: {err}")))?;
    Ok(key)
}

fn parse_container(input: &[u8]) -> Result<(&[u8], &[u8], &[u8])> {
    let min_len = MAGIC.len() + SALT_LEN + NONCE_LEN + 16;
    if input.len() < min_len || !input.starts_with(MAGIC) {
        return Err(error::msg("encrypted file is not an mpct crypt container"));
    }
    let salt_start = MAGIC.len();
    let nonce_start = salt_start + SALT_LEN;
    let cipher_start = nonce_start + NONCE_LEN;
    Ok((
        &input[salt_start..nonce_start],
        &input[nonce_start..cipher_start],
        &input[cipher_start..],
    ))
}

fn read_passphrase(passphrase_stdin: bool, confirm: bool) -> Result<String> {
    if passphrase_stdin {
        let mut value = String::new();
        io::stdin()
            .read_to_string(&mut value)
            .context("failed to read passphrase from stdin")?;
        return Ok(value.trim_end_matches(['\r', '\n']).to_string());
    }

    let first = rpassword::prompt_password("Passphrase: ")
        .context("failed to read passphrase from terminal")?;
    if confirm {
        let second = rpassword::prompt_password("Confirm passphrase: ")
            .context("failed to read passphrase confirmation")?;
        if first != second {
            return Err(error::msg("passphrases did not match"));
        }
    }
    Ok(first)
}

fn default_encrypt_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.mpct", path.display()))
}

fn default_decrypt_path(path: &Path) -> PathBuf {
    let text = path.display().to_string();
    if let Some(stripped) = text.strip_suffix(".mpct") {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(format!("{text}.dec"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_container() {
        assert!(parse_container(b"not encrypted").is_err());
    }
}
