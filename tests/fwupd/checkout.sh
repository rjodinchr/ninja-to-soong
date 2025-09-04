set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/fwupd/fwupd.git 560f741280ad9ac787694799ef015e11ac820015 "${DEST}/external/fwupd"
