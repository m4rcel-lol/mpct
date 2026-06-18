use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "mpct",
    about = "A single-binary multi-purpose command-line toolkit.",
    version = concat!(env!("CARGO_PKG_VERSION"), "\nmpct — one binary. every tool."),
    propagate_version = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Args)]
pub struct GlobalOptions {
    /// Emit machine-readable JSON where the command supports it.
    #[arg(long, global = true)]
    pub json: bool,

    /// Force-disable ANSI color output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Print only the core result.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Print extra diagnostics to stderr.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress the startup banner.
    #[arg(long, global = true)]
    pub no_banner: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Compute or verify text and file digests.
    Hash(HashArgs),
    /// Produce or verify sha256sum-style file manifests.
    Checksum(ChecksumArgs),
    /// Encode text, stdin, or a file.
    Encode(CodecArgs),
    /// Decode text, stdin, or a file.
    Decode(CodecArgs),
    /// Decode or verify JSON Web Tokens.
    #[command(subcommand)]
    Jwt(JwtCommand),
    /// Generate UUID values.
    Uuid(UuidArgs),
    /// Generate ULID values.
    Ulid(CountArgs),
    /// Generate Nano ID values.
    Nanoid(NanoidArgs),
    /// Generate and evaluate passwords.
    #[command(subcommand)]
    Pass(PassCommand),
    /// Create and verify TOTP codes.
    #[command(subcommand)]
    Totp(TotpCommand),
    /// Work with JSON documents.
    #[command(subcommand)]
    Json(JsonCommand),
    /// Work with YAML documents.
    #[command(subcommand)]
    Yaml(YamlCommand),
    /// Work with TOML documents.
    #[command(subcommand)]
    Toml(TomlCommand),
    /// Test or replace regular expressions.
    #[command(subcommand)]
    Regex(RegexCommand),
    /// Convert text case.
    Case(CaseArgs),
    /// Generate and inspect text.
    #[command(subcommand)]
    Text(TextCommand),
    /// Convert and display times.
    #[command(subcommand)]
    Time(TimeCommand),
    /// Generate or read QR codes.
    #[command(subcommand)]
    Qr(QrCommand),
    /// Convert colors and build palettes.
    #[command(subcommand)]
    Color(ColorCommand),
    /// Show differences between two files.
    Diff(DiffArgs),
    /// Encrypt or decrypt files symmetrically.
    #[command(subcommand)]
    Crypt(CryptCommand),
    /// Network diagnostics and record checks.
    #[command(subcommand)]
    Net(NetCommand),
    /// Read and write the system clipboard.
    #[command(subcommand)]
    Clip(ClipCommand),
    /// Print system information.
    Sysinfo,
    /// Generate shell completion scripts.
    Completions(CompletionArgs),
    /// Print the mpct banner.
    Banner,
}

#[derive(Debug, Args)]
pub struct HashArgs {
    /// Hash form: `<algo> [TEXT]` or `verify <algo> <expected_digest> [TEXT]`.
    #[arg(required = true, num_args = 1..=4)]
    pub args: Vec<String>,

    /// Read input bytes from a file instead of TEXT or stdin.
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum HashAlgo {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake3,
}

#[derive(Debug, Args)]
pub struct ChecksumArgs {
    /// Checksum form: `[FILE]...` or `verify <manifest_file>`.
    #[arg(required = true)]
    pub args: Vec<PathBuf>,

    /// Digest algorithm for manifest generation.
    #[arg(long, value_enum, default_value = "sha256")]
    pub algo: ChecksumAlgo,

    /// Recurse into directories.
    #[arg(long)]
    pub recursive: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ChecksumAlgo {
    Sha256,
}

#[derive(Debug, Args)]
pub struct CodecArgs {
    /// Codec to use.
    #[arg(value_enum)]
    pub codec: CodecKind,

    /// Text to read. If omitted and --file is absent, stdin is read.
    pub text: Option<String>,

