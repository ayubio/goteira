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
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod ping_module;
mod traceroute_module;

use clap::Parser;
use ping_module::{run_sys_ping, run_self_ping};
use traceroute_module::{run_sys_mtr, run_self_traceroute};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host or IP address
    #[arg(index = 1)]
    target: String,

    /// Use system ping (/bin/ping) instead of internal implementation
    #[arg(long)]
    sysping: bool,

    /// Use system mtr (/usr/sbin/mtr)
    #[arg(long)]
    sysmtr: bool,

    /// Use internal traceroute implementation
    #[arg(long)]
    selftraceroute: bool,
}

use chrono::Local;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Timestamp for report
    let now = Local::now();
    let timestamp_str = now.format("%d/%m/%y-%H:%M").to_string();
    
    // Determine Report Root Path
    let report_root = std::env::var("SNAP_COMMON").unwrap_or_else(|_| "/var/log/goteira".to_string());
    let report_root_cleanup = report_root.clone();

    // Cleanup old logs in background
    tokio::spawn(async move {
        if let Err(e) = clean_old_logs(&report_root_cleanup).await {
            eprintln!("Failed to clean logs: {}", e);
        }
    });

    // Run Ping and Traceroute concurrently
    let target = args.target.clone();
    let target_clone = target.clone();
    
    let ping_handle = tokio::spawn(async move {
        if args.sysping {
            run_sys_ping(&target).await
        } else {
            run_self_ping(&target).await
        }
    });

    let traceroute_handle = tokio::spawn(async move {
        if args.sysmtr {
            Some(run_sys_mtr(&target_clone).await)
        } else if args.selftraceroute {
            Some(run_self_traceroute(&target_clone).await)
        } else {
            None
        }
    });

    // Wait for Ping
    let ping_result = ping_handle.await??;
    
    // Format Ping Output
    // [TIMESTAMP] LOSS% MIN/AVG/MAX/MDEV TARGET
    println!("[{}] {:.0}% {}/{}/{}/{} {}", 
        timestamp_str, 
        ping_result.loss, 
        ping_result.min, ping_result.avg, ping_result.max, ping_result.mdev,
        args.target
    );

    // Wait for Traceroute and Write Report
    if let Some(traceroute_result_res) = traceroute_handle.await? {
        match traceroute_result_res {
            Ok(report) => {
                // Determine Report Path
                // {REPORT_ROOT}/YYYY/MM/DD/HH/MM/target.txt
                let report_dir = format!("{}/{}/{}/{}/{}/{}",
                    report_root,
                    now.format("%Y"), now.format("%m"), now.format("%d"), now.format("%H"), now.format("%M"));
                
                let report_path = Path::new(&report_dir).join(format!("{}.txt", args.target));
                
                if let Err(e) = fs::create_dir_all(&report_dir) {
                    eprintln!("Failed to create report directory: {}", e);
                } else {
                    if let Err(e) = fs::write(&report_path, report) {
                        eprintln!("Failed to write report to {:?}: {}", report_path, e);
                    }
                }
            },
            Err(e) => eprintln!("Traceroute failed: {}", e),
        }
    }

    Ok(())
}

use tokio::process::Command;

async fn clean_old_logs(root: &str) -> Result<()> {
    // find root -type f -mtime +30 -delete
    Command::new("find")
        .args(&[root, "-type", "f", "-mtime", "+30", "-delete"])
        .spawn()
        .context("Failed to spawn find command")?
        .wait().await?;
    Ok(())
}
