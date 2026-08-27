use std::process::ExitCode;

use miniscript::bitcoin::hashes::{hash160, ripemd160, sha256};
use miniscript::bitcoin::{self, absolute, relative};

use miniscript_diagnostics::{DiagnosticContext, Status};

const USAGE: &str = "\
miniscript-diagnostics — explain Miniscript satisfaction state under a spending context

USAGE:
    miniscript-diagnostics <MINISCRIPT> [OPTIONS]

    Keys are `bitcoin::PublicKey`, parsed under the Segwitv0 script context.
    See README.md for the currently supported fragments.

OPTIONS:
    --key <HEX>                 A public key we can currently sign for.
                                May be given multiple times.
    --sha256-preimage <HASH>    Mark a sha256 hash (hex) as having an available preimage.
    --hash256-preimage <HASH>   Mark a hash256 hash (hex) as having an available preimage.
    --ripemd160-preimage <HASH> Mark a ripemd160 hash (hex) as having an available preimage.
    --hash160-preimage <HASH>   Mark a hash160 hash (hex) as having an available preimage.
    --chain-height <N>          Current block height (for `after`).
    --chain-time <N>            Current median-time-past, unix seconds (for `after`).
    --elapsed-blocks <N>        Blocks elapsed since the spent output's confirmation (for `older`).
    --elapsed-time <N>          512-second intervals elapsed since confirmation (for `older`).
    --json                      Print the diagnostic tree as JSON instead of the human-readable tree.
                                stdout is pure JSON in this mode -- nothing else is printed to it.
    -h, --help                  Print this help.

EXIT CODES:
    0    root status SATISFIED
    1    invalid arguments or invalid Miniscript input
    2    root status UNAVAILABLE
    3    root status IMPOSSIBLE
    4    root status UNSUPPORTED

EXAMPLE:
    miniscript-diagnostics 'and_v(v:pk(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798),older(144))' \\
        --key 0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798 \\
        --elapsed-blocks 83
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    match run(&args) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

/// Maps a root `Status` to the process exit code documented in `USAGE`
/// above. Argument/parse errors (not a `Status` at all) exit 1, handled
/// separately in `main`.
fn exit_code_for(status: Status) -> u8 {
    match status {
        Status::Satisfied => 0,
        Status::Unavailable => 2,
        Status::Impossible => 3,
        Status::Unsupported => 4,
    }
}

fn run(args: &[String]) -> Result<u8, String> {
    let mut miniscript: Option<String> = None;
    let mut ctx: DiagnosticContext<bitcoin::PublicKey> = DiagnosticContext::new();
    let mut json = false;

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        let mut take_value = |flag: &str| -> Result<String, String> {
            index += 1;
            args.get(index)
                .cloned()
                .ok_or_else(|| format!("{flag} requires a value"))
        };

        match arg.as_str() {
            "--key" => {
                let value = take_value("--key")?;
                let pk: bitcoin::PublicKey = value
                    .parse()
                    .map_err(|e| format!("invalid --key {value}: {e}"))?;
                ctx = ctx.with_key(pk);
            }
            "--sha256-preimage" => {
                let value = take_value("--sha256-preimage")?;
                let hash: sha256::Hash = value
                    .parse()
                    .map_err(|e| format!("invalid --sha256-preimage {value}: {e}"))?;
                ctx = ctx.with_sha256_preimage(hash);
            }
            "--hash256-preimage" => {
                let value = take_value("--hash256-preimage")?;
                let hash: miniscript::hash256::Hash = value
                    .parse()
                    .map_err(|e| format!("invalid --hash256-preimage {value}: {e}"))?;
                ctx = ctx.with_hash256_preimage(hash);
            }
            "--ripemd160-preimage" => {
                let value = take_value("--ripemd160-preimage")?;
                let hash: ripemd160::Hash = value
                    .parse()
                    .map_err(|e| format!("invalid --ripemd160-preimage {value}: {e}"))?;
                ctx = ctx.with_ripemd160_preimage(hash);
            }
            "--hash160-preimage" => {
                let value = take_value("--hash160-preimage")?;
                let hash: hash160::Hash = value
                    .parse()
                    .map_err(|e| format!("invalid --hash160-preimage {value}: {e}"))?;
                ctx = ctx.with_hash160_preimage(hash);
            }
            "--chain-height" => {
                let value = take_value("--chain-height")?;
                let height_value: u32 = value
                    .parse()
                    .map_err(|_| format!("invalid --chain-height {value}"))?;
                let height = absolute::Height::from_consensus(height_value)
                    .map_err(|e| format!("invalid --chain-height {value}: {e}"))?;
                ctx = ctx.with_chain_height(height);
            }
            "--chain-time" => {
                let value = take_value("--chain-time")?;
                let timestamp: u32 = value
                    .parse()
                    .map_err(|_| format!("invalid --chain-time {value}"))?;
                let time = absolute::Time::from_consensus(timestamp)
                    .map_err(|e| format!("invalid --chain-time {value}: {e}"))?;
                ctx = ctx.with_chain_time(time);
            }
            "--elapsed-blocks" => {
                let value = take_value("--elapsed-blocks")?;
                let blocks: u16 = value
                    .parse()
                    .map_err(|_| format!("invalid --elapsed-blocks {value}"))?;
                ctx = ctx.with_elapsed_blocks(relative::Height::from(blocks));
            }
            "--elapsed-time" => {
                let value = take_value("--elapsed-time")?;
                let intervals: u16 = value
                    .parse()
                    .map_err(|_| format!("invalid --elapsed-time {value}"))?;
                ctx = ctx.with_elapsed_time(relative::Time::from_512_second_intervals(intervals));
            }
            "--json" => {
                json = true;
            }
            other => {
                if miniscript.is_some() {
                    return Err(format!("unexpected argument: {other}"));
                }
                miniscript = Some(other.to_string());
            }
        }
        index += 1;
    }

    let miniscript = miniscript.ok_or("missing MINISCRIPT argument")?;

    let diagnostic =
        miniscript_diagnostics::parse_and_evaluate::<bitcoin::PublicKey>(&miniscript, &ctx)
            .map_err(|e| e.to_string())?;

    if json {
        let json_output = serde_json::to_string_pretty(&diagnostic)
            .map_err(|e| format!("failed to serialize diagnostic: {e}"))?;
        println!("{json_output}");
    } else {
        print!("{}", diagnostic.render());
        println!("\nROOT STATUS: {}", diagnostic.status);
    }

    Ok(exit_code_for(diagnostic.status))
}
