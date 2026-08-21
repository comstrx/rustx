#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "loader.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${LOADER_LOADED:-}" ]] && return 0
LOADER_LOADED=1

__dir__="${BASH_SOURCE[0]%/*}"
[[ "${__dir__}" == "${BASH_SOURCE[0]}" ]] && __dir__="."
__dir__="$(cd -- "${__dir__}" && pwd -P)"
source "${__dir__}/boot.sh"
source "${__dir__}/help.sh"

SORTED_LIST=(cinema notify git github crate lint perf safety doctor meta ci)

should_skip () {

    local name="${1-}" s=""
    shift || true

    [[ -n "${name}" ]] || return 0
    [[ "${name}" == _* ]] && return 0

    for s in "$@"; do
        [[ -n "${s}" ]] || continue
        [[ "${name}" == "${s}" ]] && return 0
    done

    return 1

}
load_walk () {

    local mode="${1-}"
    local dir="${2-}"
    local seen_ref="${3-}"
    local mods_ref="${4-}"

    shift 4 || true

    [[ -n "${mode}" ]] || { die "load_walk: missing mode" 2; return 2; }
    [[ -n "${dir}" && -d "${dir}" ]] || return 0

    if [[ "${mode}" == "doc" ]]; then
        [[ -n "${seen_ref}" && -n "${mods_ref}" ]] || { die "load_walk: doc requires seen/mods refs" 2; return 2; }
        local -n _seen="${seen_ref}"
        local -n _mods="${mods_ref}"
    fi

    local nullglob_was_set=0

    shopt -q nullglob && nullglob_was_set=1
    shopt -s nullglob

    local file="" base="" name="" subdir="" sd=""
    local -a extra_skip=()

    (( $# > 0 )) && extra_skip=( "$@" ) || extra_skip=()

    for file in "${dir}"/*.sh; do

        base="${file##*/}"
        [[ -n "${base}" ]] || continue
        name="${base%.sh}"

        case "${mode}" in
            source)
                should_skip "${name}" "${extra_skip[@]-}" && continue

                source "${file}" || {
                    (( nullglob_was_set )) || shopt -u nullglob
                    die "Failed to source: ${file}" 2
                    return 2
                }
            ;;
            doc)
                should_skip "${name}" && continue

                [[ -n "${_seen[${name}]-}" ]] && continue
                _seen["${name}"]=1

                _mods+=( "${name}" )
            ;;
            *)
                (( nullglob_was_set )) || shopt -u nullglob
                die "load_walk: unknown mode '${mode}'" 2
                return 2
            ;;
        esac

    done

    for subdir in "${dir}"/*/; do

        sd="${subdir%/}"
        [[ -L "${sd}" ]] && continue

        base="${sd##*/}"
        [[ -n "${base}" ]] || continue

        case "${mode}" in
            source)
                should_skip "${base}" "${extra_skip[@]-}" && continue
                load_walk source "${sd}" "" "" "${extra_skip[@]-}" || {
                    (( nullglob_was_set )) || shopt -u nullglob
                    return $?
                }
            ;;
            doc)
                should_skip "${base}" && continue
                load_walk doc "${sd}" "${seen_ref}" "${mods_ref}" || {
                    (( nullglob_was_set )) || shopt -u nullglob
                    return $?
                }
            ;;
        esac

    done

    (( nullglob_was_set )) || shopt -u nullglob
    return 0

}
load_source () {

    [[ -n "${MODULES_LOADED-}" ]] && return 0
    MODULES_LOADED=1

    local dir="${1-}"

    if [[ -z "${dir}" ]]; then
        [[ -n "${MODULE_DIR:-}" ]] || { die "load_source: MODULE_DIR not set" 2; return 2; }
        dir="${MODULE_DIR}"
    fi

    [[ -d "${dir}" ]] || { die "load_source: not a dir: ${dir}" 2; return 2; }

    local -a extra_skip=()
    (( $# > 1 )) && extra_skip=( "${@:2}" ) || extra_skip=()

    load_walk source "${dir}" "" "" "${extra_skip[@]-}"

}
module_usage () {

    local name="${1-}"
    [[ -n "${name}" ]] || return 0

    local mod="" chosen=""
    mod="${name//-/_}"
    mod="${mod//./_}"

    local fn1="${mod}_usage"
    local fn2="help_${mod}"
    local fn3="${mod}_help"
    local fn4="usage_${mod}"
    local fn5="cmd_${mod}_usage"
    local fn6="cmd_help_${mod}"
    local fn7="cmd_${mod}_help"
    local fn8="cmd_usage_${mod}"

    declare -F "${fn1}" >/dev/null 2>&1 && chosen="${fn1}"
    [[ -z "${chosen}" ]] && declare -F "${fn2}" >/dev/null 2>&1 && chosen="${fn2}"
    [[ -z "${chosen}" ]] && declare -F "${fn3}" >/dev/null 2>&1 && chosen="${fn3}"
    [[ -z "${chosen}" ]] && declare -F "${fn4}" >/dev/null 2>&1 && chosen="${fn4}"
    [[ -z "${chosen}" ]] && declare -F "${fn5}" >/dev/null 2>&1 && chosen="${fn5}"
    [[ -z "${chosen}" ]] && declare -F "${fn6}" >/dev/null 2>&1 && chosen="${fn6}"
    [[ -z "${chosen}" ]] && declare -F "${fn7}" >/dev/null 2>&1 && chosen="${fn7}"
    [[ -z "${chosen}" ]] && declare -F "${fn8}" >/dev/null 2>&1 && chosen="${fn8}"
    [[ -n "${chosen}" ]] || return 0

    "${chosen}" || true

}
render_doc () {

    local dir="${MODULE_DIR:-}"
    [[ -n "${dir}" ]] || { die "render_doc: MODULE_DIR not set" 2; return 2; }

    local -a mods=()
    local -A seen=()
    local -A printed_mod=()
    local name="" mod="" chosen="" want="" printed=0

    load_source "${dir}" || return $?
    load_walk doc "${dir}" seen mods || return 2

    info_ln "Usage:\n"
    printf '%s\n' \
        "    rustx [--yes] [--quiet] [--verbose] <cmd> [args...]" \
        ''

    info_ln "Global:\n"
    printf '%s\n' \
        '    --yes,    -y     Non-interactive (assume yes)' \
        '    --quiet,  -q     Less output' \
        '    --verbose,-v     Print executed commands' \
        ''

    for want in "${SORTED_LIST[@]-}"; do

        [[ -n "${want}" ]] || continue

        for name in "${mods[@]-}"; do

            [[ "${name}" == "${want}" ]] || continue

            module_usage "${name}"

            printed=1
            printed_mod["${name}"]=1
            break

        done

    done
    for name in "${mods[@]-}"; do

        [[ -n "${name}" ]] || continue
        [[ -n "${printed_mod[${name}]-}" ]] && continue

        module_usage "${name}"

        printed=1
        printed_mod["${name}"]=1

    done

    (( printed )) || printf '%s\n' '(no module usage found)' ''

}
parse_global () {

    MODULE_CMD="h"
    MODULE_ARGS=()

    local saw_help=0
    local saw_version=0

    while [[ $# -gt 0 ]]; do
        case "${1}" in
            -h|--help)      saw_help=1; shift || true ;;
            --version)      saw_version=1; shift || true ;;
            --yes|-y)       YES_ENV=1; shift || true ;;
            --quiet|-q)     QUIET_ENV=1; shift || true ;;
            --verbose|-v)   VERBOSE_ENV=1; shift || true ;;
            --)             shift || true; break ;;
            -*)             die "Unknown global flag: ${1}" 2 ;;
            *)              break ;;
        esac
    done

    (( saw_help )) && { MODULE_CMD="h"; MODULE_ARGS=(); return 0; }
    (( saw_version )) && { MODULE_CMD="v"; MODULE_ARGS=(); return 0; }

    if (( $# > 0 )); then
        MODULE_CMD="${1}"
        shift || true
        MODULE_ARGS=( "$@" )
    fi

}
dispatch () {

    local cmd="${1:-}"
    shift || true

    case "${cmd}" in
        h) render_doc; return 0 ;;
        v) echo "v0.1.0"; return 0 ;;
    esac

    if ! [[ "${cmd}" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]]; then
        eprint "Unknown command: ( ${cmd} )"
        eprint "See Docs: --help"
        return 2
    fi

    local sub="${1-}" mod="" fn=""
    mod="${cmd//-/_}"
    mod="${mod//./_}"

    if [[ -n "${sub}" && "${sub}" != -* ]]; then

        fn="cmd_${mod}_${sub//-/_}"
        fn="${fn//./_}"

        if declare -F "${fn}" >/dev/null 2>&1; then
            shift || true
            "${fn}" "$@"
            return $?
        fi

    fi

    fn="cmd_${mod}"
    fn="${fn//./_}"

    if declare -F "${fn}" >/dev/null 2>&1; then
        "${fn}" "$@"
        return $?
    fi

    eprint "Unknown command: ( ${cmd} )"
    eprint "See Docs: --help"
    return 2

}
load () {

    cd_root

    local old_trap="$(trap -p ERR 2>/dev/null || true)"
    if declare -F on_err >/dev/null 2>&1; then trap 'on_err "$?"' ERR; fi

    local dir="${MODULE_DIR:-}"
    [[ -n "${dir}" ]] || { die "load: MODULE_DIR not set" 2; return 2; }

    load_source "${dir}" || return $?
    parse_global "$@"

    local ec=0

    if [[ ${MODULE_ARGS[0]+x} ]]; then
        dispatch "${MODULE_CMD}" "${MODULE_ARGS[@]}"
        ec=$?
    else
        dispatch "${MODULE_CMD}"
        ec=$?
    fi

    if [[ -n "${old_trap}" ]]; then
        eval "${old_trap}"
    else
        trap - ERR 2>/dev/null || true
    fi

    return "${ec}"

}
