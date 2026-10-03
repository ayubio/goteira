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

use anyhow::{anyhow, Result};
use regex::Regex;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::process::Command;
use tokio::time::{self, Duration};

use std::net::IpAddr;
use std::str::FromStr;
// rand::random is unused

#[derive(Debug)]
pub struct PingResult {
    pub loss: f64,
    pub min: f64,
    pub avg: f64,
    pub max: f64,
    pub mdev: f64,
}

impl PingResult {
    /// Result reported when no reply was received (or ping could not run).
    pub fn lost() -> Self {
        PingResult {
            loss: 100.0,
            min: 0.0,
            avg: 0.0,
            max: 0.0,
            mdev: 0.0,
        }
    }
}

pub async fn run_sys_ping(target: &str) -> Result<PingResult> {
    // ping -qnAw 59 target
    // -q: quiet
    // -n: numeric output
    // -A: adaptive
    // -w 59: deadline 59 seconds
    // LC_ALL=C avoids locale-dependent output breaking the parser.
    let output = Command::new("ping")
        .env("LC_ALL", "C")
        .args(["-qnAw", "59", target])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ping_output(&stdout).map_err(|e| {
        let stderr = String::from_utf8_lossy(&output.stderr);
        match stderr.trim() {
            "" => e,
            msg => anyhow!("{} ({})", e, msg),
        }
    })
}

/// Parses the summary printed by iputils `ping -q`.
fn parse_ping_output(stdout: &str) -> Result<PingResult> {
    // Example: "100 packets transmitted, 100 received, 0% packet loss, time 1999ms"
    // Loss may be fractional on newer iputils: "1.5% packet loss"
    let re_loss = Regex::new(r"([\d.]+)% packet loss").unwrap();
    let loss = re_loss
        .captures(stdout)
        .ok_or_else(|| anyhow!("Could not parse packet loss"))?[1]
        .parse::<f64>()?;

    // Example: "rtt min/avg/max/mdev = 1.000/2.000/3.000/0.500 ms"
    let re_rtt =
        Regex::new(r"rtt min/avg/max/mdev = ([\d.]+)/([\d.]+)/([\d.]+)/([\d.]+) ms").unwrap();

    if let Some(rtt_cap) = re_rtt.captures(stdout) {
        Ok(PingResult {
            loss,
            min: rtt_cap[1].parse()?,
            avg: rtt_cap[2].parse()?,
            max: rtt_cap[3].parse()?,
            mdev: rtt_cap[4].parse()?,
        })
    } else if loss == 100.0 {
        // No RTT line when every packet is lost
        Ok(PingResult::lost())
    } else {
        Err(anyhow!("Could not parse RTT statistics"))
    }
}

pub async fn run_self_ping(target: &str) -> Result<PingResult> {
    // Resolve IP
    let ip = match IpAddr::from_str(target) {
        Ok(addr) => addr,
        Err(_) => {
            // Simple DNS lookup using std::net::ToSocketAddrs (blocking, but okay for now or use tokio defaults)
            // Or assume input is hostname and resolve it
            match tokio::net::lookup_host(format!("{}:0", target))
                .await?
                .next()
            {
                Some(socket_addr) => socket_addr.ip(),
                None => return Err(anyhow!("Could not resolve host")),
            }
        }
    };

    let client = Client::new(&Config::default())?;
    let mut pinger = client
        .pinger(ip, PingIdentifier(rand::random::<u16>()))
        .await;

    let mut rtts = Vec::new();
    let _count = 59; // Approx 59 pings to match 59s duration if 1/sec, or adaptive.
                     // The original script uses -A (adaptive), so it floods.
                     // We will stick to a reasonable interval, e.g., 200ms = 5 pings/sec * 12 sec = 60 pings?
                     // Or just 1 ping per second for 59 seconds?
                     // The user said: "gerador e interpretador do ICMP Ping seja o próprio código fonte Rust"
                     // Let's do 60 pings with 200ms interval (approx 12s total?) or spread over 59s?
                     // The command `ping -w 59` runs for 59 seconds. `ping -A` sends packets as soon as reply is received.
                     // Implementing adaptive ping is complex. Let's do a fast ping: 100ms interval for 60 seconds? That's too many.
                     // Let's do 1 ping per second for 59 seconds to match the deadline duration roughly, OR
                     // match the packet count. The script doesn't set count, just deadline.
                     // Let's aim for 60 samples.

    let interval = Duration::from_millis(500); // 2 pings/sec
    let duration = Duration::from_secs(59);
    let start = time::Instant::now();
    let mut sent = 0;
    let mut received = 0;

    let mut seq: u16 = 0;
    loop {
        if start.elapsed() > duration {
            break;
        }

        match pinger.ping(PingSequence(seq), &PAYLOAD).await {
            Ok((_packet, duration)) => {
                rtts.push(duration.as_secs_f64() * 1000.0); // ms
                received += 1;
            }
            Err(_) => {
                // timeout or error
            }
        }
        sent += 1;
        seq = seq.wrapping_add(1);
        time::sleep(interval).await;
    }

    if sent == 0 {
        return Err(anyhow!("No packets sent"));
    }

    let loss = ((sent - received) as f64 / sent as f64) * 100.0;

    if rtts.is_empty() {
        return Ok(PingResult {
            loss,
            min: 0.0,
            avg: 0.0,
            max: 0.0,
            mdev: 0.0,
        });
    }

    let min = rtts.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = rtts.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let sum: f64 = rtts.iter().sum();
    let avg = sum / rtts.len() as f64;

    // mdev = sqrt(sum((x - avg)^2) / N)
    let variance_sum: f64 = rtts.iter().map(|x| (x - avg).powi(2)).sum();
    let mdev = (variance_sum / rtts.len() as f64).sqrt();

    Ok(PingResult {
        loss,
        min,
        avg,
        max,
        mdev,
    })
}

// Helper payload
const PAYLOAD: [u8; 56] = [0; 56];

#[cfg(test)]
mod tests {
    use super::*;

    const OK: &str = "--- 8.8.8.8 ping statistics ---\n\
        1000 packets transmitted, 1000 received, 0% packet loss, time 59001ms\n\
        rtt min/avg/max/mdev = 3.120/6.201/83.512/3.244 ms, ipg/ewma 59.001/5.900 ms\n";

    #[test]
    fn parses_success() {
        let r = parse_ping_output(OK).unwrap();
        assert_eq!(r.loss, 0.0);
        assert_eq!((r.min, r.avg, r.max, r.mdev), (3.120, 6.201, 83.512, 3.244));
    }

    #[test]
    fn parses_fractional_loss() {
        let out = OK.replace("0% packet loss", "1.5% packet loss");
        assert_eq!(parse_ping_output(&out).unwrap().loss, 1.5);
    }

    #[test]
    fn parses_total_loss_without_rtt() {
        let out = "5 packets transmitted, 0 received, 100% packet loss, time 4090ms\n";
        let r = parse_ping_output(out).unwrap();
        assert_eq!((r.loss, r.avg), (100.0, 0.0));
    }

    #[test]
    fn rejects_unparseable_output() {
        assert!(parse_ping_output("").is_err());
        let partial = "10 packets transmitted, 9 received, 10% packet loss, time 9ms\n";
        assert!(parse_ping_output(partial).is_err());
    }
}
