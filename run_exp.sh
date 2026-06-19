#!/usr/bin/env bash
set -euo pipefail

ROLE="${1:-}"
if [[ -z "$ROLE" ]]; then
  echo "Usage: $0 {server0|server1|client|dealer}" >&2
  exit 1
fi

mkdir -p results

BUSIESTS=(week month)
SHARES=(fss okvs)
METHODS=(gc_gc gc_fss fss_gc fss_fss)
METRICS=(linf l1 l2)

case "$ROLE" in
  server0|server1|client|dealer)
    ;;
  *)
    echo "Invalid role: $ROLE" >&2
    echo "Usage: $0 {server0|server1|client|dealer}" >&2
    exit 1
    ;;
esac

for BUSIEST in "${BUSIESTS[@]}"; do
  for SHARE in "${SHARES[@]}"; do
    for METHOD in "${METHODS[@]}"; do
      for METRIC in "${METRICS[@]}"; do
        echo "Running ROLE=${ROLE}, BUSIEST=${BUSIEST}, SHARE=${SHARE}, METHOD=${METHOD}, METRIC=${METRIC}"

        case "$ROLE" in
          server0)
            cargo run --release --bin mosaic_server -- \
              --side 0 \
              --config "configs/unknown_dictionary/busiest_${BUSIEST}/${METRIC}_${SHARE}_config.json" \
              --network-config configs/network_config.json \
              --method-config "configs/method/${METHOD}.json" \
              --threads 32 > "results/server0_busiest_${BUSIEST}_${METRIC}_${SHARE}_${METHOD}.log" 2>&1
            ;;
          server1)
            cargo run --release --bin mosaic_server -- \
              --side 1 \
              --config "configs/unknown_dictionary/busiest_${BUSIEST}/${METRIC}_${SHARE}_config.json" \
              --network-config configs/network_config.json \
              --method-config "configs/method/${METHOD}.json" \
              --threads 32 > "results/server1_busiest_${BUSIEST}_${METRIC}_${SHARE}_${METHOD}.log" 2>&1
            ;;
          client)
            cargo run --release --bin mosaic_client -- \
              --config "configs/unknown_dictionary/busiest_${BUSIEST}/${METRIC}_${SHARE}_config.json" \
              --network-config configs/network_config.json > "results/client_busiest_${BUSIEST}_${METRIC}_${SHARE}_${METHOD}.log" 2>&1
            ;;
          dealer)
            cargo run --release --bin mosaic_dealer -- \
              --config "configs/unknown_dictionary/busiest_${BUSIEST}/${METRIC}_${SHARE}_config.json" \
              --network-config configs/network_config.json \
              --threads 32 > "results/dealer_busiest_${BUSIEST}_${METRIC}_${SHARE}_${METHOD}.log" 2>&1
            ;;
        esac
      done
    done
  done
done
