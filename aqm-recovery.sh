#!/bin/bash
# aqm-recovery : snapshots BTRFS reels de la partition data persistante
# (config, /home, file DTN) — pas une sauvegarde simulee.
set -euo pipefail

DATA_MOUNT="/data"
SNAP_DIR="${DATA_MOUNT}/snapshots"

case "${1:-}" in
  snapshot)
    label="${2:-manual}"
    ts="$(date +%Y%m%d-%H%M%S)"
    mkdir -p "${SNAP_DIR}"
    btrfs subvolume snapshot -r "${DATA_MOUNT}" "${SNAP_DIR}/${ts}-${label}"
    echo "Snapshot cree: ${SNAP_DIR}/${ts}-${label}"
    ;;
  list)
    ls -1 "${SNAP_DIR}" 2>/dev/null || echo "Aucun snapshot."
    ;;
  restore)
    snap="${2:?Usage: aqm-recovery restore <nom-snapshot>}"
    echo "Restauration de ${snap} — a confirmer avant reboot."
    btrfs subvolume snapshot "${SNAP_DIR}/${snap}" "${DATA_MOUNT}.restored"
    echo "Snapshot restaure dans ${DATA_MOUNT}.restored, bascule manuelle requise."
    ;;
  *)
    echo "Usage: aqm-recovery snapshot|list|restore [args]"
    exit 1
    ;;
esac
