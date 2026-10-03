# Migrating from `goteira.sh` (shell) to `goteira` (Rust)

The shell script is **deprecated** (last version 0.3.1) and will be removed in 1.0.0. The Rust version is a drop-in replacement for the common use case. Portuguese: [MIGRATION.pt-br.md](MIGRATION.pt-br.md).

## Crontab

```diff
-*/5 * * * * /opt/goteira.sh -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
+*/5 * * * * /opt/goteira/goteira -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
```

Only the path changes; `-m` and the log layout are the same. Install the binary as described in the [README](README.md#pre-built-binary-recommended). `ping` (and `mtr` for `-m`) are still required on the system.

## What is identical

- Command line: `goteira [-m] TARGET`.
- Stdout line, **TAB-separated**: `[DD/MM/YY-HH:MM]<TAB>LOSS%<TAB>MIN/AVG/MAX/MDEV<TAB>TARGET`.
- ping: `ping -qnAw 59` (59 s of adaptive ICMP).
- mtr: `mtr --report --report-wide --aslookup --report-cycles 30`, saved to `/var/log/goteira/YYYY/MM/DD/HH/MM/TARGET.txt` (or `$SNAP_COMMON` in a snap).
- Reports older than 30 days are deleted.

## Differences to know

| Topic | Shell | Rust |
|---|---|---|
| Exit code | Always `0` (script's last command) | `1` if ping fails or loses 100% of packets, otherwise `0` |
| Ping failure (bad host, no reply) | Prints a `100.0%` line | Same line, plus the cause on stderr |
| stderr | Deprecation notice, only on a terminal | Errors only |
| Extra options | none | `--selfping`, `--selftraceroute` (experimental, off by default) |
| Dependencies | `sh`, coreutils, grep, sed | None besides `ping`/`mtr` (static binary) |

If something that checks the exit code now reacts to `1` in a cron job, that is the intended signal that the link is down; append `|| true` to keep the old behavior.

## Snap

The legacy `goteira-rust` snap is replaced by `goteira`:

```bash
sudo snap remove goteira-rust
sudo snap install goteira
sudo snap connect goteira:network-observe
```

Reports move from `/var/snap/goteira-rust/common` to `/var/snap/goteira/common`; old ones are not migrated automatically.
