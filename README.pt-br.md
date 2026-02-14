# Manual do Goteira

Este documento fornece instruções completas para instalação e uso das duas versões do software: **Shell Script** (`goteira.sh`) e **Rust** (`goteira`). O Gemmini foi responsável pela versão em Rust derivada do script Bash original. O objetivo é fornecer uma versão independente (standalone), já que o script bash requer dependências do sistema.

Ambas as versões realizam testes de conectividade (ping) e podem opcionalmente executar um traceroute (mtr) para diagnóstico de rede, gerando relatórios com data/hora.

Cada teste de ICMP Ping é realizado continuamente por 59 segundos. O objetivo é capturar oscilações no link ou variações de latência. Se você precisa de relatórios de alta precisão, recomenda-se testar a cada minuto.

Você deve criar o diretório `/var/log/goteira` antes de executar o script e garantir que ele tenha permissões de escrita.

Originalmente, esse software se chamava "sergioreis.sh" em homenagem ao cantor e compositor brasileiro Sérgio Reis e sua música "Pinga Ni Mim", de 1985. Ao lançar o código fonte publicamente, por não ter autorização do artista para usar o nome dele, optei por rebatizá-lo de goteira, que significa em inglês "drip" ou "a leak in the ceiling". 

---

## 1. Versão Shell Script (`goteira.sh`)

A versão original em Bash, leve e com dependências comuns de sistemas Linux.

### Pré-requisitos

Certifique-se de ter as seguintes ferramentas instaladas no seu sistema:

- `bash` (ou `sh` compatível)
- `ping` (iputils-ping)
- `mtr` (para a funcionalidade de traceroute)
- `coreutils` (date, mktemp, rm, mv, mkdir, etc.)

Em sistemas baseados em Debian/Ubuntu, você pode instalar o necessário com:
```bash
sudo apt update
sudo apt install iputils-ping mtr-tiny coreutils
```

### Instalação

1.  Baixe o script `goteira.sh`.
2.  Dê permissão de execução ao arquivo:
    ```bash
    chmod +x goteira.sh
    ```
3.  (Opcional) Mova para um diretório no seu PATH para executar de qualquer lugar:
    ```bash
    sudo mv goteira.sh /opt/goteira/goteira.sh
    ```

### Uso

A sintaxe básica é:

```bash
/opt/goteira/goteira.sh [-m] <ALVO>
```

- **`<ALVO>`**: O endereço IP ou hostname que você deseja testar (ex: `8.8.8.8`, `google.com`).
- **`-m`**: (Opcional) Ativa a execução do traceroute (`mtr`) em paralelo ao ping. Se omitido, apenas o ping será executado.

#### Exemplos

**Apenas Ping (Padrão):**
```bash
./goteira.sh 8.8.8.8
```
*Saída: Exibe estatísticas de latência e perda de pacotes no terminal.*

**Ping com Traceroute (MTR):**
```bash
./goteira.sh -m 8.8.8.8
```
*Saída: Exibe estatísticas de ping no terminal e, em background, salva um relatório detalhado do MTR em `/var/log/goteira/...`.*

---

## 2. Versão Rust (`goteira`)

A versão moderna e reescrita em Rust, com melhor performance e estruturação.

### Pré-requisitos

Para compilar e rodar esta versão, você precisa do ambiente de desenvolvimento Rust instalado.

