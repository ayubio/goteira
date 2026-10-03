# Migrando do `goteira.sh` (shell) para o `goteira` (Rust)

O shell script está **deprecado** (última versão 0.3.1) e será removido na 1.0.0. A versão Rust substitui o script no uso comum sem mudanças de comportamento relevantes. English: [MIGRATION.md](MIGRATION.md).

## Crontab

```diff
-*/5 * * * * /opt/goteira.sh -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
+*/5 * * * * /opt/goteira/goteira -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
```

Só o caminho muda; `-m` e a estrutura dos logs são iguais. Instale o binário conforme o [README](README.pt-br.md). `ping` (e `mtr`, para o `-m`) continuam sendo necessários no sistema.

## O que é idêntico

- Linha de comando: `goteira [-m] ALVO`.
- Linha no stdout, **separada por TAB**: `[DD/MM/YY-HH:MM]<TAB>LOSS%<TAB>MIN/AVG/MAX/MDEV<TAB>ALVO`.
- ping: `ping -qnAw 59` (59 s de ICMP adaptativo).
- mtr: `mtr --report --report-wide --aslookup --report-cycles 30`, salvo em `/var/log/goteira/AAAA/MM/DD/HH/MM/ALVO.txt` (ou `$SNAP_COMMON` em um snap).
- Relatórios com mais de 30 dias são apagados.

## Diferenças a conhecer

| Tema | Shell | Rust |
|---|---|---|
| Código de saída | Sempre `0` (último comando do script) | `1` se o ping falha ou perde 100% dos pacotes, senão `0` |
| Falha de ping (host inválido, sem resposta) | Imprime linha `100.0%` | Mesma linha, mais a causa no stderr |
| stderr | Aviso de deprecação, só em terminal | Apenas erros |
| Opções extras | nenhuma | `--selfping`, `--selftraceroute` (experimentais, desligadas por padrão) |
| Dependências | `sh`, coreutils, grep, sed | Só `ping`/`mtr` (binário estático) |

Se um cron que verifica o código de saída passar a reagir ao `1`, esse é o sinal intencional de que o link caiu; acrescente `|| true` para manter o comportamento antigo.

## Snap

O snap legado `goteira-rust` foi substituído pelo `goteira`:

```bash
sudo snap remove goteira-rust
sudo snap install goteira
sudo snap connect goteira:network-observe
```

Os relatórios passam de `/var/snap/goteira-rust/common` para `/var/snap/goteira/common`; os antigos não são migrados automaticamente.
