<div align="center">
  <img src="https://github.com/user-attachments/assets/90b0864e-f528-401e-a165-f039b7d2ae78" alt="Goteira Logo" width="200">
</div>

# Goteira Manual

This document provides complete instructions for installing and using both versions of the software: **Shell Script** (`goteira.sh`) and **Rust** (`goteira`). Gemmini was responsible for the Rust version derived from the original Bash script. The goal is to provide a standalone version, since the bash script requires system dependencies.

Both versions perform connectivity tests (ping) and can optionally execute a traceroute (mtr) for network diagnostics, generating timestamped reports.

Each ICMP Ping test is performed for 59 seconds in a row. The goal is to capture any link oscillations or variations in latency. If you need a high precision report, testing every minute is recommended.

You must create the `/var/log/goteira` directory before running the script and ensure it has write permissions.

Originally, this software was named "sergioreis.sh" in honor of the Brazilian singer and songwriter Sérgio Reis and his 1985 song "Pinga Ni Mim". Upon releasing the source code publicly, as I did not have the artist's authorization to use his name, I chose to rename it to "goteira", which means "drip" or "a leak in the ceiling" in Portuguese.

---

## 1. Shell Script Version (`goteira.sh`) — DEPRECATED

> See [MIGRATION.md](MIGRATION.md) to move to the Rust version.

> **Deprecated.** The Rust version is now the only maintained one and is a drop-in replacement: same command line (`-m`), same TAB-separated output. The shell script will be removed in 1.0.0.

The original Bash version, lightweight and with common Linux system dependencies.

### Prerequisites

Ensure you have the following tools installed on your system:

- `bash` (or compatible `sh`)
- `ping` (iputils-ping)
- `mtr` (for traceroute functionality)
- `coreutils` (date, mktemp, rm, mv, mkdir, etc.)

On Debian/Ubuntu based systems, you can install the necessary tools with:
```bash
sudo apt update
sudo apt install iputils-ping mtr-tiny coreutils
```

### Installation

1.  Download the `goteira.sh` script.
2.  Grant execution permission to the file:
    ```bash
    chmod +x goteira.sh
    ```
3.  (Optional) Move it to a directory in your PATH to execute it from anywhere:
    ```bash
    sudo mv goteira.sh /opt/goteira/goteira.sh
    ```

### Usage

The basic syntax is:

```bash
/opt/goteira/goteira.sh [-m] <TARGET>
```

- **`<TARGET>`**: The IP address or hostname you want to test (e.g., `8.8.8.8`, `google.com`).
- **`-m`**: (Optional) Enables traceroute (`mtr`) execution in parallel to ping. If omitted, only ping will be executed.

#### Examples

**Ping Only (Default):**
```bash
./goteira.sh 8.8.8.8
```
*Output: Displays latency and packet loss statistics in the terminal.*

**Ping with Traceroute (MTR):**
```bash
./goteira.sh -m 8.8.8.8
```
*Output: Displays ping statistics in the terminal and, in the background, saves a detailed MTR report in `/var/log/goteira/...`.*

---

## 2. Rust Version (`goteira`)

The modern version rewritten in Rust, featuring better performance and structure.

### Pre-built binary (recommended)

