// Goteira - Connectivity tester and network diagnostics tool.
// Copyright (C) 2026 Ayub <dev@ayub.net.br>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License

mod ping_module;
mod traceroute_module;

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use ping_module::{run_self_ping, run_sys_ping, PingResult};
use std::fs;
use std::path::Path;
use tokio::process::Command;
use traceroute_module::{run_self_traceroute, run_sys_mtr};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host or IP address
    #[arg(index = 1)]
    target: String,

    /// Also run mtr (system binary) in parallel and save its report
    #[arg(short = 'm', long = "mtr", visible_alias = "sysmtr")]
    mtr: bool,

    /// [experimental] Use the internal traceroute implementation (needs CAP_NET_RAW)
    #[arg(long, conflicts_with = "mtr")]
    selftraceroute: bool,

    /// [experimental] Use the internal ping implementation instead of system ping
    #[arg(long)]
    selfping: bool,

    /// Deprecated: system ping is now the default
    #[arg(long, hide = true)]
    sysping: bool,
}

/// Formats the stdout line consumed by legacy parsers (fields separated by TAB):
/// [DD/MM/YY-HH:MM] LOSS% MIN/AVG/MAX/MDEV TARGET
fn format_ping_line(timestamp: &str, r: &PingResult, target: &str) -> String {
    format!(
        "[{}]\t{:.1}%\t{:.1}/{:.1}/{:.1}/{:.1}\t{}",
        timestamp, r.loss, r.min, r.avg, r.max, r.mdev, target
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Timestamp for report
    let now = Local::now();
    let timestamp_str = now.format("%d/%m/%y-%H:%M").to_string();

    // Determine Report Root Path
    let report_root =
        std::env::var("SNAP_COMMON").unwrap_or_else(|_| "/var/log/goteira".to_string());
    let report_root_cleanup = report_root.clone();

    // Cleanup old logs in background
    let cleanup_handle = tokio::spawn(async move {
        if let Err(e) = clean_old_logs(&report_root_cleanup).await {
            eprintln!("Failed to clean logs: {}", e);
        }
    });

    // Run Ping and Traceroute concurrently
    let target = args.target.clone();
    let target_clone = target.clone();
    let selfping = args.selfping;

    let ping_handle = tokio::spawn(async move {
        if selfping {
            run_self_ping(&target).await
        } else {
            run_sys_ping(&target).await
        }
    });

    let (mtr, selftraceroute) = (args.mtr, args.selftraceroute);
    let traceroute_handle = tokio::spawn(async move {
        if mtr {
            Some(run_sys_mtr(&target_clone).await)
        } else if selftraceroute {
            Some(run_self_traceroute(&target_clone).await)
        } else {
            None
        }
    });

    // Wait for Ping. A failure must never leave a gap in the log: print a
    // 100% loss line, report the cause on stderr and exit non-zero.
    let ping_result = match ping_handle.await? {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Ping failed: {:#}", e);
            PingResult::lost()
        }
    };
    let ping_failed = ping_result.loss >= 100.0;

    println!(
        "{}",
        format_ping_line(&timestamp_str, &ping_result, &args.target)
    );

    // Wait for Traceroute and Write Report (also when ping failed: that is
    // exactly when the route report is most useful)
    if let Some(traceroute_result_res) = traceroute_handle.await? {
        match traceroute_result_res {
            Ok(report) => {
                // {REPORT_ROOT}/YYYY/MM/DD/HH/MM/target.txt
                let report_dir = format!(
                    "{}/{}/{}/{}/{}/{}",
                    report_root,
                    now.format("%Y"),
                    now.format("%m"),
                    now.format("%d"),
                    now.format("%H"),
                    now.format("%M")
                );

                let report_path = Path::new(&report_dir).join(format!("{}.txt", args.target));

                if let Err(e) = fs::create_dir_all(&report_dir) {
                    eprintln!("Failed to create report directory: {}", e);
                } else if let Err(e) = fs::write(&report_path, report) {
                    eprintln!("Failed to write report to {:?}: {}", report_path, e);
                }
            }
            Err(e) => eprintln!("Traceroute failed: {:#}", e),
        }
    }

    let _ = cleanup_handle.await;

    // Exit code: 0 = ok, 1 = ping failed/100% loss
    if ping_failed {
        std::process::exit(1);
    }
    Ok(())
}

async fn clean_old_logs(root: &str) -> Result<()> {
    // find root -type f -mtime +30 -delete
    Command::new("find")
        .args([root, "-type", "f", "-mtime", "+30", "-delete"])
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn find command")?
        .wait()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_line_is_tab_separated() {
        let r = PingResult {
            loss: 0.0,
            min: 3.12,
            avg: 6.2,
            max: 83.5,
            mdev: 3.24,
        };
        assert_eq!(
            format_ping_line("14/02/26-18:24", &r, "8.8.8.8"),
            "[14/02/26-18:24]\t0.0%\t3.1/6.2/83.5/3.2\t8.8.8.8"
        );
    }

    #[test]
    fn lost_result_formats_like_shell_fallback() {
        assert_eq!(
            format_ping_line("01/01/26-00:00", &PingResult::lost(), "x"),
            "[01/01/26-00:00]\t100.0%\t0.0/0.0/0.0/0.0\tx"
        );
    }
}
