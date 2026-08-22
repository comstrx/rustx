#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "loader.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${LOADER_LOADED:-}" ]] && return 0
LOADER_LOADED=1

__dir__="${BASH_SOURCE[0]%/*}"
[[ "${__dir__}" == "${BASH_SOURCE[0]}" ]] && __dir__="."
__dir__="$(cd -- "${__dir__}" && pwd -P)"
source "${__dir__}/boot.sh"
source "${__dir__}/help.sh"

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

    local file="" base="" name="" subdir="" sd="" rc=0
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

                [[ -n "${_seen[${file}]-}" ]] && continue
                _seen["${file}"]=1

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
                    rc=$?
                    (( nullglob_was_set )) || shopt -u nullglob
                    return "${rc}"
                }
            ;;
            doc)
                should_skip "${base}" && continue
                load_walk doc "${sd}" "${seen_ref}" "${mods_ref}" || {
                    rc=$?
                    (( nullglob_was_set )) || shopt -u nullglob
                    return "${rc}"
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
        h)
            local -a mods=()
            local -A seen=()

            load_walk doc "${MODULE_DIR:?dispatch: MODULE_DIR not set}" seen mods || return 2
            render_doc "${mods[@]-}"

            return $?
        ;;
        v) cmd_version; return $? ;;
    esac

    if ! [[ "${cmd}" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]]; then
        die "Unknown command: ( ${cmd} )"$'\n'"See Docs: --help" 2
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

    if [[ -n "${sub}" && "${sub}" != -* ]] && compgen -A function "cmd_${mod}_" >/dev/null 2>&1; then

        local docs="--help"
        declare -F "cmd_${mod}_help" >/dev/null 2>&1 && docs="${cmd}-help"

        die "Unknown subcommand: ( ${sub} ) for ( ${cmd} )"$'\n'"See Docs: ${docs}" 2

    fi

    die "Unknown command: ( ${cmd} )"$'\n'"See Docs: --help" 2

}
report_err () {

    local code="${1-}" cmd="${2-}" file="${3-}" line="${4-}"

    [[ "${code}" =~ ^[0-9]+$ ]] && (( code )) || code=1

    printf '%s\n' "❌ Failed ( exit ${code} ) : ${file##*/}:${line}: ${cmd}" >&2

    exit "${code}"

}
load () {

    cd_root

    local old_trap=""
    old_trap="$(trap -p ERR 2>/dev/null || true)"

    if declare -F on_err >/dev/null 2>&1; then on_err report_err; fi

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
