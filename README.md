<div align="center">
  <img src="assets/logo.gif" alt="Goteira Logo" width="200">
</div>

# Goteira Manual

This document provides complete instructions for installing and using both versions of the software: **Shell Script** (`goteira.sh`) and **Rust** (`goteira`). Gemmini was responsible for the Rust version derived from the original Bash script. The goal is to provide a standalone version, since the bash script requires system dependencies.

Both versions perform connectivity tests (ping) and can optionally execute a traceroute (mtr) for network diagnostics, generating timestamped reports.

Each ICMP Ping test is performed for 59 seconds in a row. The goal is to capture any link oscillations or variations in latency. If you need a high precision report, testing every minute is recommended.

You must create the `/var/log/goteira` directory before running the script and ensure it has write permissions.

Originally, this software was named "sergioreis.sh" in honor of the Brazilian singer and songwriter Sérgio Reis and his 1985 song "Pinga Ni Mim". Upon releasing the source code publicly, as I did not have the artist's authorization to use his name, I chose to rename it to "goteira", which means "drip" or "a leak in the ceiling" in Portuguese.

---

## 1. Shell Script Version (`goteira.sh`)

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
- **`--sysping`**: Uses the system's `ping` command instead of the internal Rust implementation.
- **`--sysmtr`**: Uses the system's `mtr` command for traceroute. Dependencies must be installed and are the same packages required by the bash script version.
- **`--selftraceroute`**: Uses the internal Rust traceroute implementation.
- **`-h`, `--help`**: Displays help information.

**Note:** If no traceroute option (`--sysmtr` or `--selftraceroute`) is provided, only ping will be executed.

#### Examples

**Ping Only (Internal Implementation):**
```bash
./target/release/goteira 8.8.8.8
```

**Ping (System) + MTR (System):**
```bash
./target/release/goteira --sysping --sysmtr 8.8.8.8
```
*This reproduces the behavior of the `goteira.sh -m` script.*

**Ping (Internal) + Traceroute (Internal):**
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
    */5 * * * * /opt/goteira.sh -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
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

Goteira is available as a Snap package in two versions:

1.  **goteira-shell**: The shell script version.
2.  **goteira-rust**: The Rust version.

### Install from the Snap Store

You can install either version directly from the Snap Store:

**Install Shell Version:**
```bash
sudo snap install goteira-shell
```

**Install Rust Version:**
```bash
sudo snap install goteira-rust
```

```bash
sudo snap connect goteira-shell:network-observe
# or
sudo snap connect goteira-rust:network-observe
```

### Logs and Reports (Snap)

When installed via Snap, the software does not have permission to write to `/var/log/goteira`. Instead, it uses the standard Snap writable directory:

- **Reports Path**: `/var/snap/goteira-[rust|shell]/common/YEAR/MONTH/DAY/...`
- **Variable**: The software automatically detects the `$SNAP_COMMON` environment variable to determine this path.

For manual installations, the path remains `/var/log/goteira`.

---

## 6. License

This project is licensed under the **GNU General Public License v3.0 or later**. See the [LICENSE](LICENSE) file for details.