    /// Read input from a file.
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Use URL-safe base64 alphabet.
    #[arg(long)]
    pub url_safe: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CodecKind {
    Base64,
    Hex,
    Url,
    Base32,
}

#[derive(Debug, Subcommand)]
pub enum JwtCommand {
    /// Decode JWT header and payload without verifying the signature.
    Decode(JwtDecodeArgs),
    /// Verify an HMAC-signed JWT and print claims.
    Verify(JwtVerifyArgs),
}

#[derive(Debug, Args)]
pub struct JwtDecodeArgs {
    /// JWT string.
    pub token: String,
}

#[derive(Debug, Args)]
pub struct JwtVerifyArgs {
    /// JWT string.
    pub token: String,

    /// HMAC secret.
    #[arg(long)]
    pub secret: String,

    /// Expected HMAC algorithm.
    #[arg(long, default_value = "HS256")]
    pub alg: String,
}

#[derive(Debug, Args)]
pub struct UuidArgs {
    /// Generate UUIDv4 values.
    #[arg(long, conflicts_with = "v7")]
    pub v4: bool,

    /// Generate UUIDv7 values.
    #[arg(long)]
    pub v7: bool,

    /// Number of IDs to generate.
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: usize,

    /// Print IDs in uppercase.
    #[arg(long)]
    pub upper: bool,
}

#[derive(Debug, Args)]
pub struct CountArgs {
    /// Number of values to generate.
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: usize,
}

#[derive(Debug, Args)]
pub struct NanoidArgs {
    /// Number of IDs to generate.
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: usize,

    /// Length of each ID.
    #[arg(long, default_value_t = 21)]
    pub size: usize,

    /// Alphabet to sample from.
    #[arg(long)]
    pub alphabet: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum PassCommand {
    /// Generate random passwords.
    Gen(PassGenArgs),
    /// Generate passphrases from an embedded wordlist.
    Phrase(PassPhraseArgs),
    /// Estimate password strength.
    Strength(PassStrengthArgs),
}

#[derive(Debug, Args)]
pub struct PassGenArgs {
    /// Password length.
    #[arg(long, default_value_t = 20)]
    pub length: usize,

    /// Exclude symbols.
    #[arg(long)]
    pub no_symbols: bool,

    /// Exclude digits.
    #[arg(long)]
    pub no_digits: bool,

    /// Number of passwords to generate.
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: usize,
}

#[derive(Debug, Args)]
pub struct PassPhraseArgs {
    /// Number of words per phrase.
    #[arg(long, default_value_t = 6)]
    pub words: usize,

    /// Separator between words.
    #[arg(long, default_value = "-")]
    pub separator: String,

    /// Number of phrases to generate.
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: usize,
}

#[derive(Debug, Args)]
pub struct PassStrengthArgs {
    /// Password to score.
    pub password: String,
}

#[derive(Debug, Subcommand)]
pub enum TotpCommand {
    /// Generate a fresh base32 TOTP secret.
    New(TotpNewArgs),
    /// Generate the current TOTP code.
    Gen(TotpGenArgs),
    /// Verify a TOTP code.
    Verify(TotpVerifyArgs),
}

#[derive(Debug, Args)]
pub struct TotpNewArgs {
    /// Also print an otpauth provisioning URI.
    #[arg(long)]
    pub uri: bool,

    /// Account name for the provisioning URI.
    #[arg(long, default_value = "user")]
    pub account: String,

    /// Issuer for the provisioning URI.
    #[arg(long, default_value = "mpct")]
    pub issuer: String,
}

#[derive(Debug, Args)]
pub struct TotpGenArgs {
    /// Base32 TOTP secret.
    pub secret: String,

    /// Number of output digits.
    #[arg(long, default_value_t = 6)]
    pub digits: usize,

    /// TOTP period in seconds.
    #[arg(long, default_value_t = 30)]
    pub period: u64,
}

#[derive(Debug, Args)]
pub struct TotpVerifyArgs {
    /// Base32 TOTP secret.
    pub secret: String,

