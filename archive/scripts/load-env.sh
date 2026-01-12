#!/usr/bin/env bash
set -euo pipefail

# 本地开发环境注入脚本
# 用法：
#   source scripts/load-env.sh                # 默认加载 config/env/.env.local
#   source scripts/load-env.sh dev           # 加载 config/env/.env.dev（如果存在）
#   source scripts/load-env.sh example       # 加载 config/env/.env.example

ENV_NAME="${1:-local}"

case "${ENV_NAME}" in
  local)   ENV_FILE="config/env/.env.local" ;;
  dev)     ENV_FILE="config/env/.env.dev" ;;
  test)    ENV_FILE="config/env/.env.test" ;;
  example) ENV_FILE="config/env/.env.example" ;;
  *)
    echo "Unknown env name: ${ENV_NAME}" >&2
    return 2
    ;;
esac

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "Env file not found: ${ENV_FILE}" >&2
  echo "Tip: cp config/env/.env.example config/env/.env.local" >&2
  return 1
fi

set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
set +a

echo "Loaded env: ${ENV_FILE}"
