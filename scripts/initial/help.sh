#!/usr/bin/env bash

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

    local def="$(declare -f "${1}" 2>/dev/null)" || return 1

    local out="$(
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
help_print_generated_help () {

    local cmd="${1-}" fn="${2-}" schema="${3-}"
    local pad_arg=23 pad_opt=23 pad_flag=23

    [[ -n "${schema}" ]] || { printf '    %s\n' ""; printf '    %s\n\n' "No args detected (no parse schema)."; return 0; }

    local -a tokens=()
    read -r -a tokens <<< "${schema}"

    local tok="" raw="" name="" rest="" type="" defv="" human="" left="" desc=""
    local has_args=0 has_opts=0 has_flags=0

    for tok in "${tokens[@]}"; do

        [[ -n "${tok}" ]] || continue
        [[ "${tok}" == "--" ]] && continue

        [[ "${tok}" == :* ]] || continue

        raw="${tok#:}"
        name="${raw%%[:=]*}"
        rest="${raw#${name}}"

        [[ -n "${name}" ]] || continue

        type="str"
        defv=""

        if [[ "${rest}" == :* ]]; then
            rest="${rest#:}"
            type="${rest%%=*}"
            rest="${rest#${type}}"
            [[ -n "${type}" ]] || type="str"
        fi
        if [[ "${rest}" == =* ]]; then
            defv="${rest#=}"
            defv="${defv%\"}"; defv="${defv#\"}"
            defv="${defv%\'}"; defv="${defv#\'}"
        fi

        [[ "${type}" == "bool" ]] && continue
        human="$(help_humanize_name "${name}")"

        left="<${name//_/-}>"
        [[ "${type}" == "list" ]] && left="<${name//_/-}...>"

        desc="${human}"

        if [[ -n "${defv}" ]]; then
            desc+=" ( Default: ${defv} )"
        else
            desc+=" ( Required )"
        fi

        desc+=" ( Type: '${type}' )"

        (( has_args )) || { log_info "\n    Args:\n"; has_args=1; }
        printf '        %-*s %s\n' "${pad_arg}" "${left}" "${desc}"

    done

    for tok in "${tokens[@]}"; do

        [[ -n "${tok}" ]] || continue
        [[ "${tok}" == "--" ]] && continue
        [[ "${tok}" == :* ]] && continue

        raw="${tok}"
        name="${raw%%[:=]*}"
        rest="${raw#${name}}"

        [[ -n "${name}" ]] || continue

        type="str"
        defv=""

        if [[ "${rest}" == :* ]]; then
            rest="${rest#:}"
            type="${rest%%=*}"
            rest="${rest#${type}}"
            [[ -n "${type}" ]] || type="str"
        fi
        if [[ "${rest}" == =* ]]; then
            defv="${rest#=}"
            defv="${defv%\"}"; defv="${defv#\"}"
            defv="${defv%\'}"; defv="${defv#\'}"
        fi

        [[ "${type}" == "bool" ]] && continue

        human="$(help_humanize_name "${name}")"

        if [[ "${#name}" -eq 1 ]]; then
            left="-${name}"
        else
            left="--${name//_/-}"
        fi

        desc="${human}"
        [[ "${type}" == "list" ]] && desc+=" (repeatable)"

        desc+=" ( Optional )"

        if [[ -n "${defv}" ]]; then
            desc+=" ( Default: ${defv} )"
        fi

        desc+=" ( Type: '${type}' )"

        (( has_opts )) || { log_info "\n    Options:\n"; has_opts=1; }
        printf '        %-*s %s\n' "${pad_opt}" "${left}" "${desc}"

    done

    for tok in "${tokens[@]}"; do

        [[ -n "${tok}" ]] || continue
        [[ "${tok}" == "--" ]] && continue
        [[ "${tok}" == :* ]] && continue

        raw="${tok}"
        name="${raw%%[:=]*}"
        rest="${raw#${name}}"

        [[ -n "${name}" ]] || continue

        type="str"
        defv=""

        if [[ "${rest}" == :* ]]; then
            rest="${rest#:}"
            type="${rest%%=*}"
            rest="${rest#${type}}"
            [[ -n "${type}" ]] || type="str"
        fi
        if [[ "${rest}" == =* ]]; then
            defv="${rest#=}"
            defv="${defv%\"}"; defv="${defv#\"}"
            defv="${defv%\'}"; defv="${defv#\'}"
        fi

        [[ "${type}" == "bool" ]] || continue

        human="$(help_humanize_name "${name}")"

        if [[ "${#name}" -eq 1 ]]; then
            left="-${name}"
        else
            left="--${name//_/-}"
        fi

        local d="false"
        if [[ -n "${defv}" ]]; then
            case "${defv,,}" in
                1|true|yes|on) d="true" ;;
                0|false|no|off) d="false" ;;
                *) d="${defv}" ;;
            esac
        fi

        desc="${human} ( Optional ) ( Default: ${d} ) ( Type: '${type}' )"

        (( has_flags )) || { log_info "\n    Flags:\n"; has_flags=1; }
        printf '        %-*s %s\n' "${pad_flag}" "${left}" "${desc}"

    done

    printf '\n'

}
cmd_help () {

    source <(parse "$@" -- command)

    local fn="$(help_cmd_to_fn "${command}")"
    declare -F "${fn}" >/dev/null 2>&1 || die "Unknown command: ${command}" 2

    info_ln "Help : ( ${command} )"

    if [[ -z "${command}" ]]; then

        printf '    %s\n' "Usage:"
        printf '    %s\n' "    rustx help <command>"
        printf '    %s\n' ""
        printf '    %s\n' "Example:"
        printf '    %s\n' "    rustx help fmt-check"
        printf '    %s\n' ""

        return 0

    fi

    local schema="$(help_extract_parse_schema "${fn}" 2>/dev/null || true)"
    help_print_generated_help "${command}" "${fn}" "${schema}"

}
cmd_source () {

    source <(parse "$@" -- :command lines:bool=true)

    info_ln "Source :\n"
    log_info "    Code :\n"

    local fn="$(help_cmd_to_fn "${command}")"
    declare -F "${fn}" >/dev/null 2>&1 || { error "Unknown command: ${command}"; return 1; }

    if (( lines )); then declare -f "${fn}" | nl -ba
    else declare -f "${fn}"
    fi

    printf '\n'

}
