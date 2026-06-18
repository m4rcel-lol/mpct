use std::{fs, net::TcpListener};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn mpct() -> Command {
    Command::cargo_bin("mpct").expect("binary exists")
}

#[test]
fn banner_and_core_help_work() {
    mpct()
        .assert()
        .success()
        .stdout(predicate::str::contains("mpct — one binary. every tool."));

    mpct()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands:"));

    mpct()
        .arg("banner")
        .assert()
        .success()
        .stdout(predicate::str::contains("version"));
}

#[test]
fn hash_checksum_and_codecs_work() {
    mpct()
        .args(["hash", "sha256", "abc"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ));

    mpct()
        .args(["hash", "verify", "sha256", "bad", "abc"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("MISMATCH"));

    let dir = tempdir().unwrap();
    let file = dir.path().join("data.txt");
    fs::write(&file, "abc").unwrap();
    mpct()
        .args(["checksum", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(file.to_str().unwrap()));

    mpct()
        .args(["encode", "base64", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("aGVsbG8="));
    mpct()
        .args(["decode", "hex", "6869"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hi"));
}

#[test]
fn identity_password_totp_and_jwt_work() {
    mpct()
        .args(["uuid", "--v4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-"));
    mpct().arg("ulid").assert().success();
    mpct()
        .args(["nanoid", "--size", "8"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"(?m)^.{8}$").unwrap());
    mpct()
        .args(["pass", "gen", "--length", "12"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"(?m)^.{12}$").unwrap());
    mpct()
        .args(["pass", "phrase", "--words", "4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-"));
    mpct()
        .args(["pass", "strength", "correct horse battery staple"])
        .assert()
        .success()
        .stdout(predicate::str::contains("strength:"));
    mpct()
        .args(["totp", "gen", "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"(?m)^[0-9]{6}$").unwrap());
    mpct()
        .args([
            "jwt",
            "decode",
            "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjMifQ.x",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"sub\""));
}

#[test]
fn data_regex_case_text_and_time_work() {
    let dir = tempdir().unwrap();
    let json = dir.path().join("data.json");
    fs::write(&json, r#"{"users":[{"email":"a@example.com"}]}"#).unwrap();
    mpct()
        .args([
            "json",
            "query",
            ".users[0].email",
            "--file",
            json.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("a@example.com"));

    let yaml = dir.path().join("data.yaml");
    fs::write(&yaml, "name: mpct\n").unwrap();
    mpct()
        .args(["yaml", "to-json", "--file", yaml.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\""));

    let toml = dir.path().join("data.toml");
    fs::write(&toml, "name = \"mpct\"\n").unwrap();
    mpct()
        .args(["toml", "to-json", "--file", toml.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("mpct"));

    mpct()
        .args(["regex", "replace", "\\s+", "-", "hello world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello-world"));
    mpct()
        .args(["case", "snake", "Hello world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello_world"));
    mpct()
        .args(["text", "slug", "Hello, MPCT!"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello-mpct"));
    mpct()
        .args(["time", "iso", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1970-01-01T00:00:00"));
}

#[test]
fn qr_color_diff_crypt_net_sysinfo_and_completions_work() {
    mpct()
        .args(["qr", "gen", "hello", "--ascii"])
        .assert()
        .success();
    mpct()
        .args(["color", "convert", "#ff0000", "--to", "rgb"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rgb(255, 0, 0)"));

    let dir = tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    fs::write(&a, "one\n").unwrap();
    fs::write(&b, "two\n").unwrap();
    mpct()
        .args([
            "diff",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--unified",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("-one"));

    let plain = dir.path().join("plain.txt");
    let enc = dir.path().join("plain.txt.mpct");
    let dec = dir.path().join("plain.dec.txt");
    fs::write(&plain, "secret").unwrap();
    mpct()
        .args([
            "crypt",
            "encrypt",
            plain.to_str().unwrap(),
            "-o",
            enc.to_str().unwrap(),
            "--passphrase-stdin",
        ])
        .write_stdin("passphrase\n")
        .assert()
        .success();
    mpct()
        .args([
            "crypt",
            "decrypt",
            enc.to_str().unwrap(),
            "-o",
            dec.to_str().unwrap(),
            "--passphrase-stdin",
        ])
        .write_stdin("passphrase\n")
        .assert()
        .success();
    assert_eq!(fs::read_to_string(dec).unwrap(), "secret");

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    mpct()
        .args(["net", "port", "127.0.0.1", &port])
        .assert()
        .success()
        .stdout(predicate::str::contains("open"));

    mpct()
        .arg("sysinfo")
        .assert()
        .success()
        .stdout(predicate::str::contains("arch:"));
    mpct()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mpct"));
    mpct().arg("clip").arg("--help").assert().success();
}