- **Rust e Cargo**: Instale via [rustup.rs](https://rustup.rs/):
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

### Instalação / Compilação

1.  Navegue até o diretório do projeto:
    ```bash
    cd /caminho/para/goteira
    ```
2.  Compile o projeto em modo de lançamento (release) para otimização:
    ```bash
    cargo build --release
    ```
3.  O binário será gerado em `./target/release/goteira`.

### Uso

Você pode rodar diretamente via `cargo` ou executar o binário compilado.

#### Sintaxe

```bash
cargo run --release -- [OPÇÕES] <ALVO>
# ou
./target/release/goteira [OPÇÕES] <ALVO>
```

#### Opções Disponíveis

- **`<ALVO>`**: O endereço IP ou hostname (Obrigatório).
- **`--sysping`**: Usa o comando `ping` do sistema em vez da implementação interna em Rust.
- **`--sysmtr`**: Usa o comando `mtr` do sistema para o traceroute. As dependências devem estar instaladas e são os mesmos pacotes requeridos pela versão shell script.
- **`--selftraceroute`**: Usa a implementação interna de traceroute em Rust.
- **`-h`, `--help`**: Exibe a ajuda.

**Nota:** Se nenhuma opção de traceroute (`--sysmtr` ou `--selftraceroute`) for fornecida, apenas o ping será executado.

#### Exemplos

**Apenas Ping (Implementação Interna):**
```bash
./target/release/goteira 8.8.8.8
```

**Ping (Sistema) + MTR (Sistema):**
```bash
./target/release/goteira --sysping --sysmtr 8.8.8.8
```
*Isso reproduz o comportamento do script `goteira.sh -m`.*

**Ping (Interno) + Traceroute (Interno):**
```bash
./target/release/goteira --selftraceroute 8.8.8.8
```

### Logs e Relatórios

Assim como a versão Shell, a versão Rust salva os relatórios de traceroute (quando ativados) em:  
`/var/log/goteira/ANO/MES/DIA/HORA/MINUTO/<ALVO>.txt`

---

## 3. Automação com Crontab

Para monitoramento contínuo, você pode agendar a execução do Goteira através do `crontab`.

### Exemplo de Configuração

Para rodar o script a cada 5 minutos, coletando mtr e salvando o log geral em um arquivo:

1.  Edite o crontab:
    ```bash
    crontab -e
    ```
2.  Adicione a linha (ajuste os caminhos conforme sua instalação):
    ```cron
    */5 * * * * /opt/goteira.sh -m 8.8.8.8 >> /var/log/goteira/goteira.log 2>&1
    ```

Isso irá:
- Executar o `goteira.sh` a cada 5 minutos.
- Realizar o ping e o traceroute (`-m`).
- Salvar a saída padrão (ping stats) em `/var/log/goteira/goteira.log`.
- Os relatórios detalhados do MTR continuarão sendo salvos na estrutura de diretórios de data/hora.


## 4. Exemplo de Saída

```
ayubio@baostar:~/software/goteira$ while true; do ./goteira.sh 8.8.8.8; sleep 60; done
[14/02/26-18:24]	0.0%	3.1/6.2/83.5/3.2	8.8.8.8
[14/02/26-18:26]	0.0%	3.1/5.7/28.6/1.5	8.8.8.8
[14/02/26-18:28]	0.0%	3.1/7.3/228.3/9.2	8.8.8.8
[14/02/26-18:30]	0.0%	3.1/6.8/201.6/8.1	8.8.8.8
```

A primeira coluna é o carimbo de data/hora, a segunda coluna é a porcentagem de perda de pacotes (loss%), a terceira coluna min/média/máx/jitter (como `ping -q` mostraria), e a última coluna é o endereço IP alvo para facilitar o uso do `grep`.

---

## 5. Instalação (Snap)

O Goteira está disponível como pacote Snap em duas versões:

1.  **goteira-shell**: A versão em shell script.
2.  **goteira-rust**: A versão em Rust.

### Instalar via Snap Store

Você pode instalar qualquer uma das versões diretamente da loja oficial:

**Instalar Versão Shell:**
```bash
sudo snap install goteira-shell
```

**Instalar Versão Rust:**
```bash
sudo snap install goteira-rust
```

### Permissões Necessárias

Como os Snaps rodam em um ambiente restrito, você deve conectar manualmente a interface `network-observe` para permitir que a ferramenta realize diagnósticos de rede (necessário para o `mtr` e pings de baixo nível):

```bash
sudo snap connect goteira-shell:network-observe
# ou
sudo snap connect goteira-rust:network-observe
```

### Logs e Relatórios (Snap)

Quando instalado via Snap, o software não tem permissão para escrever em `/var/log/goteira`. Em vez disso, ele utiliza o diretório padrão de escrita do Snap:

- **Caminho dos Relatórios**: `/var/snap/goteira-[rust|shell]/common/ANO/MES/DIA/...`
- **Variável**: O software detecta automaticamente a variável de ambiente `$SNAP_COMMON` para determinar este caminho.

Para instalações manuais, o caminho permanece sendo `/var/log/goteira`.