    /// Code to verify.
    pub code: String,

    /// Number of adjacent periods accepted on each side.
    #[arg(long, default_value_t = 1)]
    pub window: u8,
}

#[derive(Debug, Subcommand)]
pub enum JsonCommand {
    /// Pretty-print JSON.
    Pretty(FormatArgs),
    /// Minify JSON.
    Minify(FileOnlyArgs),
    /// Validate JSON syntax.
    Validate(FileOnlyArgs),
    /// Query JSON with a deterministic dotted path.
    Query(JsonQueryArgs),
}

#[derive(Debug, Subcommand)]
pub enum YamlCommand {
    /// Pretty-print YAML.
    Pretty(FormatArgs),
    /// Minify YAML.
    Minify(FileOnlyArgs),
    /// Validate YAML syntax.
    Validate(FileOnlyArgs),
    /// Convert YAML to JSON.
    ToJson(FileOnlyArgs),
    /// Convert JSON to YAML.
    FromJson(FormatArgs),
}

#[derive(Debug, Subcommand)]
pub enum TomlCommand {
    /// Pretty-print TOML.
    Pretty(FileOnlyArgs),
    /// Validate TOML syntax.
    Validate(FileOnlyArgs),
    /// Convert TOML to JSON.
    ToJson(FileOnlyArgs),
    /// Convert JSON to TOML.
    FromJson(FileOnlyArgs),
}

#[derive(Debug, Args)]
pub struct FormatArgs {
    /// Read input from a file. If omitted, stdin is read.
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Indentation width for formats that support it.
    #[arg(long, default_value_t = 2)]
    pub indent: usize,
}

#[derive(Debug, Args)]
pub struct FileOnlyArgs {
    /// Read input from a file. If omitted, stdin is read.
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct JsonQueryArgs {
    /// Path expression such as `.users[0].email`.
    pub path_expr: String,

    /// Read input from a file. If omitted, stdin is read.
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum RegexCommand {
    /// Test a regular expression.
    Test(RegexTestArgs),
    /// Replace regular expression matches.
    Replace(RegexReplaceArgs),
}

#[derive(Debug, Args)]
pub struct RegexTestArgs {
    /// Regular expression pattern.
    pub pattern: String,

    /// Text to test. Optional when --file is supplied.
    pub text: Option<String>,

    /// Read tested text from a file.
    #[arg(long)]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct RegexReplaceArgs {
    /// Regular expression pattern.
    pub pattern: String,

    /// Replacement string.
    pub replacement: String,

    /// Input text.
    pub text: String,
}

#[derive(Debug, Args)]
pub struct CaseArgs {
    /// Target case style.
    #[arg(value_enum)]
    pub style: CaseStyle,

    /// Text to convert.
    pub text: String,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CaseStyle {
    Camel,
    Pascal,
    Snake,
    Kebab,
    #[value(name = "screaming-snake")]
    ScreamingSnake,
    Title,
    Sentence,
}

#[derive(Debug, Subcommand)]
pub enum TextCommand {
    /// Generate lorem ipsum text.
    Lorem(TextLoremArgs),
    /// Count words, lines, chars, and bytes.
    Count(FileOnlyArgs),
    /// Slugify text.
    Slug(TextSlugArgs),
}

#[derive(Debug, Args)]
pub struct TextLoremArgs {
    /// Number of paragraphs to generate.
    #[arg(long, default_value_t = 3)]
    pub paragraphs: usize,

    /// Approximate words per paragraph.
    #[arg(long, default_value_t = 40)]
    pub words_per_paragraph: usize,
}

#[derive(Debug, Args)]
pub struct TextSlugArgs {
    /// Text to slugify.
    pub text: String,
}

#[derive(Debug, Subcommand)]
pub enum TimeCommand {
    /// Print the current time.
    Now(TimeNowArgs),
    /// Convert an ISO 8601 or simple human time to a Unix epoch.
    Epoch(TimeEpochArgs),
    /// Convert a Unix epoch to ISO 8601.
    Iso(TimeIsoArgs),
    /// Convert a time from one timezone to another.
    Tz(TimeTzArgs),
}

#[derive(Debug, Args)]
pub struct TimeNowArgs {
    /// Print UTC time.
    #[arg(long, conflicts_with = "local")]
    pub utc: bool,

    /// Print local time.
    #[arg(long)]
    pub local: bool,

    /// strftime-compatible output format.
    #[arg(long)]
    pub format: Option<String>,
}

#[derive(Debug, Args)]
pub struct TimeEpochArgs {
    /// ISO 8601, `now`, `YYYY-MM-DD`, or `YYYY-MM-DD HH:MM:SS`.
    pub time: String,
}

#[derive(Debug, Args)]
pub struct TimeIsoArgs {
    /// Unix epoch seconds.
    pub epoch: i64,
}

#[derive(Debug, Args)]
pub struct TimeTzArgs {
    /// Time value to convert.
    pub time: String,

    /// Source timezone, for example `UTC` or `Europe/Warsaw`.
    #[arg(long)]
    pub from: String,

    /// Target timezone, for example `America/New_York`.
    #[arg(long)]
    pub to: String,
}

#[derive(Debug, Subcommand)]
pub enum QrCommand {
    /// Generate a QR code.
    Gen(QrGenArgs),
    /// Read a QR code from an image.
    Read(QrReadArgs),
}

#[derive(Debug, Args)]
pub struct QrGenArgs {
    /// Text to encode.
    pub text: String,

    /// Print a terminal QR code.
    #[arg(long, conflicts_with_all = ["png", "svg"])]
    pub ascii: bool,

    /// Write a PNG image.
    #[arg(long, conflicts_with = "svg")]
    pub png: Option<PathBuf>,

    /// Write an SVG image.
    #[arg(long)]
    pub svg: Option<PathBuf>,

    /// Error correction level.
    #[arg(long, value_enum, default_value = "m")]
    pub ecc: QrEcc,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum QrEcc {
    L,
    M,
    Q,
    H,
}

#[derive(Debug, Args)]
pub struct QrReadArgs {
    /// Image path to decode.
    pub image_path: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum ColorCommand {
    /// Convert a color to another format.
    Convert(ColorConvertArgs),
    /// Generate a related color palette.
    Palette(ColorPaletteArgs),
}

#[derive(Debug, Args)]
pub struct ColorConvertArgs {
    /// Color value in hex, rgb(), hsl(), or cmyk() form.
    pub value: String,

    /// Target color format.
    #[arg(long, value_enum)]
    pub to: ColorFormat,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ColorFormat {
    Hex,
    Rgb,
    Hsl,
    Cmyk,
}

#[derive(Debug, Args)]
pub struct ColorPaletteArgs {
    /// Base color in hex form.
    pub base_hex: String,

    /// Palette scheme.
    #[arg(long, value_enum, default_value = "complementary")]
    pub scheme: PaletteScheme,

    /// Number of colors to print.
    #[arg(long, default_value_t = 5)]
    pub count: usize,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum PaletteScheme {
    Complementary,
    Analogous,
    Triadic,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// First file.
    pub file_a: PathBuf,

    /// Second file.
    pub file_b: PathBuf,

    /// Print unified diff.
    #[arg(long, conflicts_with = "side_by_side")]
    pub unified: bool,

    /// Print side-by-side diff.
    #[arg(long)]
    pub side_by_side: bool,
}

#[derive(Debug, Subcommand)]
pub enum CryptCommand {
    /// Encrypt a file.
    Encrypt(CryptArgs),
    /// Decrypt a file.
    Decrypt(CryptArgs),
}

#[derive(Debug, Args)]
pub struct CryptArgs {
    /// Input file.
    pub file: PathBuf,

    /// Output file path.
    #[arg(short = 'o', long)]
    pub out: Option<PathBuf>,

    /// Read passphrase from stdin instead of a hidden prompt.
    #[arg(long)]
    pub passphrase_stdin: bool,
}

#[derive(Debug, Subcommand)]
pub enum NetCommand {
    /// Ping a host with the OS ping binary.
    Ping(NetPingArgs),
    /// Check a single TCP port.
    Port(NetPortArgs),
    /// Scan a TCP port range.
    Portscan(NetPortscanArgs),
    /// Resolve DNS records.
    Dns(NetDnsArgs),
    /// Query WHOIS servers.
    Whois(NetWhoisArgs),
    /// Inspect a TLS certificate.
    Tls(NetTlsArgs),
    /// Fetch HTTP response headers.
    Headers(NetHeadersArgs),
    /// Print the public IP address.
    Myip(NetMyIpArgs),
    /// Fetch SPF TXT records.
    Spf(NetDomainArgs),
    /// Fetch a DKIM TXT record.
    Dkim(NetDkimArgs),
    /// Fetch DMARC TXT records.
    Dmarc(NetDomainArgs),
}

#[derive(Debug, Args)]
pub struct NetPingArgs {
    /// Hostname or address.
    pub host: String,

    /// Number of echo requests.
    #[arg(short = 'c', long, default_value_t = 4)]
    pub count: u16,

    /// Timeout in milliseconds.
    #[arg(long, default_value_t = 1000)]
    pub timeout: u64,
}

#[derive(Debug, Args)]
pub struct NetPortArgs {
    /// Hostname or address.
    pub host: String,

    /// TCP port.
    pub port: u16,

    /// Timeout in milliseconds.
    #[arg(long, default_value_t = 1000)]
    pub timeout: u64,
}

#[derive(Debug, Args)]
pub struct NetPortscanArgs {
    /// Hostname or address.
    pub host: String,

    /// Port expression such as `1-1024` or `22,80,443`.
    #[arg(long, default_value = "1-1024")]
    pub ports: String,

    /// Per-port timeout in milliseconds.
    #[arg(long, default_value_t = 500)]
    pub timeout: u64,

    /// Maximum worker threads.
    #[arg(long, default_value_t = 100)]
    pub concurrency: usize,
}

#[derive(Debug, Args)]
pub struct NetDnsArgs {
    /// Domain name.
    pub domain: String,

    /// Record type.
    #[arg(long = "type", value_enum, default_value = "a")]
    pub record_type: DnsRecordType,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DnsRecordType {
    A,
    Aaaa,
    Mx,
    Txt,
    Ns,
    Cname,
    All,
}

#[derive(Debug, Args)]
pub struct NetWhoisArgs {
    /// Domain name.
    pub domain: String,
}

#[derive(Debug, Args)]
pub struct NetTlsArgs {
    /// Host with optional `:port`; defaults to 443.
    pub host: String,
}

#[derive(Debug, Args)]
pub struct NetHeadersArgs {
    /// URL to request.
    pub url: String,

    /// HTTP method.
    #[arg(long, value_enum, default_value = "head")]
    pub method: HttpMethod,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
}

#[derive(Debug, Args)]
pub struct NetMyIpArgs {
    /// Force IPv4 lookup.
    #[arg(long, conflicts_with = "v6")]
    pub v4: bool,

    /// Force IPv6 lookup.
    #[arg(long)]
    pub v6: bool,
}

#[derive(Debug, Args)]
pub struct NetDomainArgs {
    /// Domain name.
    pub domain: String,
}

#[derive(Debug, Args)]
pub struct NetDkimArgs {
    /// Domain name.
    pub domain: String,

    /// DKIM selector.
    #[arg(long)]
    pub selector: String,
}

#[derive(Debug, Subcommand)]
pub enum ClipCommand {
    /// Get clipboard text.
    Get,
    /// Set clipboard text.
    Set(ClipSetArgs),
}

#[derive(Debug, Args)]
pub struct ClipSetArgs {
    /// Text to set. If omitted, stdin is read.
    pub text: Option<String>,
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    /// Target shell.
    #[arg(value_enum)]
    pub shell: CompletionShell,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}
