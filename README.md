# Goteira Manual

This document provides complete instructions for installing and using both versions of the software: **Shell Script** (`goteira.sh`) and **Rust** (`goteira`).

Both versions perform connectivity tests (ping) and can optionally execute a traceroute (mtr) for network diagnostics, generating timestamped reports.

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
    sudo mv goteira.sh /usr/local/bin/goteira
    ```

### Usage

The basic syntax is:

```bash
./goteira.sh [-m] <TARGET>
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
- **`--sysmtr`**: Uses the system's `mtr` command for traceroute.
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
    */5 * * * * /usr/local/bin/goteira.sh -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
    ```

This will:
- Execute `goteira.sh` every 5 minutes.
- Perform ping and traceroute (`-m`).
- Save standard output (ping stats) to `/var/log/goteira/goteira.log`.
- Detailed MTR reports will continue to be saved in the date/time directory structure.
