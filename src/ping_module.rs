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

use tokio::process::Command;
use regex::Regex;
use anyhow::{Result, anyhow};
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence};
use tokio::time::{self, Duration};

use std::net::IpAddr;
use std::str::FromStr;
use rand::random;

#[derive(Debug)]
pub struct PingResult {
    pub loss: f64,
    pub min: f64,
    pub avg: f64,
    pub max: f64,
    pub mdev: f64,
}

pub async fn run_sys_ping(target: &str) -> Result<PingResult> {
    // ping -qnAw 59 target
    // -q: quiet
    // -n: numeric output
    // -A: adaptive
    // -w 59: deadline 59 seconds
    let output = Command::new("ping")
        .args(&["-qnAw", "59", target])
        .output().await?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse loss
    // Example: "100 packets transmitted, 100 received, 0% packet loss, time 1999ms"
    // Extract "0" from "0% packet loss"
    let re_loss = Regex::new(r"(\d+)% packet loss").unwrap();
    let loss_cap = re_loss.captures(&stdout).ok_or(anyhow!("Could not parse packet loss"))?;
    let loss = loss_cap[1].parse::<f64>()?;

    // Parse RTT
    // Example: "rtt min/avg/max/mdev = 1.000/2.000/3.000/0.500 ms"
    let re_rtt = Regex::new(r"rtt min/avg/max/mdev = ([\d\.]+)/([\d\.]+)/([\d\.]+)/([\d\.]+) ms").unwrap();
    
    // Sometimes ping output might not have RTT stats if all packets are lost
    if let Some(rtt_cap) = re_rtt.captures(&stdout) {
        Ok(PingResult {
            loss,
            min: rtt_cap[1].parse()?,
            avg: rtt_cap[2].parse()?,
            max: rtt_cap[3].parse()?,
            mdev: rtt_cap[4].parse()?,
        })
    } else {
        // Handle 100% loss case or unexpected output
        if loss == 100.0 {
             Ok(PingResult {
                loss,
                min: 0.0,
                avg: 0.0,
                max: 0.0,
                mdev: 0.0,
            })
        } else {
            Err(anyhow!("Could not parse RTT statistics"))
        }
    }
}

pub async fn run_self_ping(target: &str) -> Result<PingResult> {
    // Resolve IP
    let ip = match IpAddr::from_str(target) {
        Ok(addr) => addr,
        Err(_) => {
            // Simple DNS lookup using std::net::ToSocketAddrs (blocking, but okay for now or use tokio defaults)
            // Or assume input is hostname and resolve it
             match tokio::net::lookup_host(format!("{}:0", target)).await?.next() {
                Some(socket_addr) => socket_addr.ip(),
                None => return Err(anyhow!("Could not resolve host")),
            }
        }
    };

    let client = Client::new(&Config::default())?;
    let mut pinger = client.pinger(ip, PingIdentifier(rand::random::<u16>())).await;

    let mut rtts = Vec::new();
    let count = 59; // Approx 59 pings to match 59s duration if 1/sec, or adaptive.
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
