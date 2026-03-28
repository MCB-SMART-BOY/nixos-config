# run.sh 覆盖配置聚合入口

TARGET_OVERRIDES_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/overrides" && pwd -P)"
# shellcheck source=/dev/null
source "${TARGET_OVERRIDES_LIB_DIR}/gpu.sh"
# shellcheck source=/dev/null
source "${TARGET_OVERRIDES_LIB_DIR}/server.sh"
# shellcheck source=/dev/null
source "${TARGET_OVERRIDES_LIB_DIR}/template.sh"
