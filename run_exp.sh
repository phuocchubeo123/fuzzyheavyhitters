#!/usr/bin/env bash
set -euo pipefail

ROLE="${1:-}"
if [[ -z "$ROLE" ]]; then
  echo "Usage: $0 {server0|server1|client|dealer}" >&2
  exit 1
fi

mkdir -p results

BUSIESTS=(day week month)
SHARES=(fss okvs)
METHODS=(gc_gc gc_fss fss_gc fss_fss)
METRICS=(linf l1 l2)

run_server() {
  local side="$1"
  local role_name="$2"

  for BUSIEST in "${BUSIESTS[@]}"; do
    for SHARE in "${SHARES[@]}"; do
      for METHOD in "${METHODS[@]}"; do
        for METRIC in "${METRICS[@]}"; do
          echo "Running ROLE=${role_name}, BUSIEST=${BUSIEST}, SHARE=${SHARE}, METHOD=${METHOD}, METRIC=${METRIC}"

          cargo run --release --bin mosaic_server -- \
            --side "$side" \
            --config "configs/unknown_dictionary/busiest_${BUSIEST}/${METRIC}_${SHARE}_config.json" \
            --network-config configs/network_config.json \
            --method-config "configs/method/${METHOD}.json" \
            --threads 32 > "results/${role_name}_busiest_${BUSIEST}_${METRIC}_${SHARE}_${METHOD}.log" 2>&1
        done
      done
    done
  done
}

run_client() {
  for BUSIEST in "${BUSIESTS[@]}"; do
    for SHARE in "${SHARES[@]}"; do
      for METRIC in "${METRICS[@]}"; do
        echo "Running ROLE=client, BUSIEST=${BUSIEST}, SHARE=${SHARE}, METRIC=${METRIC}"

        cargo run --release --bin mosaic_client -- \
          --config "configs/unknown_dictionary/busiest_${BUSIEST}/${METRIC}_${SHARE}_config.json" \
          --network-config configs/network_config.json > "results/client_busiest_${BUSIEST}_${METRIC}_${SHARE}.log" 2>&1
      done
    done
  done
}

run_dealer() {
  for BUSIEST in "${BUSIESTS[@]}"; do
    for SHARE in "${SHARES[@]}"; do
      for METRIC in "${METRICS[@]}"; do
        echo "Running ROLE=dealer, BUSIEST=${BUSIEST}, SHARE=${SHARE}, METRIC=${METRIC}"

        cargo run --release --bin mosaic_dealer -- \
          --config "configs/unknown_dictionary/busiest_${BUSIEST}/${METRIC}_${SHARE}_config.json" \
          --network-config configs/network_config.json \
          --threads 32 > "results/dealer_busiest_${BUSIEST}_${METRIC}_${SHARE}.log" 2>&1
      done
    done
  done
}

case "$ROLE" in
  server0)
    run_server 0 server0
    ;;
  server1)
    run_server 1 server1
    ;;
  client)
    run_client
    ;;
  dealer)
    run_dealer
    ;;
  *)
    echo "Invalid role: $ROLE" >&2
    echo "Usage: $0 {server0|server1|client|dealer}" >&2
    exit 1
    ;;
esac
