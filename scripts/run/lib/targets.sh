# run.sh 主机/用户目标配置聚合入口

TARGETS_LIB_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/targets" && pwd -P)"
# shellcheck source=/dev/null
source "${TARGETS_LIB_DIR}/host.sh"
# shellcheck source=/dev/null
source "${TARGETS_LIB_DIR}/users.sh"
# shellcheck source=/dev/null
source "${TARGETS_LIB_DIR}/overrides.sh"
