// SPDX-License-Identifier: AGPL-3.0-only
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "networkd.toml".into());
    let run = || -> anyhow::Result<()> {
        let cfg = wutong_networkd::Config::from_toml(&std::fs::read_to_string(&path)?)?;
        wutong_networkd::ipc::serve(&cfg)
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("wutong-networkd: {e:#}");
            ExitCode::from(1)
        }
    }
}
