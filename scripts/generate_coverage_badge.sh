#!/usr/bin/env bash
set -euo pipefail

USAGE=$(cat << EOS
USAGE
    generate_coverage_badge.sh [OPTIONS] [json report path]

DESCRIPTION
    Generates a json file for [shilds.io's Endpoint Badge](https://shields.io/badges/endpoint-badge)
    from [cargo llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)'s --json report.

ARGUMENTS
    A file path that holds the json report by cargo llvm-cov.
    If this is not passed, the json report content is expected to be passed via stdin.

OPTIONS
    -h or --help
        Show this message and exit.
    -o or --out <path>
        Set output file path. (default: coverage_badge.json)
EOS
)

OUT_FILE="$PWD/coverage_badge.json"
JSON_REPORT_PATH=''

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    -h|--help)
      echo "$USAGE"
      shift 1
      exit 0
      ;;
    -o|--out)
      OUT_FILE="$2"
        shift 2
        ;;
    *)
      JSON_REPORT_PATH="$1"
      shift 1
      if [[ "$#" -gt 0 ]]; then
        echo "unknown arguments: $*"
        echo "$USAGE"
        exit 1
      fi
  esac
done

if [ -n "$JSON_REPORT_PATH" ]; then
  JSON_REPORT=$(cat "$JSON_REPORT_PATH")
else
  read -r JSON_REPORT || [ -n "$JSON_REPORT" ]
fi

read -r LINE FUNC REGION < <(
  echo "$JSON_REPORT" \
  | jq -r '
    [
      (.data[0].totals.lines.percent     // 0),
      (.data[0].totals.functions.percent // 0),
      (.data[0].totals.regions.percent   // 0)
    ] | @tsv
  ' \
  | awk -F'\t' '{
    for (i=1;i<=3;i++) {
      gsub(/[^0-9.]/, "", $i);
      if ($i == "" ) $i = 0;
    }
    print $1, $2, $3
  }'
)

PERCENT="$(awk -v l="$LINE" -v f="$FUNC" -v r="$REGION" 'BEGIN{ printf("%.2f", (l+f+r)/3.0) }')"
PERCENT_INT="$(awk -v p="$PERCENT" 'BEGIN{ printf("%d", (p + 0.5)) }')"

if [ "$PERCENT_INT" -ge 90 ]; then
  COLOR="brightgreen"
elif [ "$PERCENT_INT" -ge 75 ]; then
  COLOR="yellowgreen"
else
  COLOR="red"
fi

mkdir -p "$(dirname OUT_DIR)"
cat > "$OUT_FILE" <<EOF
{
  "schemaVersion": 1,
  "label": "coverage",
  "message": "${PERCENT}%",
  "color": "${COLOR}"
}
EOF

echo "Wrote $OUT_FILE (LINE=$LINE FUNC=$FUNC REGION=$REGION AVG=$PERCENT COLOR=$COLOR)"
