# MPCT

```text
__/\\\\____________/\\\\__/\\\\\\\\\\\\\__________/\\\\\\\\\__/\\\\\\\\\\\\\\\_        
 _\/\\\\\\________/\\\\\\_\/\\\/////////\\\_____/\\\////////__\///////\\\/////__       
  _\/\\\//\\\____/\\\//\\\_\/\\\_______\/\\\___/\\\/_________________\/\\\_______      
   _\/\\\\///\\\/\\\/_\/\\\_\/\\\\\\\\\\\\\/___/\\\___________________\/\\\_______     
    _\/\\\__\///\\\/___\/\\\_\/\\\/////////____\/\\\___________________\/\\\_______    
     _\/\\\____\///_____\/\\\_\/\\\_____________\//\\\__________________\/\\\_______   
      _\/\\\_____________\/\\\_\/\\\______________\///\\\________________\/\\\_______  
       _\/\\\_____________\/\\\_\/\\\________________\////\\\\\\\\\_______\/\\\_______ 
        _\///______________\///__\///____________________\/////////________\///________
```

mpct — one binary. every tool. MPCT is a script-friendly command-line toolkit for hashing, encoding, document formatting, IDs, passwords, time, QR codes, colors, diffs, file encryption, networking, clipboard access, system information, and completions.

## Install

```sh
cargo install --path .
```

Prebuilt binaries can be attached to tagged GitHub Releases by the included workflow.

## Examples

```sh
mpct hash sha256 "hello"
mpct hash verify sha256 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824 "hello"
mpct checksum Cargo.toml
mpct checksum verify SHA256SUMS.txt
mpct encode base64 "hello"
mpct decode base64 aGVsbG8=
mpct jwt decode "$TOKEN"
mpct jwt verify "$TOKEN" --secret "$SECRET"
mpct uuid --v7 -n 3
mpct ulid -n 3
mpct nanoid --size 12
mpct pass gen --length 24
mpct pass phrase --words 6
mpct pass strength "correct horse battery staple"
mpct totp new --uri --account alice@example.com
mpct totp gen JBSWY3DPEHPK3PXP
mpct totp verify JBSWY3DPEHPK3PXP 123456
mpct json pretty --file data.json
mpct json query .users[0].email --file data.json
mpct yaml to-json --file config.yaml
mpct yaml from-json --file data.json
mpct toml pretty --file Cargo.toml
mpct toml to-json --file Cargo.toml
mpct regex test '(?P<word>\\w+)' 'hello world'
mpct regex replace '\\s+' '-' 'hello world'
mpct case snake 'Hello world'
mpct text lorem --paragraphs 2
mpct text count --file README.md
mpct text slug 'Hello, MPCT!'
mpct time now --utc
mpct time epoch 2026-06-18T12:00:00Z
mpct time iso 1781784000
mpct time tz '2026-06-18 12:00:00' --from Europe/Warsaw --to UTC
mpct qr gen 'hello' --ascii
mpct qr gen 'hello' --png hello.png
mpct qr read hello.png
mpct color convert '#336699' --to hsl
mpct color palette '#336699' --scheme triadic --count 6
mpct diff old.txt new.txt --unified
mpct crypt encrypt secret.txt -o secret.txt.mpct --passphrase-stdin
mpct crypt decrypt secret.txt.mpct -o secret.txt --passphrase-stdin
mpct net ping example.com -c 2
mpct net port example.com 443
mpct net portscan example.com --ports 80,443
mpct net dns example.com --type MX
mpct net whois example.com
mpct net tls example.com
mpct net headers https://example.com --method HEAD
mpct net myip
mpct net spf example.com
mpct net dkim example.com --selector default
mpct net dmarc example.com
mpct clip set 'hello'
mpct clip get
mpct sysinfo --json
mpct completions zsh
mpct banner
```
