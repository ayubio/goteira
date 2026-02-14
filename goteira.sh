#!/bin/sh
# goteira.sh - v0.3 by dev@ayub.net.br
# - "Nessa casa tem goteira! Pinga ni mim! Pinga nimim! Pinga nimim!"
#
# Usage: ./goteira.sh [-m] alvo
# Results are shown in stdout and traceroute (mtr) is saved at /var/log/goteira. Use >> to redirect output to a file.
# Example to use in crontab:
# */5 * * * * /opt/goteira.sh -m 8.8.8.8 >> /var/log/goteira/goteira.log
#
set -u

# Default values
RUN_MTR=0
target=""

# Argument parsing
while getopts ":m" opt; do
  case ${opt} in
    m )
      RUN_MTR=1
      ;;
    \? )
      echo "Opção inválida: -$OPTARG" 1>&2
      echo "Uso: $0 [-m] alvo" 1>&2
      exit 1
      ;;
  esac
done
shift $((OPTIND -1))

if [ $# -eq 0 ]; then
    echo "Erro: Alvo não especificado."
    echo "Uso: $0 [-m] alvo"
    exit 1
fi

target=$1
timestamp=$(date +%d/%m/%y-%H:%M)
day=$(date +%d)
month=$(date +%m)
year=$(date +%Y)
hour=$(date +%H)
minute=$(date +%M)
reportrootpath="/var/log/goteira"
thisreportpath=$reportrootpath/$year/$month/$day/$hour/$minute
thisreportfullpath="$thisreportpath/$target.txt"
temp1=$(mktemp)
temp2=$(mktemp)

#Coletando dados
if [ "$RUN_MTR" -eq 1 ]; then
    mkdir -p $thisreportpath
    mtr --report --report-wide --aslookup --report-cycles 30 $target > $temp2 &
    MTR_PID=$!
fi

ping -qnAw 59 $target > $temp1

#Processando dados
loss=$(fgrep "loss" $temp1 | cut -d"," -f 3 | sed 's/^ *//g' | cut -d" " -f 1 | cut -d"%" -f 1 | xargs printf "%.*f\n" 1)
stats=$(fgrep "rtt" $temp1 | cut -d"=" -f 2 | cut -d"," -f 1 | cut -d" " -f 2)

if [ -z "$stats" ]; then
    # Handle case where ping fails completely (no RTT stats)
    min="0.0"
    avg="0.0"
    max="0.0"
    mdev="0.0"
    # Loss might still be parsed or needs default; typically 100% if no reply
    if [ -z "$loss" ]; then loss="100.0"; fi
else
    min=$(echo $stats | cut -d"/" -f 1 | xargs printf "%.*f\n" 1)
    avg=$(echo $stats | cut -d"/" -f 2 | xargs printf "%.*f\n" 1)
    max=$(echo $stats | cut -d"/" -f 3 | xargs printf "%.*f\n" 1)
    mdev=$(echo $stats | cut -d"/" -f 4 | xargs printf "%.*f\n" 1)
fi

#Exibindo e salvando dados
echo "[$timestamp] $loss% $min/$avg/$max/$mdev $target" | sed -e 's/ /\t/g'

if [ "$RUN_MTR" -eq 1 ]; then
    wait $MTR_PID
    mv $temp2 $thisreportfullpath
fi

#Limpando temporários e relatórios mais antigos que 30 dias
rm -f $temp1 $temp2 &
find $reportrootpath -type f -mtime +30 -delete &

