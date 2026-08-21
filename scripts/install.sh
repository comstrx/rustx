#!/usr/bin/env bash
set -Eeuo pipefail

__dir__="${BASH_SOURCE[0]%/*}"
[[ "${__dir__}" == "${BASH_SOURCE[0]}" ]] && __dir__="."
__dir__="$(cd -- "${__dir__}" && pwd -P)"

source "${__dir__}/initial/installer.sh"
install "$@"
