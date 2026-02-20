#!/usr/bin/env bash
set -euo pipefail

COVERAGE_JSON="${1:-coverage.json}"
OUT_DIR="${2:-site}"
OUT_FILE="${OUT_DIR}/coverage_badge.json"

mkdir -p "$OUT_DIR"

read -r LINE FUNC REGION < <(
  jq -r '
    [
      (.data[0].totals.lines.percent     // 0),
      (.data[0].totals.functions.percent // 0),
      (.data[0].totals.regions.percent   // 0)
    ] | @tsv
  ' "$COVERAGE_JSON" \
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

if   [ "$PERCENT_INT" -ge 90 ]; then COLOR="brightgreen"
elif [ "$PERCENT_INT" -ge 75 ]; then COLOR="yellowgreen"
else COLOR="red"
fi

cat > "$OUT_FILE" <<EOF
{
  "schemaVersion": 1,
  "label": "coverage",
  "message": "${PERCENT}%",
  "color": "${COLOR}"
}
EOF

echo "Wrote $OUT_FILE (LINE=$LINE FUNC=$FUNC REGION=$REGION AVG=$PERCENT COLOR=$COLOR)"