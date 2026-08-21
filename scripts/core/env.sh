#!/usr/bin/env bash

[[ -n "${BASH_VERSION:-}" ]] || { printf '%s\n' "env.sh: Bash required." >&2; return 2 2>/dev/null || exit 2; }
(( ${BASH_VERSINFO[0]:-0} >= 4 )) || { printf '%s\n' "env.sh: Bash 5+ required." >&2; return 2 2>/dev/null || exit 2; }

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "env.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${ENV_LOADED:-}" ]] && return 0
ENV_LOADED=1

YES_ENV="${YES_ENV:-0}"
QUIET_ENV="${QUIET_ENV:-0}"
VERBOSE_ENV="${VERBOSE_ENV:-0}"

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
info () {

    (( QUIET_ENV )) && return 0

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre}💥 $*${suf}" >&2

}
success () {

    (( QUIET_ENV )) && return 0

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre}✅ $*${suf}" >&2

}
warn () {

    (( QUIET_ENV )) && return 0

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre}⚠️ $*${suf}" >&2

}
error () {

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre}❌ $*${suf}" >&2

}
info_ln () {

    (( QUIET_ENV )) && return 0

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '\n%b\n' "${pre}💥 $*${suf}" >&2

}
success_ln () {

    (( QUIET_ENV )) && return 0

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '\n%b\n' "${pre}✅ $*${suf}" >&2

}
warn_ln () {

    (( QUIET_ENV )) && return 0

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '\n%b\n' "${pre}⚠️ $*${suf}" >&2

}
error_ln () {

    local pre="$(colorize reset 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '\n%b\n' "${pre}❌ $*${suf}" >&2

}
log () {

    local IFS=' '
    (( $# )) || { printf '\n' >&2; return 0; }
    printf '%b\n' "$*" >&2

}
log_success () {

    local pre="$(colorize green 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre} $*${suf}" >&2

}
log_info () {

    local pre="$(colorize blue 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre} $*${suf}" >&2

}
log_warn () {

    local pre="$(colorize yellow 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre} $*${suf}" >&2

}
log_error () {

    local pre="$(colorize red 2)"
    local suf="$(colorize reset 2)"

    local IFS=' '
    printf '%b\n' "${pre} $*${suf}" >&2

}
print () {

    local IFS=' '

    if (( $# == 0 )); then
        printf '\n'
        return 0
    fi

    printf '%b\n' "$*"

}
eprint () {

    local IFS=' '

    if (( $# == 0 )); then
        printf '\n' >&2
        return 0
    fi

    printf '%b\n' "$*" >&2

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
input_bool () {

    local prompt="${1-}"
    local def="${2-}"
    local tries="${3:-3}"

    local def_norm=""
    case "${def}" in
        1|true|TRUE|True|yes|YES|Yes|y|Y) def_norm="1" ;;
        0|false|FALSE|False|no|NO|No|n|N) def_norm="0" ;;
    esac

    local v="" i=0

    for (( i=0; i<tries; i++ )); do

        v="$(input "${prompt}" "${def}")" || return $?

        case "${v}" in
            1|true|TRUE|True|yes|YES|Yes|y|Y) printf '1'; return 0 ;;
            0|false|FALSE|False|no|NO|No|n|N) printf '0'; return 0 ;;
            "") [[ -n "${def_norm}" ]] && { printf '%s' "${def_norm}"; return 0; } ;;
        esac

        eprint "Invalid bool. Use: y/n, yes/no, 1/0, true/false"

    done

    die "input_bool: too many invalid attempts" 2

}
input_int () {

    local prompt="${1-}"
    local def="${2-}"
    local tries="${3:-3}"

    local v="" i=0

    for (( i=0; i<tries; i++ )); do

        v="$(input "${prompt}" "${def}")" || return $?

        [[ -z "${v}" && -n "${def}" ]] && v="${def}"

        if [[ "${v}" =~ ^-?[0-9]+$ ]]; then
            printf '%s' "${v}"
            return 0
        fi

        eprint "Invalid int. Example: 0, 12, -7"

    done

    die "input_int: too many invalid attempts" 2

}
input_uint () {

    local prompt="${1-}"
    local def="${2-}"
    local tries="${3:-3}"

    local v="" i=0

    for (( i=0; i<tries; i++ )); do

        v="$(input "${prompt}" "${def}")" || return $?

        [[ -z "${v}" && -n "${def}" ]] && v="${def}"

        if [[ "${v}" =~ ^[0-9]+$ ]]; then
            printf '%s' "${v}"
            return 0
        fi

        eprint "Invalid uint. Example: 0, 12, 7"

    done

    die "input_uint: too many invalid attempts" 2

}
input_float () {

    local prompt="${1-}"
    local def="${2-}"
    local tries="${3:-3}"

    local v="" i=0

    for (( i=0; i<tries; i++ )); do

        v="$(input "${prompt}" "${def}")" || return $?

        [[ -z "${v}" && -n "${def}" ]] && v="${def}"

        if [[ "${v}" =~ ^[+-]?([0-9]+([.][0-9]+)?|[.][0-9]+)$ ]]; then
            printf '%s' "${v}"
            return 0
        fi

        eprint "Invalid float. Example: 0, 12.5, -7, .3"

    done

    die "input_float: too many invalid attempts" 2

}
input_char () {

    local prompt="${1-}"
    local def="${2-}"
    local tries="${3:-3}"

    local v="" i=0

    for (( i=0; i<tries; i++ )); do

        v="$(input "${prompt}" "${def}")" || return $?

        [[ -z "${v}" && -n "${def}" ]] && v="${def}"

        if (( ${#v} == 1 )); then
            printf '%s' "${v}"
            return 0
        fi

        eprint "Invalid char. Example: a"

    done

    die "input_char: too many invalid attempts" 2

}
input_pass () {

    local prompt="${1-}"
    local tty="/dev/tty"
    local line=""

    [[ -c "${tty}" && -r "${tty}" && -w "${tty}" ]] || die "input_pass: no /dev/tty (cannot securely read password)" 2

    [[ -n "${prompt}" ]] && printf '%b' "${prompt}" >"${tty}"

    if IFS= read -r -s line <"${tty}" 2>/dev/null; then
        printf '\n' >"${tty}"
        printf '%s' "${line}"
        return 0
    fi

    command -v stty >/dev/null 2>&1 || die "input_pass: cannot disable echo (read -s failed, stty missing)" 2

    local stty_old="$(stty -g <"${tty}" 2>/dev/null || true)"
    local old_int="$(trap -p INT 2>/dev/null || true)"
    local old_term="$(trap -p TERM 2>/dev/null || true)"
    local old_return="$(trap -p RETURN 2>/dev/null || true)"
    local abort=0 rc=0

    __input_pass_restore () {

        [[ -n "${stty_old}" ]] && stty "${stty_old}" <"${tty}" 2>/dev/null || stty echo <"${tty}" 2>/dev/null || true

        [[ -n "${old_int}" ]] && eval "${old_int}" || trap - INT
        [[ -n "${old_term}" ]] && eval "${old_term}" || trap - TERM
        [[ -n "${old_return}" ]] && eval "${old_return}" || trap - RETURN

    }

    __input_pass_abort_int () { abort=130; __input_pass_restore; }
    __input_pass_abort_term () { abort=143; __input_pass_restore; }

    trap '__input_pass_abort_int' INT
    trap '__input_pass_abort_term' TERM
    trap '__input_pass_restore' RETURN

    stty -echo <"${tty}" 2>/dev/null || true

    rc=0
    IFS= read -r line <"${tty}" || rc=$?

    __input_pass_restore

    unset -f __input_pass_restore __input_pass_abort_int __input_pass_abort_term 2>/dev/null || true

    printf '\n' >"${tty}"

    (( abort )) && return "${abort}"
    (( rc != 0 )) && return "${rc}"

    printf '%s' "${line}"

}
input_path () {

    local prompt="${1-}"
    local def="${2-}"
    local mode="${3:-any}"
    local tries="${4:-3}"

    local p="" i=0

    for (( i=0; i<tries; i++ )); do

        p="$(input "${prompt}" "${def}")" || return $?

        [[ -z "${p}" && -n "${def}" ]] && p="${def}"
        [[ -n "${p}" ]] || { eprint "Path is required"; continue; }

        case "${mode}" in
            any) printf '%s' "${p}"; return 0 ;;
            exists) [[ -e "${p}" ]] && { printf '%s' "${p}"; return 0; } ;;
            file) [[ -f "${p}" ]] && { printf '%s' "${p}"; return 0; } ;;
            dir) [[ -d "${p}" ]] && { printf '%s' "${p}"; return 0; } ;;
            *) die "input_path: invalid mode '${mode}'" 2 ;;
        esac

        eprint "Invalid path for mode '${mode}': ${p}"

    done

    die "input_path: too many invalid attempts" 2

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

    local ans="$(input "${msg} ${hint}" "${def}")" || return $?

    case "${ans}" in
        y|Y|yes|YES|Yes|yep|Yep|YEP|1|true|TRUE|True) return 0 ;;
        n|N|no|NO|No|0|false|FALSE|False) return 1 ;;
        "") (( d_is_yes )) && return 0 || return 1 ;;
    esac

    return 1

}
confirm_bool () {

    if confirm "$@"; then
        printf '1'
        return 0
    fi

    printf '0'
    return 1

}
choose () {

    local prompt="${1:-Choose:}"
    shift || true

    local -a items=( "$@" )
    (( ${#items[@]} )) || die "choose: missing items" 2

    local i=0
    eprint "${prompt}"
    for (( i=0; i<${#items[@]}; i++ )); do
        eprint "  $((i+1))) ${items[$i]}"
    done

    local pick=""
    pick="$(input "Enter number [1-${#items[@]}]: ")" || return $?

    [[ "${pick}" =~ ^[0-9]+$ ]] || die "choose: invalid number" 2
    (( pick >= 1 && pick <= ${#items[@]} )) || die "choose: out of range" 2

    printf '%s' "${items[$((pick-1))]}"

}
current_date () {

    LC_ALL=C command date '+%Y-%m-%d'

}
current_time () {

    LC_ALL=C command date '+%H:%M:%S'

}
current_datetime () {

    LC_ALL=C command date '+%Y-%m-%d %H:%M:%S'

}
cd_root () {

    cd -- "${ROOT_DIR}"

}
os_name () {

    local u=""
    u="$(uname -s 2>/dev/null || printf '%s' unknown)"

    case "${u}" in
        Linux)   printf '%s' linux ;;
        Darwin)  printf '%s' mac ;;
        MSYS*|MINGW*|CYGWIN*) printf '%s' windows ;;
        *)       printf '%s' unknown ;;
    esac

}
get_env () {

    local key="${1:-}"
    local def="${2-}"

    [[ -n "${key}" ]] || { printf '%s' "${def}"; return 0; }
    [[ "${key}" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || { printf '%s' "${def}"; return 0; }

    if [[ -n "${!key+x}" ]]; then
        printf '%s' "${!key}"
    else
        printf '%s' "${def}"
    fi

}
is_main () {

    [[ "${BASH_SOURCE[0]-}" == "$0" ]]

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

    [[ -n "${WSL_INTEROP:-}" || -n "${WSL_DISTRO_NAME:-}" ]] && return 0

    [[ -r /proc/version ]] && grep -qiE 'microsoft|wsl' /proc/version 2>/dev/null && return 0
    [[ -r /proc/sys/kernel/osrelease ]] && grep -qiE 'microsoft|wsl' /proc/sys/kernel/osrelease 2>/dev/null && return 0

    return 1

}
is_linux () {

    [[ "$(os_name)" == "linux" ]]

}
is_mac () {

    [[ "$(os_name)" == "mac" ]]

}
is_windows () {

    [[ "$(os_name)" == "windows" ]]

}
abs_dir () {

    local p="${1:-}"
    local d=""

    if [[ -z "${p}" ]]; then
        pwd -P
        return 0
    fi

    if [[ -d "${p}" ]]; then
        d="${p}"
    else
        d="$(dirname -- "${p}")"
    fi

    ( cd -- "${d}" 2>/dev/null && pwd -P ) || return 1

}
uc_first () {

    local s="${1:-}"
    [[ -n "${s}" ]] || { printf '%s' ""; return 0; }

    printf '%s%s' "$(printf '%s' "${s:0:1}" | tr '[:lower:]' '[:upper:]')" "${s:1}"

}
unique_list () {

    local -n in="${1}"
    local -a out=()
    local -A seen=()

    local x=""

    for x in "${in[@]-}"; do

        [[ -n "${x}" ]] || continue
        [[ -n "${seen["$x"]+x}" ]] && continue

        seen["$x"]=1
        out+=( "$x" )

    done

    in=( "${out[@]}" )

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
ln_sf () {

    local src="${1:-}"
    local dst="${2:-}"

    [[ -n "${src}" && -n "${dst}" ]] || die "ln_sf: usage: ln_sf <src> <dst>" 2

    rm -rf "${dst}" 2>/dev/null || true
    ln -s "${src}" "${dst}" 2>/dev/null && return 0

    if [[ -d "${src}" ]]; then
        cp -R "${src}" "${dst}"
    else
        cp "${src}" "${dst}"
    fi

}
slugify () {

    local s="${1-}"
    s="${s//[[:space:]]/-}"

    while [[ "${s}" == *"--"* ]]; do
        s="${s//--/-}"
    done

    s="${s#-}"
    s="${s%-}"

    printf '%s' "${s}"

}
is_valid_alias () {

    local a="${1:-}"

    [[ -n "${a}" ]] || return 1
    [[ "${a}" != *"/"* && "${a}" != *"\\"* ]] || return 1
    [[ "${a}" =~ ^[A-Za-z_][A-Za-z0-9_-]*$ ]]

}
rc_path () {

    local shell="${SHELL:-}" home="${HOME:-}" kind="sh" os="$(os_name)"

    if [[ -z "${home}" ]]; then
        home="$(cd ~ 2>/dev/null && pwd || printf '%s' ".")"
    fi

    if [[ -n "${ZSH_VERSION:-}" || "${shell}" == */zsh ]]; then kind="zsh"
    elif [[ -n "${BASH_VERSION:-}" || "${shell}" == */bash ]]; then kind="bash"
    elif [[ -n "${FISH_VERSION:-}" || "${shell}" == */fish ]]; then kind="fish"
    elif [[ "${shell}" == */ksh ]]; then kind="ksh"
    fi

    if [[ "${kind}" == "zsh" ]]; then
        printf '%s\n' "${home}/.zshrc"
        return 0
    fi
    if [[ "${kind}" == "fish" ]]; then
        printf '%s\n' "${home}/.config/fish/config.fish"
        return 0
    fi
    if [[ "${kind}" == "ksh" ]]; then
        printf '%s\n' "${home}/.kshrc"
        return 0
    fi
    if [[ "${kind}" == "bash" ]]; then

        local bashrc="${home}/.bashrc"

        if [[ "${os}" == "linux" ]]; then
            printf '%s\n' "${bashrc}"
            return 0
        fi
        if [[ "${os}" == "windows" ]]; then
            printf '%s\n' "${bashrc}"
            return 0
        fi
        if [[ "${os}" == "mac" ]]; then

            local bash_profile="${home}/.bash_profile"
            local profile="${home}/.profile"
            local login="${bash_profile}"

            [[ -f "${bash_profile}" ]] && login="${bash_profile}"
            [[ -f "${bash_profile}" ]] || { [[ -f "${profile}" ]] && login="${profile}"; }

            if [[ -f "${login}" && -f "${bashrc}" ]]; then

                local line=""
                while IFS= read -r line || [[ -n "${line}" ]]; do

                    line="${line%%#*}"

                    case "${line}" in
                        *".bashrc"*)
                            case "${line}" in
                                *"source "*".bashrc"*|*". "*".bashrc"*) printf '%s\n' "${bashrc}"; return 0 ;;
                            esac
                        ;;
                    esac

                done < "${login}" 2>/dev/null || true

            fi

            printf '%s\n' "${login}"
            return 0

        fi

        printf '%s\n' "${bashrc}"
        return 0

    fi

    printf '%s\n' "${home}/.profile"

}
home_path () {

    local h="${HOME:-}"

    if [[ -n "${h}" ]]; then
        printf '%s' "${h}"
        return 0
    fi

    h="$(cd ~ 2>/dev/null && pwd || true)"
    [[ -n "${h}" ]] || die "HOME not set; cannot resolve home directory." 2

    printf '%s' "${h}"

}
open_path () {

    local p="${1:-}"

    [[ -n "${p}" ]] || die "open_path: missing path" 2
    [[ -e "${p}" ]] || die "open_path: not found: ${p}" 2

    if is_wsl; then

        if command -v wslview >/dev/null 2>&1; then
            run wslview "${p}"
            return 0
        fi
        if command -v explorer.exe >/dev/null 2>&1; then
            run explorer.exe "$(wslpath -w "${p}" 2>/dev/null || printf '%s' "${p}")"
            return 0
        fi

    fi

    case "$(os_name)" in
        mac)
            command -v open >/dev/null 2>&1 || die "open_path: 'open' not found" 2
            run open "${p}"
        ;;
        linux)
            command -v xdg-open >/dev/null 2>&1 || die "open_path: 'xdg-open' not found" 2
            run xdg-open "${p}"
        ;;
        windows)
            local wp="${p}"

            if command -v cygpath >/dev/null 2>&1; then
                wp="$(cygpath -w "${p}" 2>/dev/null || printf '%s' "${p}")"
            fi

            if command -v cygstart >/dev/null 2>&1; then
                run cygstart "${wp}"
            else
                cmd.exe /c start "" "${wp}" >/dev/null 2>&1 || die "open_path: failed to open: ${p}" 2
            fi
        ;;
        *)
            die "open_path: unsupported OS" 2
        ;;
    esac

}
trap_on_err () {

    local handler="${1:-}"
    local code="$?" || true
    local cmd="${BASH_COMMAND-}"
    local file="${BASH_SOURCE[1]-}"
    local line="${BASH_LINENO[0]-}"

    trap - ERR
    "${handler}" "${code}" "${cmd}" "${file}" "${line}" || true

    if [[ "${-}" == *i* && "${BASH_SOURCE[0]-}" != "${0-}" ]]; then
        return "${code}" 2>/dev/null || exit "${code}"
    fi

    exit "${code}"

}
on_err () {

    local handler="${1:-}"
    [[ -n "${handler}" ]] || die "on_err: missing handler function name" 2

    set -E
    trap 'trap_on_err "'"${handler}"'"' ERR

}
