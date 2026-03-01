#!/usr/bin/env bash
set -euo pipefail

APP_NAME="${APP_NAME:-pratrol}"
SERVICE_NAME="${SERVICE_NAME:-pratrol}"
BUILD_PROFILE="${BUILD_PROFILE:-release}"
BUILD_BIN="${BUILD_BIN:-target/${BUILD_PROFILE}/${APP_NAME}}"
RELEASES_DIR="${RELEASES_DIR:-/opt/pratrol/releases}"
OWNER="${OWNER:-pratrol}"
GROUP="${GROUP:-pratrol}"
NGINX_ACTIVE_SNIPPET="${NGINX_ACTIVE_SNIPPET:-/etc/nginx/snippets/pratrol-active.conf}"
NGINX_BLUE_SNIPPET="${NGINX_BLUE_SNIPPET:-/etc/nginx/snippets/pratrol-blue.conf}"
NGINX_GREEN_SNIPPET="${NGINX_GREEN_SNIPPET:-/etc/nginx/snippets/pratrol-green.conf}"

COLOR="auto"
SWITCH_TRAFFIC=0
SKIP_BUILD=0

usage() {
  cat <<'EOF'
Usage: deploy/deploy-bluegreen.sh [options]

Options:
  --color blue|green|auto  Deploy target color. auto = opposite of active nginx color.
  --switch                 Switch nginx active snippet to deployed color and reload nginx.
  --skip-build             Skip `cargo build --release`.
  -h, --help               Show this help.

Environment overrides:
  APP_NAME, SERVICE_NAME, BUILD_PROFILE, BUILD_BIN, RELEASES_DIR, OWNER, GROUP,
  NGINX_ACTIVE_SNIPPET, NGINX_BLUE_SNIPPET, NGINX_GREEN_SNIPPET.
EOF
}

as_root() {
  if [[ "${EUID}" -eq 0 ]]; then
    "$@"
  else
    sudo "$@"
  fi
}

active_color_from_snippet() {
  local resolved
  resolved="$(readlink -f "${NGINX_ACTIVE_SNIPPET}" 2>/dev/null || true)"
  if [[ "${resolved}" == *"pratrol-blue.conf" ]]; then
    echo "blue"
  elif [[ "${resolved}" == *"pratrol-green.conf" ]]; then
    echo "green"
  else
    echo "unknown"
  fi
}

inactive_color() {
  local active="$1"
  if [[ "${active}" == "blue" ]]; then
    echo "green"
  else
    echo "blue"
  fi
}

listen_addr_from_env_file() {
  local color="$1"
  local env_file="/etc/pratrol/${color}.env"
  if [[ -r "${env_file}" ]]; then
    awk -F= '/^LISTEN_ADDR=/{print $2; exit}' "${env_file}"
  fi
}

switch_nginx_to_color() {
  local color="$1"
  local target
  local previous

  if [[ "${color}" == "blue" ]]; then
    target="${NGINX_BLUE_SNIPPET}"
  else
    target="${NGINX_GREEN_SNIPPET}"
  fi

  previous="$(readlink -f "${NGINX_ACTIVE_SNIPPET}" 2>/dev/null || true)"

  as_root ln -sfn "${target}" "${NGINX_ACTIVE_SNIPPET}"

  if ! as_root nginx -t; then
    echo "nginx config test failed after switching snippet, reverting."
    if [[ -n "${previous}" ]]; then
      as_root ln -sfn "${previous}" "${NGINX_ACTIVE_SNIPPET}"
      as_root nginx -t >/dev/null 2>&1 || true
    fi
    return 1
  fi

  as_root systemctl reload nginx
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --color)
      COLOR="${2:-}"
      shift 2
      ;;
    --switch)
      SWITCH_TRAFFIC=1
      shift
      ;;
    --skip-build)
      SKIP_BUILD=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ "${COLOR}" != "blue" && "${COLOR}" != "green" && "${COLOR}" != "auto" ]]; then
  echo "--color must be blue, green, or auto." >&2
  exit 1
fi

if [[ "${COLOR}" == "auto" ]]; then
  ACTIVE_COLOR="$(active_color_from_snippet)"
  if [[ "${ACTIVE_COLOR}" == "unknown" ]]; then
    echo "Cannot detect active color from ${NGINX_ACTIVE_SNIPPET}. Use --color blue|green." >&2
    exit 1
  fi
  TARGET_COLOR="$(inactive_color "${ACTIVE_COLOR}")"
else
  TARGET_COLOR="${COLOR}"
fi

echo "Deploy target color: ${TARGET_COLOR}"

if [[ "${SKIP_BUILD}" -eq 0 ]]; then
  echo "Building ${APP_NAME} (${BUILD_PROFILE})..."
  if [[ "${BUILD_PROFILE}" == "release" ]]; then
    cargo build --release
  else
    cargo build --profile "${BUILD_PROFILE}"
  fi
fi

if [[ ! -x "${BUILD_BIN}" ]]; then
  echo "Built binary missing or not executable: ${BUILD_BIN}" >&2
  exit 1
fi

TIMESTAMP="$(date +%Y%m%d%H%M%S)"
RELEASE_DIR="${RELEASES_DIR}/${TIMESTAMP}"
COLOR_LINK="${RELEASES_DIR}/current-${TARGET_COLOR}"
DEST_BIN="${RELEASE_DIR}/${APP_NAME}"

echo "Installing release to ${DEST_BIN}"
as_root install -d -m 0755 -o "${OWNER}" -g "${GROUP}" "${RELEASE_DIR}"
as_root install -m 0755 -o "${OWNER}" -g "${GROUP}" "${BUILD_BIN}" "${DEST_BIN}"

echo "Updating ${COLOR_LINK} symlink"
as_root ln -sfn "${RELEASE_DIR}" "${COLOR_LINK}"

echo "Restarting ${SERVICE_NAME}@${TARGET_COLOR}.service"
as_root systemctl restart "${SERVICE_NAME}@${TARGET_COLOR}.service"
as_root systemctl is-active --quiet "${SERVICE_NAME}@${TARGET_COLOR}.service"

LISTEN_ADDR="$(listen_addr_from_env_file "${TARGET_COLOR}" || true)"
if command -v curl >/dev/null 2>&1 && [[ -n "${LISTEN_ADDR}" ]]; then
  echo "Health-checking http://${LISTEN_ADDR}/health"
  tries=30
  ok=0
  for ((i=1; i<=tries; i++)); do
    if curl -fsS "http://${LISTEN_ADDR}/health" >/dev/null; then
      ok=1
      break
    fi
    sleep 1
  done

  if [[ "${ok}" -ne 1 ]]; then
    echo "Health check failed for ${TARGET_COLOR} after ${tries} attempts." >&2
    exit 1
  fi
else
  echo "Skipping HTTP health check (curl missing or LISTEN_ADDR missing in /etc/pratrol/${TARGET_COLOR}.env)."
fi

if [[ "${SWITCH_TRAFFIC}" -eq 1 ]]; then
  echo "Switching nginx active color to ${TARGET_COLOR}"
  switch_nginx_to_color "${TARGET_COLOR}"
fi

echo "Deployment complete."
