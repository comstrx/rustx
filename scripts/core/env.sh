#!/usr/bin/env bash

[[ -n "${BASH_VERSION:-}" ]] || { printf '%s\n' "env.sh: Bash required." >&2; return 2 2>/dev/null || exit 2; }
(( ${BASH_VERSINFO[0]:-0} >= 5 )) || { printf '%s\n' "env.sh: Bash 5+ required." >&2; return 2 2>/dev/null || exit 2; }

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "env.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${ENV_LOADED:-}" ]] && return 0
ENV_LOADED=1

YES_ENV="${YES_ENV:-0}"
QUIET_ENV="${QUIET_ENV:-0}"
VERBOSE_ENV="${VERBOSE_ENV:-0}"
ERR_HANDLER="${ERR_HANDLER:-}"

OS_NAME_CACHE=""
IS_WSL_CACHE=""

if [[ -z "${ROOT_DIR:-}" ]]; then
    ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
    readonly ROOT_DIR
fi

colorize () {

    local want="${1-}"
    local fd="${2:-2}"

    [[ -z "${NO_COLOR:-}" && "${TERM:-}" != "dumb" ]] || { printf ''; return 0; }
    [[ -t "${fd}" ]] || { printf ''; return 0; }

    case "${want}" in
        blue)    printf '\033[38;5;51m' ;;
        green)   printf '\033[38;5;46m' ;;
        yellow)  printf '\033[38;5;226m' ;;
        red)     printf '\033[38;5;196m' ;;
        reset)   printf '\033[0m' ;;
        *)       printf '' ;;
    esac

}
log_emit () {

    local color="${1-}"
    local tag="${2-}"
    local lead="${3-}"
    shift 3 || true

    local pre="" suf=""

    pre="$(colorize "${color}" 2)"
    [[ -n "${pre}" ]] && suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b%b\n' "${lead}" "${pre}${tag}$*${suf}" >&2

}
info () {

    (( QUIET_ENV )) && return 0

    log_emit blue '💥 ' '' "$@"

}
success () {

    (( QUIET_ENV )) && return 0

    log_emit green '✅ ' '' "$@"

}
warn () {

    (( QUIET_ENV )) && return 0

    log_emit yellow '⚠️ ' '' "$@"

}
error () {

    log_emit red '❌ ' '' "$@"

}
info_ln () {

    (( QUIET_ENV )) && return 0

    log_emit blue '💥 ' '\n' "$@"

}
success_ln () {

    (( QUIET_ENV )) && return 0

    log_emit green '✅ ' '\n' "$@"

}
log () {

    local IFS=' '
    (( $# )) || { printf '\n' >&2; return 0; }
    printf '%b\n' "$*" >&2

}
log_info () {

    log_emit blue ' ' '' "$@"

}
print () {

    local IFS=' '

    if (( $# == 0 )); then
        printf '\n'
        return 0
    fi

    printf '%b\n' "$*"

}
die () {

    local msg="${1-}"
    local code="${2:-1}"

    [[ "${code}" =~ ^[0-9]+$ ]] || code=1
    [[ -n "${msg}" ]] && printf '%s\n' "❌ ${msg}" >&2
    [[ "${-}" == *i* && "${BASH_SOURCE[0]-}" != "${0-}" ]] && return "${code}"

    exit "${code}"

}
input () {

    local prompt="${1-}"
    local def="${2-}"
    local tty="/dev/tty"
    local line=""
    local rc=0

    if [[ -c "${tty}" && -r "${tty}" && -w "${tty}" ]]; then

        [[ -n "${prompt}" ]] && printf '%b' "${prompt}" >"${tty}"

        rc=0
        IFS= read -r line <"${tty}" || rc=$?

    else

        if declare -F is_ci >/dev/null 2>&1; then

            is_ci && {
                if declare -F die >/dev/null 2>&1; then
                    die "input: non-interactive (no /dev/tty)" 2
                fi
                return 2
            }

        fi

        [[ -n "${prompt}" ]] && printf '%b' "${prompt}" >&2

        rc=0
        IFS= read -r line || rc=$?

    fi

    if (( rc != 0 )); then

        [[ -n "${def}" ]] && { printf '%s' "${def}"; return 0; }
        return 1

    fi

    [[ -z "${line}" && -n "${def}" ]] && line="${def}"
    printf '%s' "${line}"

}
confirm () {

    local msg="${1:-Continue?}"
    local def="${2:-N}"

    (( YES_ENV )) && return 0

    if declare -F is_ci >/dev/null 2>&1; then
        is_ci && die "Refusing interactive prompt in CI." 2
    fi

    local d_is_yes=0
    case "${def}" in
        y|Y|yes|YES|Yes|1|true|TRUE|True) d_is_yes=1 ;;
    esac

    local hint="[y/N]: "
    (( d_is_yes )) && hint="[Y/n]: "

    local ans=""
    ans="$(input "${msg} ${hint}" "${def}")" || return $?

    case "${ans}" in
        y|Y|yes|YES|Yes|yep|Yep|YEP|1|true|TRUE|True) return 0 ;;
        n|N|no|NO|No|0|false|FALSE|False) return 1 ;;
        "") (( d_is_yes )) && return 0 || return 1 ;;
    esac

    return 1

}
cd_root () {

    cd -- "${ROOT_DIR}" || die "cd_root: cannot enter ROOT_DIR: ${ROOT_DIR}" 2

}
os_name () {

    [[ -n "${OS_NAME_CACHE}" ]] && { printf '%s' "${OS_NAME_CACHE}"; return 0; }

    local u=""
    u="$(uname -s 2>/dev/null || true)"

    [[ -n "${u}" ]] || u="${OSTYPE:-}"

    case "${u}" in
        Linux|linux*)                              OS_NAME_CACHE=linux ;;
        Darwin|darwin*)                            OS_NAME_CACHE=mac ;;
        MSYS*|MINGW*|CYGWIN*|msys*|mingw*|cygwin*) OS_NAME_CACHE=windows ;;
        *)                                         OS_NAME_CACHE=unknown ;;
    esac

    printf '%s' "${OS_NAME_CACHE}"

}
is_ci () {

    [[ -n "${CI:-}" || -n "${GITHUB_ACTIONS:-}" || -n "${GITLAB_CI:-}" || -n "${BUILDKITE:-}" || -n "${TF_BUILD:-}" ]]

}
is_ci_pull () {
    
    is_ci && [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]]

}
is_ci_push () {
    
    is_ci && [[ "${GITHUB_EVENT_NAME:-}" == "push" && "${GITHUB_REF:-}" == refs/tags/v* ]]

}
is_wsl () {

    [[ -n "${IS_WSL_CACHE}" ]] && return "${IS_WSL_CACHE}"

    IS_WSL_CACHE=1

    if [[ -n "${WSL_INTEROP:-}" || -n "${WSL_DISTRO_NAME:-}" ]]; then
        IS_WSL_CACHE=0
    elif [[ -r /proc/version ]] && grep -qiE 'microsoft|wsl' /proc/version 2>/dev/null; then
        IS_WSL_CACHE=0
    elif [[ -r /proc/sys/kernel/osrelease ]] && grep -qiE 'microsoft|wsl' /proc/sys/kernel/osrelease 2>/dev/null; then
        IS_WSL_CACHE=0
    fi

    return "${IS_WSL_CACHE}"

}
is_mac () {

    [[ "$(os_name)" == "mac" ]]

}
run () {

    (( $# )) || return 0

    if (( VERBOSE_ENV )); then

        local s="" a="" q=""

        for a in "$@"; do

            q="$(printf '%q' "${a}")"

            if [[ -z "${s}" ]]; then
                s="${q}"
            else
                s="${s} ${q}"
            fi

        done

        printf '%s\n' "+ ${s}" >&2

    fi

    "$@"

}
has () {

    local cmd="${1:-}"
    [[ -n "${cmd}" ]] || return 1

    command -v -- "${cmd}" >/dev/null 2>&1

}
path_prepend () {

    local d="${1-}"
    [[ -n "${d}" ]] || return 0

    if [[ -z "${PATH-}" ]]; then
        PATH="${d}"
        export PATH
        return 0
    fi

    local -a parts=()
    local entry="" rest=""

    IFS=':' read -r -a parts <<< "${PATH}"

    for entry in "${parts[@]}"; do
        [[ "${entry}" == "${d}" ]] && continue
        rest="${rest}:${entry}"
    done

    PATH="${d}${rest}"
    export PATH

    return 0

}
unix_path () {

    local p="${1-}"
    [[ -n "${p}" ]] || return 0

    [[ "$(os_name)" == "windows" ]] || { printf '%s' "${p}"; return 0; }

    case "${p}" in
        [A-Za-z]:[/\\]*|*'\'*) ;;
        *) printf '%s' "${p}"; return 0 ;;
    esac

    has cygpath || die "unix_path: cygpath is required to translate a Windows path: ${p}" 2

    local out=""
    out="$(cygpath -u -- "${p}" 2>/dev/null)" || die "unix_path: cygpath failed for: ${p}" 2

    printf '%s' "${out}"

}
open_path () {

    local p="${1:-}"

    [[ -n "${p}" ]] || die "open_path: missing path" 2
    [[ -e "${p}" ]] || die "open_path: not found: ${p}" 2

    if is_wsl; then

        if has wslview; then
            run wslview "${p}"
            return 0
        fi
        if has explorer.exe; then
            run explorer.exe "$(wslpath -w "${p}" 2>/dev/null || printf '%s' "${p}")"
            return 0
        fi

    fi

    case "$(os_name)" in
        mac)
            has open || die "open_path: 'open' not found" 2
            run open "${p}"
        ;;
        linux)
            has xdg-open || die "open_path: 'xdg-open' not found" 2
            run xdg-open "${p}"
        ;;
        windows)
            if has cygstart; then
                run cygstart "${p}"
                return 0
            fi

            has cygpath || die "open_path: need cygstart or cygpath to open: ${p}" 2

            local wp=""
            wp="$(cygpath -w -- "${p}" 2>/dev/null)" || die "open_path: cygpath failed for: ${p}" 2

            has cmd.exe || die "open_path: cmd.exe not found" 2
            run cmd.exe /c start "" "${wp}" >/dev/null 2>&1 || die "open_path: failed to open: ${p}" 2
        ;;
        *)
            die "open_path: unsupported OS" 2
        ;;
    esac

}
err_dispatch () {

    local code="${1:-1}"
    local cmd="${2-}"
    local file="${3-}"
    local line="${4-}"

    trap - ERR
    (( code )) || code=1

    if [[ -n "${ERR_HANDLER-}" ]] && declare -F -- "${ERR_HANDLER}" >/dev/null 2>&1; then
        "${ERR_HANDLER}" "${code}" "${cmd}" "${file}" "${line}" || true
    else
        error "exit ${code} at ${file:-?}:${line:-?}: ${cmd:-?}"
    fi

    if [[ "${-}" == *i* && "${BASH_SOURCE[0]-}" != "${0-}" ]]; then
        return "${code}" 2>/dev/null || exit "${code}"
    fi

    exit "${code}"

}
trap_on_err () {

    local code=$? cmd="${BASH_COMMAND-}"

    local file="${BASH_SOURCE[1]-}"
    local line="${BASH_LINENO[0]-}"

    [[ -n "${1-}" ]] && ERR_HANDLER="${1}"

    err_dispatch "${code}" "${cmd}" "${file}" "${line}"

}
on_err () {

    local cmd="${BASH_COMMAND-}" arg="${1-}"

    if [[ "${arg}" =~ ^[0-9]+$ ]]; then
        err_dispatch "${arg}" "${cmd}" "${BASH_SOURCE[1]-}" "${BASH_LINENO[0]-}"
        return $?
    fi

    [[ -z "${arg}" ]] || declare -F -- "${arg}" >/dev/null 2>&1 || die "on_err: not a function: ${arg}" 2

    ERR_HANDLER="${arg}"

    set -E
    trap 'trap_on_err' ERR

    return 0

}

os_name >/dev/null
is_wsl || true