Static binaries (no Rust toolchain needed) for `x86_64` and `aarch64` are attached to each [GitHub Release](https://github.com/ayubio/goteira/releases), with a `.sha256` checksum:

```bash
VER=v0.4.0; ARCH=$(uname -m)   # x86_64 or aarch64
curl -LO https://github.com/ayubio/goteira/releases/download/$VER/goteira-$VER-linux-$ARCH.tar.gz
curl -LO https://github.com/ayubio/goteira/releases/download/$VER/goteira-$VER-linux-$ARCH.tar.gz.sha256
sha256sum -c goteira-$VER-linux-$ARCH.tar.gz.sha256
tar xzf goteira-$VER-linux-$ARCH.tar.gz
sudo install -m 755 goteira-$VER-linux-$ARCH/goteira /opt/goteira/goteira
```

The default mode only needs the system `ping` (`iputils-ping`), plus `mtr` if you use `-m`.

**Privileges for the experimental internal features** (not needed by default):
- `--selfping` uses unprivileged ICMP sockets: your group must be inside `net.ipv4.ping_group_range` (`cat /proc/sys/net/ipv4/ping_group_range`; most distros already allow all groups).
- `--selftraceroute` uses raw sockets and needs `CAP_NET_RAW`: `sudo setcap cap_net_raw+ep /opt/goteira/goteira`.

### Building from source

### Prerequisites

To compile and run this version, you need the Rust development environment installed.

- **Rust and Cargo**: Install via [rustup.rs](https://rustup.rs/):
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

### Installation / Compilation

1.  Navigate to the project directory:
    ```bash
    cd /path/to/goteira
    ```
2.  Compile the project in release mode for optimization:
    ```bash
    cargo build --release
    ```
3.  The binary will be generated at `./target/release/goteira`.

### Usage

You can run it directly via `cargo` or execute the compiled binary.

#### Syntax

```bash
cargo run --release -- [OPTIONS] <TARGET>
# or
./target/release/goteira [OPTIONS] <TARGET>
```

#### Available Options

- **`<TARGET>`**: The IP address or hostname (Required).
- **`-m`, `--mtr`** (alias `--sysmtr`): Also runs the system's `mtr` in parallel with ping and saves its report. Requires `mtr` installed.
- **`--selfping`**: *(experimental)* Uses the internal Rust ping instead of the system `ping`.
- **`--selftraceroute`**: *(experimental)* Uses the internal Rust traceroute (needs `CAP_NET_RAW`).
- **`-h`, `--help`**: Displays help information.

**Note:** The system `ping` (iputils) is required and is the default. If no traceroute option (`-m` or `--selftraceroute`) is provided, only ping will be executed.

**Exit code:** `0` on success; `1` when ping fails or all packets are lost (the output line is still printed, as `100.0%`, so logs have no gaps; the cause goes to stderr).

#### Examples

**Ping only:**
```bash
./target/release/goteira 8.8.8.8
```

**Ping + MTR (same as `goteira.sh -m`):**
```bash
./target/release/goteira -m 8.8.8.8
```

**Internal traceroute (experimental):**
```bash
./target/release/goteira --selftraceroute 8.8.8.8
```

### Logs and Reports

Just like the Shell version, the Rust version saves traceroute reports (when enabled) in:
`/var/log/goteira/YEAR/MONTH/DAY/HOUR/MINUTE/<TARGET>.txt`

---

## 3. Automation with Crontab

For continuous monitoring, you can schedule Goteira execution via `crontab`.

### Configuration Example

To run the script every 5 minutes, collecting mtr and saving the general log to a file:

1.  Edit your crontab:
    ```bash
    crontab -e
    ```
2.  Add the line (adjust paths according to your installation):
    ```cron
    */5 * * * * /opt/goteira/goteira -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
    ```

This will:
- Execute `goteira.sh` every 5 minutes.
- Perform ping and traceroute (`-m`).
- Save standard output (ping stats) to `/var/log/goteira/goteira.log`.
- Detailed MTR reports will continue to be saved in the date/time directory structure.

## 4. Output sample

```
ayubio@baostar:~/software/goteira$ while true; do ./goteira.sh 8.8.8.8; sleep 60; done
[14/02/26-18:24]	0.0%	3.1/6.2/83.5/3.2	8.8.8.8
[14/02/26-18:26]	0.0%	3.1/5.7/28.6/1.5	8.8.8.8
[14/02/26-18:28]	0.0%	3.1/7.3/228.3/9.2	8.8.8.8
[14/02/26-18:30]	0.0%	3.1/6.8/201.6/8.1	8.8.8.8
```

First column is the timestamp, second column is the packet loss percentage (loss%), third column min/avg/max/jitter (as ping -q would show), and the last column is the target IP address for grepping.

---

## 5. Installation (Snap)

```bash
sudo snap install goteira
sudo snap connect goteira:network-observe
```

> The former `goteira-rust` snap is superseded by `goteira`. To migrate: `sudo snap remove goteira-rust && sudo snap install goteira`. Old reports stay in `/var/snap/goteira-rust/common` (the new snap uses `/var/snap/goteira/common`).

### Logs and Reports (Snap)

When installed via Snap, reports are written to the standard Snap writable directory, detected through `$SNAP_COMMON`:

- **Reports Path**: `/var/snap/goteira/common/YEAR/MONTH/DAY/...`

For manual installations, the path remains `/var/log/goteira`.

---

## 6. License

This project is licensed under the **GNU General Public License v3.0 or later**. See the [LICENSE](LICENSE) file for details.
