#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "help.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${HELP_LOADED:-}" ]] && return 0
HELP_LOADED=1

SORTED_LIST=( notify crate lint perf safety doctor meta ci )

module_usage () {

    local name="${1-}"
    [[ -n "${name}" ]] || return 0

    local mod="" chosen=""
    mod="${name//-/_}"
    mod="${mod//./_}"

    declare -F "cmd_${mod}_help" >/dev/null 2>&1 && chosen="cmd_${mod}_help"
    [[ -z "${chosen}" ]] && declare -F "${mod}_help" >/dev/null 2>&1 && chosen="${mod}_help"
    [[ -n "${chosen}" ]] || return 0

    "${chosen}" || true

}
render_doc () {

    local -a mods=( "$@" )
    local -A printed_mod=()
    local -A found=()
    local name="" want="" printed=0

    for name in "${mods[@]-}"; do
        [[ -n "${name}" ]] && found["${name}"]=1
    done

    for want in "${SORTED_LIST[@]-}"; do
        [[ -z "${want}" || -n "${found[${want}]-}" ]] || die "render_doc: stale SORTED_LIST entry '${want}': no module found" 2
    done

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
help_cmd_to_fn () {

    local cmd="${1-}"

    cmd="${cmd//-/_}"
    printf '%s' "cmd_${cmd}"

}
help_humanize_name () {

    local s="${1-}"

    s="${s//_/ }"
    s="${s//-/ }"
    s="${s^}"

    printf '%s' "${s}"

}
help_extract_parse_schema () {

    local def=""
    def="$(declare -f "${1}" 2>/dev/null)" || return 1

    local out=""
    out="$(
        awk '
            BEGIN { on=0 }
            {
                if (!on) {
                    if ($0 ~ /source[[:space:]]*<\([[:space:]]*parse[[:space:]]*"\$@"[[:space:]]*--/) {
                        on=1
                        sub(/.*parse[[:space:]]*"\$@"[[:space:]]*--[[:space:]]*/, "", $0)
                    }
                    else if ($0 ~ /parse[[:space:]]*"\$@"[[:space:]]*--/) {
                        on=1
                        sub(/.*parse[[:space:]]*"\$@"[[:space:]]*--[[:space:]]*/, "", $0)
                    }
                    else {
                        next
                    }
                }
                if (on) {
                    print
                    if (index($0, ")") > 0) { exit }
                }
            }
        ' <<<"${def}"
    )"

    out="$(tr '\n' ' ' <<<"${out}")"
    out="${out%%)*}"
    out="${out//\\/}"

    out="${out#"${out%%[![:space:]]*}"}"
    out="${out%"${out##*[![:space:]]}"}"

    printf '%s' "${out}"

}
help_decode_token () {

    local tok="${1-}"
    local -n _name="${2}"
    local -n _type="${3}"
    local -n _def="${4}"

    local raw="${tok#:}" rest=""

    _name="${raw%%[:=]*}"
    [[ -n "${_name}" ]] || return 1

    rest="${raw#"${_name}"}"

    _type="str"
    _def=""

    if [[ "${rest}" == :* ]]; then
        rest="${rest#:}"
        _type="${rest%%=*}"
        rest="${rest#"${_type}"}"
        [[ -n "${_type}" ]] || _type="str"
    fi
    if [[ "${rest}" == =* ]]; then
        _def="${rest#=}"
        _def="${_def%\"}"; _def="${_def#\"}"
        _def="${_def%\'}"; _def="${_def#\'}"
    fi

    return 0

}
help_print_section () {

    local kind="${1-}" label="${2-}"
    shift 2 || true

    local pad=23 shown=0
    local tok="" name="" type="" defv="" human="" left="" desc="" d=""

    for tok in "$@"; do

        [[ -n "${tok}" ]] || continue
        [[ "${tok}" == "--" ]] && continue

        if [[ "${kind}" == "args" ]]; then
            [[ "${tok}" == :* ]] || continue
        else
            [[ "${tok}" == :* ]] && continue
        fi

        help_decode_token "${tok}" name type defv || continue

        if [[ "${kind}" == "flags" ]]; then
            [[ "${type}" == "bool" ]] || continue
        else
            [[ "${type}" == "bool" ]] && continue
        fi

        human="$(help_humanize_name "${name}")"

        if [[ "${kind}" == "args" ]]; then
            left="<${name//_/-}>"
            [[ "${type}" == "list" ]] && left="<${name//_/-}...>"
        elif (( ${#name} == 1 )); then
            left="-${name}"
        else
            left="--${name//_/-}"
        fi

        case "${kind}" in
            args)
                desc="${human}"

                if [[ -n "${defv}" ]]; then
                    desc+=" ( Default: ${defv} )"
                else
                    desc+=" ( Required )"
                fi

                desc+=" ( Type: '${type}' )"
            ;;
            options)
                desc="${human}"
                [[ "${type}" == "list" ]] && desc+=" (repeatable)"

                desc+=" ( Optional )"
                [[ -n "${defv}" ]] && desc+=" ( Default: ${defv} )"

                desc+=" ( Type: '${type}' )"
            ;;
            flags)
                d="false"

                if [[ -n "${defv}" ]]; then
                    case "${defv,,}" in
                        1|true|yes|on) d="true" ;;
                        0|false|no|off) d="false" ;;
                        *) d="${defv}" ;;
                    esac
                fi

                desc="${human} ( Optional ) ( Default: ${d} ) ( Type: '${type}' )"
            ;;
            *)
                die "help_print_section: unknown kind '${kind}'" 2
            ;;
        esac

        (( shown )) || { log_info "\n    ${label}:\n"; shown=1; }
        printf '        %-*s %s\n' "${pad}" "${left}" "${desc}"

    done

    return 0

}
help_print_generated_help () {

    local schema="${1-}"

    [[ -n "${schema}" ]] || { printf '    %s\n' ""; printf '    %s\n\n' "No args detected (no parse schema)."; return 0; }

    local -a tokens=()
    read -r -a tokens <<< "${schema}"

    help_print_section args    "Args"    "${tokens[@]-}"
    help_print_section options "Options" "${tokens[@]-}"
    help_print_section flags   "Flags"   "${tokens[@]-}"

    printf '\n'

}
cmd_help () {

    source <(parse "$@" -- command)

    if [[ -z "${command}" ]]; then

        info_ln "Help :"

        printf '    %s\n' \
            "Usage:" \
            "    rustx help <command>" \
            "" \
            "Example:" \
            "    rustx help fmt-check" \
            ""

        return 0

    fi

    local fn=""
    fn="$(help_cmd_to_fn "${command}")"

    declare -F "${fn}" >/dev/null 2>&1 || die "Unknown command: ${command}" 2

    info_ln "Help : ( ${command} )"

    local schema=""
    schema="$(help_extract_parse_schema "${fn}" 2>/dev/null || true)"

    help_print_generated_help "${schema}"

}
cmd_source () {

    source <(parse "$@" -- :command lines:bool=true)

    info_ln "Source :\n"
    log_info "    Code :\n"

    local fn=""
    fn="$(help_cmd_to_fn "${command}")"

    declare -F "${fn}" >/dev/null 2>&1 || die "Unknown command: ${command}" 2

    if (( lines )); then declare -f "${fn}" | nl -ba
    else declare -f "${fn}"
    fi

    printf '\n'

}
