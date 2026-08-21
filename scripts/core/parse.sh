#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf "%s\n" "parse.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${PARSE_LOADED:-}" ]] && return 0
PARSE_LOADED=1

__dir="${BASH_SOURCE[0]%/*}"
[[ "${__dir}" == "${BASH_SOURCE[0]}" ]] && __dir="."
__core_dir="$(cd -- "${__dir}" && pwd -P)"
source "${__core_dir}/env.sh"

parse_require_bash () {

    [[ -n "${BASH_VERSINFO[0]-}" ]] || die "parse: bash required" 2
    (( ${BASH_VERSINFO[0]:-0} >= 5 )) || die "parse: requires bash >= 5" 2

    return 0

}
parse_norm_key () {

    local k="${1-}"

    k="${k#--}"
    k="${k#-}"
    k="${k//-/_}"

    [[ -n "${k}" ]] || die "parse: empty key" 2
    [[ "${k}" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || die "parse: invalid key '${k}'" 2

    printf '%s' "${k}"
    return 0

}
parse_try_norm_key () {

    local k="${1-}"

    k="${k#--}"
    k="${k#-}"
    k="${k//-/_}"

    [[ -n "${k}" ]] || return 1
    [[ "${k}" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || return 1

    printf '%s' "${k}"
    return 0

}
parse_is_schema_token () {

    local s="${1-}"
    [[ "${s}" =~ ^:?(--|-)?[a-zA-Z_][a-zA-Z0-9_-]*(\|(--|-)?[a-zA-Z_][a-zA-Z0-9_-]*)*(:(int|float|str|char|bool|list|any))?([=].*)?$ ]]

}
parse_is_int () {

    [[ "${1-}" =~ ^[+-]?[0-9]+$ ]]

}
parse_is_float () {

    [[ "${1-}" =~ ^[+-]?([0-9]+([.][0-9]+)?|[.][0-9]+)$ ]]

}
parse_is_neg_number_token () {

    local v="${1-}"

    [[ "${v}" =~ ^-[0-9]+$ ]] && return 0
    [[ "${v}" =~ ^-[0-9]+[.][0-9]+$ ]] && return 0
    [[ "${v}" =~ ^-[.][0-9]+$ ]] && return 0

    return 1

}
parse_is_option_like () {

    local v="${1-}"

    [[ "${v}" == "--" ]] && return 1
    [[ "${v}" == --* ]] && return 0
    [[ "${v}" == -* && "${v}" != "-" ]] && return 0

    return 1

}
parse_args__is_known_opt_token () {

    local tok="${1-}"
    local -n __alias_to="${2}"
    local -n __stype="${3}"

    local key="" kn="" k=""

    case "${tok}" in
        --no-*|-no-*)
            key="${tok#--no-}"
            key="${key#-no-}"

            kn="$(parse_try_norm_key "${key}" || true)"
            [[ -n "${kn}" ]] || return 1

            k="${__alias_to[${kn}]-}"
            [[ -n "${k}" ]] || return 1
            [[ "${__stype[${k}]-}" == "bool" ]] || return 1

            return 0
        ;;
        --*=*|-*=*)
            key="${tok%%=*}"

            key="${key#--}"
            key="${key#-}"

            kn="$(parse_try_norm_key "${key}" || true)"
            [[ -n "${kn}" ]] || return 1

            k="${__alias_to[${kn}]-}"
            [[ -n "${k}" ]] || return 1

            return 0
        ;;
        --*|-*)
            [[ "${tok}" == "-" || "${tok}" == "--" ]] && return 1

            key="${tok#--}"
            key="${key#-}"

            kn="$(parse_try_norm_key "${key}" || true)"
            [[ -n "${kn}" ]] || return 1

            k="${__alias_to[${kn}]-}"
            [[ -n "${k}" ]] || return 1

            return 0
        ;;
    esac

    return 1

}
parse_int_norm () {

    local v="${1-}"
    local label="${2-int}"

    [[ -n "${v}" ]] || die "parse: '${label}' must be an integer" 2

    parse_is_int "${v}" && { printf '%s' "${v}"; return 0; }

    if [[ "${v}" =~ ^([+-]?[0-9]+)[.](0+)$ ]]; then
        printf '%s' "${BASH_REMATCH[1]}"
        return 0
    fi

    die "parse: '${label}' must be an integer" 2

}
parse_bool_norm () {

    local v="${1-}"
    local label="${2-bool}"

    [[ -n "${v}" ]] || die "parse: '${label}' must be 'true' or 'false' (or 1/0)" 2

    v="${v,,}"

    case "${v}" in
        1|true|yes|y|on|t)  printf '1' ;;
        0|false|no|n|off|f) printf '0' ;;
        *) die "parse: '${label}' must be 'true' or 'false' (or 1/0)" 2 ;;
    esac

    return 0

}
parse_set_scalar () {

    local __p_key="${1-}"
    local __p_val="${2-}"

    printf -v "${__p_key}" '%s' "${__p_val}"
    return 0

}
parse_set_array () {

    local __p_key="${1-}"
    shift || true

    local -n __p_ref="${__p_key}"
    __p_ref=()

    (( $# )) && __p_ref+=( "$@" )

    return 0

}
parse_array_append () {

    local __p_key="${1-}"
    local __p_val="${2-}"

    local -n __p_ref="${__p_key}"
    __p_ref+=( "${__p_val}" )

    return 0

}
parse_args_split () {

    local -n out_argv="${1}"
    local -n out_schema="${2}"
    shift 2 || true

    out_argv=()
    out_schema=()

    local -a all=( "$@" )
    local sep=-1
    local i=0

    for (( i=${#all[@]}-1; i>=0; i-- )); do
        if [[ "${all[$i]}" == "--" ]]; then
            sep=$i
            break
        fi
    done

    (( sep >= 0 )) || die "parse: missing '--' separator" 2

    out_argv=( "${all[@]:0:$sep}" )
    out_schema=( "${all[@]:$(( sep + 1 ))}" )

    (( ${#out_schema[@]} )) || die "parse: missing schema" 2

    return 0

}
parse_emit_scalar () {

    local scope="${1-}"
    local name="${2-}"
    local value="${3-}"

    if [[ "${scope}" == "local" ]]; then
        printf 'local %s=%q\n' "${name}" "${value}"
        return 0
    fi

    printf '%s=%q\n' "${name}" "${value}"
    return 0

}
parse_emit_array () {

    local scope="${1-}"
    local name="${2-}"
    shift 2 || true

    if [[ "${scope}" == "local" ]]; then

        if (( $# == 0 )); then
            printf 'local -a %s=()\n' "${name}"
            return 0
        fi

        printf 'local -a %s=(' "${name}"

        local x=""
        for x in "$@"; do
            printf ' %q' "${x}"
        done

        printf ' )\n'
        return 0

    fi

    if (( $# == 0 )); then
        printf '%s=()\n' "${name}"
        return 0
    fi

    printf '%s=(' "${name}"

    local x=""
    for x in "$@"; do
        printf ' %q' "${x}"
    done

    printf ' )\n'
    return 0

}
parse_is_reserved_key () {

    local k="${1-}"

    case "${k}" in
        ""|kwargs|argv|schema|stype|sreq|sdef|sdef_has|set|alias_to|sdisp|order|pos_order|auto_order|auto_has_opt)
            return 0
        ;;
        IFS|OPTIND|OPTARG|REPLY|PWD|OLDPWD|PATH|HOME|SHELL|UID|EUID|PPID|BASHPID)
            return 0
        ;;
        ROOT_DIR|YES_ENV|QUIET_ENV|VERBOSE_ENV|MODULE_DIR|MODULE_CMD|MODULE_ARGS)
            return 0
        ;;
    esac

    return 1

}
parse_args__schema_build () {

    local -n __schema="${1}"
    local -n __stype="${2}"
    local -n __sreq="${3}"
    local -n __sdef="${4}"
    local -n __sdef_has="${5}"
    local -n __alias_to="${6}"
    local -n __sdisp="${7}"
    local -n __order="${8}"
    local -n __pos_order="${9}"
    local -n __auto_order="${10}"
    local -n __auto_has_opt="${11}"
    local -n __kwargs_req="${12}"
    local -n __have_kwargs_schema="${13}"

    local spec="" raw="" names="" canon="" nk="" kind="" t=""
    local def_raw="" def_has=0
    local req=0

    local -a name_list=()
    local nm="" ak=""

    __kwargs_req=0
    __have_kwargs_schema=0

    for spec in "${__schema[@]}"; do

        parse_is_schema_token "${spec}" || die "parse: bad schema token '${spec}'" 2

        raw="${spec}"
        req=0
        def_has=0
        def_raw=""

        if [[ "${raw}" == :* ]]; then
            req=1
            raw="${raw#:}"
        fi
        if [[ "${raw}" == *"="* ]]; then
            def_raw="${raw#*=}"
            raw="${raw%%=*}"
            def_has=1
        fi

        if [[ "${raw}" == *:* ]]; then
            t="${raw##*:}"
            names="${raw%:*}"
        else
            t="__auto__"
            names="${raw}"
        fi

        name_list=()
        IFS='|' read -r -a name_list <<< "${names}"
        (( ${#name_list[@]} )) || die "parse: bad schema '${spec}'" 2

        canon="${name_list[0]}"
        [[ "${canon}" != --no-* && "${canon}" != -no-* ]] || die "parse: schema name '${canon}' is reserved (no- prefix)" 2

        nk="$(parse_norm_key "${canon}")"
        [[ "${nk}" != __* ]] || die "parse: key '${canon}' is reserved (internal prefix)" 2

        if [[ "${nk}" == "kwargs" ]]; then

            [[ "${canon}" != --* && "${canon}" != -* ]] || die "parse: kwargs must be positional (no -/-- prefix)" 2
            (( ${#name_list[@]} == 1 )) || die "parse: kwargs must not have aliases" 2
            (( def_has )) && die "parse: kwargs does not support default value" 2

            __have_kwargs_schema=1
            __kwargs_req="${req}"

            continue

        fi

        parse_is_reserved_key "${nk}" && die "parse: key '${canon}' is reserved" 2

        if [[ "${t}" == "__auto__" ]]; then

            local has_opt=0
            for nm in "${name_list[@]-}"; do
                if [[ "${nm}" == --* || "${nm}" == -* ]]; then
                    has_opt=1
                    break
                fi
            done

            __auto_order+=( "${nk}" )
            __auto_has_opt["${nk}"]="${has_opt}"

        fi

        case "${t}" in
            __auto__|int|float|str|char|bool|list|any) ;;
            *) die "parse: unknown type '${t}' for '${spec}'" 2 ;;
        esac

        [[ -z "${__stype[${nk}]-}" ]] || die "parse: duplicate name '${nk}'" 2

        __stype["${nk}"]="${t}"
        __sreq["${nk}"]="${req}"
        __sdisp["${nk}"]="${canon}"

        if (( def_has )); then
            __sdef["${nk}"]="${def_raw}"
            __sdef_has["${nk}"]=1
        fi

        __order+=( "${nk}" )

        kind="pos"
        if [[ "${canon}" == --* ]]; then
            kind="long"
        elif [[ "${canon}" == -* ]]; then
            kind="short"
        fi

        [[ "${kind}" == "pos" ]] && __pos_order+=( "${nk}" )

        for nm in "${name_list[@]-}"; do

            [[ "${nm}" != --no-* && "${nm}" != -no-* ]] || die "parse: schema alias '${nm}' is reserved (no- prefix)" 2

            ak="$(parse_norm_key "${nm}")"

            if [[ -n "${__alias_to[${ak}]-}" ]]; then
                [[ "${__alias_to[${ak}]}" == "${nk}" ]] || die "parse: duplicate alias '${nm}'" 2
                continue
            fi

            __alias_to["${ak}"]="${nk}"

        done

    done

    __stype["kwargs"]="list"
    __sdisp["kwargs"]="kwargs"
    __sreq["kwargs"]="${__kwargs_req}"

    return 0

}
parse_args__infer_auto_types () {

    local -n __argv="${1}"
    local -n __auto_order="${2}"
    local -n __auto_has_opt="${3}"
    local -n __alias_to="${4}"
    local -n __stype="${5}"

    (( ${#__auto_order[@]} )) || return 0

    local -A auto_has_value=()
    local -A auto_no_value=()

    local ai=0
    local arg2="" key2="" kn2="" kk="" nxt=""

    ai=0
    while (( ai < ${#__argv[@]} )); do

        arg2="${__argv[$ai]}"
        ai=$(( ai + 1 ))

        [[ "${arg2}" == "--" ]] && break

        case "${arg2}" in
            --no-*|-no-*)
                key2="${arg2#--no-}"
                key2="${key2#-no-}"

                kn2="$(parse_try_norm_key "${key2}" || true)"
                [[ -n "${kn2}" ]] || continue

                kk="${__alias_to[${kn2}]-}"
                [[ -n "${kk}" ]] || continue
                [[ "${__stype[${kk}]-}" == "__auto__" ]] || continue

                auto_no_value["${kk}"]=1
            ;;
            --*=*|-*=*)
                key2="${arg2%%=*}"
                if [[ "${key2}" == --* ]]; then
                    key2="${key2#--}"
                else
                    key2="${key2#-}"
                fi

                kn2="$(parse_try_norm_key "${key2}" || true)"
                [[ -n "${kn2}" ]] || continue

                kk="${__alias_to[${kn2}]-}"
                [[ -n "${kk}" ]] || continue
                [[ "${__stype[${kk}]-}" == "__auto__" ]] || continue

                auto_has_value["${kk}"]=1
            ;;
            --*|-*)
                key2="${arg2#--}"
                key2="${key2#-}"

                kn2="$(parse_try_norm_key "${key2}" || true)"
                [[ -n "${kn2}" ]] || continue

                kk="${__alias_to[${kn2}]-}"
                [[ -n "${kk}" ]] || continue
                [[ "${__stype[${kk}]-}" == "__auto__" ]] || continue

                if (( ai < ${#__argv[@]} )); then

                    nxt="${__argv[$ai]}"

                    if [[ "${nxt}" != "--" ]] && { ! parse_is_option_like "${nxt}" || parse_is_neg_number_token "${nxt}"; }; then
                        auto_has_value["${kk}"]=1
                    else
                        auto_no_value["${kk}"]=1
                    fi

                else

                    auto_no_value["${kk}"]=1

                fi
            ;;
        esac

    done

    local akey=""
    for akey in "${__auto_order[@]-}"; do

        if [[ -n "${auto_has_value[${akey}]-}" && -n "${auto_no_value[${akey}]-}" ]]; then
            __stype["${akey}"]="any"
            continue
        fi

        if [[ -n "${auto_has_value[${akey}]-}" ]]; then
            __stype["${akey}"]="str"
            continue
        fi

        if [[ -n "${auto_no_value[${akey}]-}" ]]; then
            __stype["${akey}"]="bool"
            continue
        fi

        if (( ${__auto_has_opt[${akey}]-0} )); then
            __stype["${akey}"]="bool"
        else
            __stype["${akey}"]="str"
        fi

    done

    return 0

}
parse_args__validate_pos_list_last () {

    return 0

}
parse_args__init_values () {

    local -n __order="${1}"
    local -n __stype="${2}"

    local n="" tv=""
    for n in "${__order[@]}"; do

        tv="${__stype[${n}]}"
        case "${tv}" in
            int)   parse_set_scalar "${n}" "0" ;;
            float) parse_set_scalar "${n}" "0.0" ;;
            bool)  parse_set_scalar "${n}" "0" ;;
            list)  parse_set_array  "${n}" ;;
            char|str|any) parse_set_scalar "${n}" "" ;;
        esac

    done

    parse_set_array kwargs
    return 0

}
parse_args__parse_argv () {

    local -n __argv="${1}"
    local -n __pos_order="${2}"
    local -n __stype="${3}"
    local -n __alias_to="${4}"
    local -n __sdisp="${5}"
    local -n __set="${6}"

    local raw_mode=0 pos_i=0 pos_list=""
    local i=0 arg="" key="" val="" next="" k="" knorm="" tv=""

    while (( i < ${#__argv[@]} )); do

        arg="${__argv[$i]}"
        i=$(( i + 1 ))

        if (( raw_mode )); then
            parse_array_append kwargs "${arg}"
            continue
        fi
        if [[ "${arg}" == "--" ]]; then

            parse_array_append kwargs "${arg}"

            while (( i < ${#__argv[@]} )); do
                parse_array_append kwargs "${__argv[$i]}"
                i=$(( i + 1 ))
            done

            raw_mode=1
            break

        fi
        if [[ -n "${pos_list}" ]]; then

            if [[ "${arg}" == "--" ]]; then

                parse_array_append kwargs "${arg}"

                while (( i < ${#__argv[@]} )); do
                    parse_array_append kwargs "${__argv[$i]}"
                    i=$(( i + 1 ))
                done

                raw_mode=1
                break
            fi
            if parse_is_neg_number_token "${arg}"; then
                parse_array_append "${pos_list}" "${arg}"
                __set["${pos_list}"]=1
                continue
            fi

            if parse_is_option_like "${arg}" && parse_args__is_known_opt_token "${arg}" "${!__alias_to}" "${!__stype}"; then
                :
            else
                parse_array_append "${pos_list}" "${arg}"
                __set["${pos_list}"]=1
                continue
            fi

        fi
        if [[ "${arg}" == "-" ]]; then

            parse_array_append kwargs "${arg}"
            continue

        fi
        if [[ "${arg}" =~ ^-[0-9] || "${arg}" =~ ^-\.[0-9] ]]; then

            local assigned=0
            while (( pos_i < ${#__pos_order[@]} )); do

                local pn="${__pos_order[$pos_i]}"
                [[ -n "${__set[${pn}]-}" ]] && { pos_i=$(( pos_i + 1 )); continue; }

                tv="${__stype[${pn}]}"
                if [[ "${tv}" == "list" ]]; then
                    pos_list="${pn}"
                    parse_array_append "${pn}" "${arg}"
                    __set["${pn}"]=1
                    assigned=1
                    break
                fi

                case "${tv}" in
                    int)   arg="$(parse_int_norm "${arg}" "${__sdisp[${pn}]}" )" ;;
                    float) parse_is_float "${arg}" || die "parse: '${__sdisp[${pn}]}' must be a float number" 2 ;;
                    bool)  arg="$(parse_bool_norm "${arg}" "${__sdisp[${pn}]}" )" ;;
                    char)  [[ "${#arg}" -eq 1 ]] || die "parse: '${__sdisp[${pn}]}' must be exactly 1 character" 2 ;;
                esac

                parse_set_scalar "${pn}" "${arg}"
                __set["${pn}"]=1
                pos_i=$(( pos_i + 1 ))
                assigned=1
                break

            done

            (( assigned )) || parse_array_append kwargs "${arg}"
            continue

        fi

        case "${arg}" in
            --no-*|-no-*)
                key="${arg#--no-}"
                key="${key#-no-}"

                knorm="$(parse_try_norm_key "${key}" || true)"
                k=""
                [[ -n "${knorm}" ]] && k="${__alias_to[${knorm}]-}"

                if [[ -n "${k}" && "${__stype[${k}]}" == "bool" ]]; then
                    parse_set_scalar "${k}" "0"
                    __set["${k}"]=1
                else
                    parse_array_append kwargs "${arg}"
                fi

                continue
            ;;
            --*=*|-*=*)
                key="${arg%%=*}"
                val="${arg#*=}"

                if [[ "${key}" == --* ]]; then
                    key="${key#--}"
                else
                    key="${key#-}"
                fi

                knorm="$(parse_try_norm_key "${key}" || true)"
                k=""
                [[ -n "${knorm}" ]] && k="${__alias_to[${knorm}]-}"

                if [[ -z "${k}" ]]; then
                    parse_array_append kwargs "${arg}"
                    continue
                fi

                tv="${__stype[${k}]}"
                if [[ "${tv}" == "bool" ]]; then
                    val="$(parse_bool_norm "${val}" "${__sdisp[${k}]}" )"
                    parse_set_scalar "${k}" "${val}"
                elif [[ "${tv}" == "int" ]]; then
                    val="$(parse_int_norm "${val}" "${__sdisp[${k}]}" )"
                    parse_set_scalar "${k}" "${val}"
                elif [[ "${tv}" == "float" ]]; then
                    parse_is_float "${val}" || die "parse: '${__sdisp[${k}]}' must be a float number" 2
                    parse_set_scalar "${k}" "${val}"
                elif [[ "${tv}" == "char" ]]; then
                    [[ "${#val}" -eq 1 ]] || die "parse: '${__sdisp[${k}]}' must be exactly 1 character" 2
                    parse_set_scalar "${k}" "${val}"
                elif [[ "${tv}" == "list" ]]; then
                    parse_array_append "${k}" "${val}"

                    while (( i < ${#__argv[@]} )); do

                        next="${__argv[$i]}"

                        [[ "${next}" == "--" ]] && break

                        if parse_is_neg_number_token "${next}"; then
                            parse_array_append "${k}" "${next}"
                            i=$(( i + 1 ))
                            continue
                        fi
                        if parse_is_option_like "${next}" && parse_args__is_known_opt_token "${next}" "${!__alias_to}" "${!__stype}"; then
                            break
                        fi

                        parse_array_append "${k}" "${next}"
                        i=$(( i + 1 ))

                    done

                else
                    parse_set_scalar "${k}" "${val}"
                fi

                __set["${k}"]=1
                continue
            ;;
            --*|-*)
                if [[ "${arg}" == --* ]]; then
                    key="${arg#--}"
                else
                    key="${arg#-}"
                fi

                knorm="$(parse_try_norm_key "${key}" || true)"
                k=""
                [[ -n "${knorm}" ]] && k="${__alias_to[${knorm}]-}"

                if [[ -z "${k}" ]]; then

                    parse_array_append kwargs "${arg}"

                    if (( i < ${#__argv[@]} )); then
                        next="${__argv[$i]}"

                        if [[ "${next}" != "--" ]] && { ! parse_is_option_like "${next}" || parse_is_neg_number_token "${next}"; }; then
                            parse_array_append kwargs "${next}"
                            i=$(( i + 1 ))
                        fi
                    fi

                    continue

                fi

                tv="${__stype[${k}]}"

                if [[ "${tv}" == "bool" ]]; then

                    if (( i < ${#__argv[@]} )) && [[ "${__argv[$i]}" != "--" ]] && { ! parse_is_option_like "${__argv[$i]}" || parse_is_neg_number_token "${__argv[$i]}"; }; then
                        val="$(parse_bool_norm "${__argv[$i]}" "${__sdisp[${k}]}" )"
                        parse_set_scalar "${k}" "${val}"
                        i=$(( i + 1 ))
                    else
                        parse_set_scalar "${k}" "1"
                    fi

                    __set["${k}"]=1
                    continue

                fi

                if [[ "${tv}" == "any" ]]; then

                    if (( i < ${#__argv[@]} )) && [[ "${__argv[$i]}" != "--" ]] && { ! parse_is_option_like "${__argv[$i]}" || parse_is_neg_number_token "${__argv[$i]}"; }; then
                        parse_set_scalar "${k}" "${__argv[$i]}"
                        i=$(( i + 1 ))
                    else
                        parse_set_scalar "${k}" "1"
                    fi

                    __set["${k}"]=1
                    continue

                fi

                if [[ "${tv}" == "list" ]]; then

                    local consumed=0

                    while (( i < ${#__argv[@]} )); do

                        next="${__argv[$i]}"

                        [[ "${next}" == "--" ]] && break

                        if parse_is_neg_number_token "${next}"; then
                            parse_array_append "${k}" "${next}"
                            i=$(( i + 1 ))
                            consumed=1
                            continue
                        fi

                        if parse_is_option_like "${next}" && parse_args__is_known_opt_token "${next}" "${!__alias_to}" "${!__stype}"; then
                            break
                        fi

                        parse_array_append "${k}" "${next}"
                        i=$(( i + 1 ))
                        consumed=1

                    done

                    (( consumed )) || die "parse: '${arg}' expects a value" 2

                    __set["${k}"]=1
                    continue

                fi

                (( i < ${#__argv[@]} )) || die "parse: '${arg}' expects a value" 2
                next="${__argv[$i]}"

                if [[ "${next}" == "--" ]]; then
                    die "parse: '${arg}' expects a value" 2
                fi
                if parse_is_option_like "${next}"; then

                    if [[ "${tv}" == "int" || "${tv}" == "float" ]] && parse_is_neg_number_token "${next}"; then
                        :
                    else
                        die "parse: '${arg}' expects a value (use ${arg}=VALUE for values starting with '-')" 2
                    fi

                fi

                i=$(( i + 1 ))

                if [[ "${tv}" == "int" ]]; then
                    next="$(parse_int_norm "${next}" "${__sdisp[${k}]}" )"
                elif [[ "${tv}" == "float" ]]; then
                    parse_is_float "${next}" || die "parse: '${__sdisp[${k}]}' must be a float number" 2
                elif [[ "${tv}" == "char" ]]; then
                    [[ "${#next}" -eq 1 ]] || die "parse: '${__sdisp[${k}]}' must be exactly 1 character" 2
                fi

                if [[ "${tv}" == "list" ]]; then
                    parse_array_append "${k}" "${next}"
                else
                    parse_set_scalar "${k}" "${next}"
                fi

                __set["${k}"]=1
                continue
            ;;
        esac

        local assigned=0
        while (( pos_i < ${#__pos_order[@]} )); do

            local pn="${__pos_order[$pos_i]}"
            [[ -n "${__set[${pn}]-}" ]] && { pos_i=$(( pos_i + 1 )); continue; }

            tv="${__stype[${pn}]}"
            if [[ "${tv}" == "list" ]]; then
                pos_list="${pn}"
                parse_array_append "${pn}" "${arg}"
                __set["${pn}"]=1
                assigned=1
                break
            fi

            case "${tv}" in
                int)   arg="$(parse_int_norm "${arg}" "${__sdisp[${pn}]}" )" ;;
                float) parse_is_float "${arg}" || die "parse: '${__sdisp[${pn}]}' must be a float number" 2 ;;
                bool)  arg="$(parse_bool_norm "${arg}" "${__sdisp[${pn}]}" )" ;;
                char)  [[ "${#arg}" -eq 1 ]] || die "parse: '${__sdisp[${pn}]}' must be exactly 1 character" 2 ;;
            esac

            parse_set_scalar "${pn}" "${arg}"
            __set["${pn}"]=1
            pos_i=$(( pos_i + 1 ))
            assigned=1
            break

        done

        (( assigned )) || parse_array_append kwargs "${arg}"

    done

    return 0

}
parse_args__apply_defaults () {

    local -n __order="${1}"
    local -n __stype="${2}"
    local -n __sdef="${3}"
    local -n __sdef_has="${4}"
    local -n __sdisp="${5}"
    local -n __set="${6}"

    local n="" tv="" def_raw=""
    for n in "${__order[@]}"; do

        [[ -n "${__set[${n}]-}" ]] && continue
        [[ -n "${__sdef_has[${n}]-}" ]] || continue

        tv="${__stype[${n}]}"
        def_raw="${__sdef[${n}]-}"

        case "${tv}" in
            int)
                def_raw="$(parse_int_norm "${def_raw}" "${__sdisp[${n}]}" )"
                parse_set_scalar "${n}" "${def_raw}"
            ;;
            float)
                parse_is_float "${def_raw}" || die "parse: '${__sdisp[${n}]}' default must be a float number" 2
                parse_set_scalar "${n}" "${def_raw}"
            ;;
            bool)
                def_raw="$(parse_bool_norm "${def_raw}" "${__sdisp[${n}]}" )"
                parse_set_scalar "${n}" "${def_raw}"
            ;;
            char)
                [[ "${#def_raw}" -eq 1 ]] || die "parse: '${__sdisp[${n}]}' default must be exactly 1 character" 2
                parse_set_scalar "${n}" "${def_raw}"
            ;;
            list)
                if [[ -z "${def_raw}" ]]; then
                    parse_set_array "${n}"
                else
                    local -a parts=()
                    IFS=',' read -r -a parts <<< "${def_raw}"
                    parse_set_array "${n}" "${parts[@]-}"
                fi
            ;;
            str|any)
                parse_set_scalar "${n}" "${def_raw}"
            ;;
        esac

        __set["${n}"]=1

    done

    return 0

}
parse_args__validate_and_normalize () {

    local scope="${1-}"
    local -n __order="${2}"
    local -n __stype="${3}"
    local -n __sreq="${4}"
    local -n __sdisp="${5}"
    local -n __set="${6}"

    if (( __sreq[kwargs] )); then
        local -n __r_kwargs="kwargs"
        (( ${#__r_kwargs[@]} )) || die "parse: missing required 'kwargs'" 2
        __set["kwargs"]=1
    fi

    local n="" tv="" vv=""
    for n in "${__order[@]}"; do

        tv="${__stype[${n}]}"

        if (( __sreq[${n}] )); then
            [[ -n "${__set[${n}]-}" ]] || die "parse: missing required '${__sdisp[${n}]}'" 2
        fi

        [[ -n "${__set[${n}]-}" ]] || continue

        case "${tv}" in
            int)
                parse_set_scalar "${n}" "$(parse_int_norm "${!n-}" "${__sdisp[${n}]}" )"
            ;;
            float)
                parse_is_float "${!n-}" || die "parse: '${__sdisp[${n}]}' must be a float number" 2
            ;;
            bool)
                parse_set_scalar "${n}" "$(parse_bool_norm "${!n-}" "${__sdisp[${n}]}" )"
            ;;
            char)
                vv="${!n-}"
                if (( __sreq[${n}] )); then
                    [[ "${#vv}" -eq 1 ]] || die "parse: '${__sdisp[${n}]}' must be exactly 1 character" 2
                else
                    [[ -z "${vv}" || "${#vv}" -eq 1 ]] || die "parse: '${__sdisp[${n}]}' must be exactly 1 character" 2
                fi
            ;;
            str|any)
                if (( __sreq[${n}] )); then
                    [[ -n "${!n-}" ]] || die "parse: '${__sdisp[${n}]}' can't be empty" 2
                fi
            ;;
            list)
                if (( __sreq[${n}] )); then
                    local -n r="${n}"
                    (( ${#r[@]} )) || die "parse: missing required '${__sdisp[${n}]}'" 2
                fi
            ;;
        esac

    done

    if [[ "${scope}" == "assign" ]]; then
        return 0
    fi

    local emit_scope="local"
    [[ "${scope}" == "global" ]] && emit_scope="global"

    for n in "${__order[@]}"; do

        tv="${__stype[${n}]}"
        if [[ "${tv}" == "list" ]]; then
            local -n r="${n}"
            parse_emit_array "${emit_scope}" "${n}" "${r[@]}"
        else
            parse_emit_scalar "${emit_scope}" "${n}" "${!n-}"
        fi

    done

    local -n r_kwargs="kwargs"
    parse_emit_array "${emit_scope}" "kwargs" "${r_kwargs[@]}"

    return 0

}
parse_usage_extract () {

    local -n in_schema="${1}"
    local -n out_usage="${2}"

    out_usage=""

    local -a cleaned=()
    local i=0

    while (( i < ${#in_schema[@]} )); do
        case "${in_schema[$i]}" in
            --usage|--help|-h|--h)
                out_usage="${in_schema[$(( i + 1 ))]-}"
                [[ -n "${out_usage}" ]] || die "parse: help/usage flag requires function name" 2
                i=$(( i + 2 ))
                continue
            ;;
            --usage=*)
                out_usage="${in_schema[$i]#--usage=}"
                [[ -n "${out_usage}" ]] || die "parse: help/usage flag requires function name" 2
                i=$(( i + 1 ))
                continue
            ;;
            --help=*)
                out_usage="${in_schema[$i]#--help=}"
                [[ -n "${out_usage}" ]] || die "parse: help/usage flag requires function name" 2
                i=$(( i + 1 ))
                continue
            ;;
            -h=*)
                out_usage="${in_schema[$i]#-h=}"
                [[ -n "${out_usage}" ]] || die "parse: help/usage flag requires function name" 2
                i=$(( i + 1 ))
                continue
            ;;
            --h=*)
                out_usage="${in_schema[$i]#--h=}"
                [[ -n "${out_usage}" ]] || die "parse: help/usage flag requires function name" 2
                i=$(( i + 1 ))
                continue
            ;;
        esac

        cleaned+=( "${in_schema[$i]}" )
        i=$(( i + 1 ))
    done

    in_schema=( "${cleaned[@]}" )

    if [[ -n "${out_usage}" ]]; then
        [[ "${out_usage}" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || die "parse: invalid usage fn: ${out_usage}" 2
    fi

}
parse_args () {

    local IFS=$' \n\t'

    local scope="assign"

    if [[ "${1-}" == "--local" ]]; then
        scope="local"
        shift || true
    elif [[ "${1-}" == "--global" ]]; then
        scope="global"
        shift || true
    fi

    local PARSE_EMIT_MODE=0
    [[ "${scope}" != "assign" ]] && PARSE_EMIT_MODE=1

    parse_require_bash

    local -a argv=()
    local -a schema=()

    parse_args_split argv schema "$@"

    local usage_fn="" a=""
    parse_usage_extract schema usage_fn

    for a in "${argv[@]}"; do
        case "${a}" in
            -h|--help)
                if [[ -n "${usage_fn}" ]]; then
                    printf '%s\n' "if declare -F ${usage_fn} >/dev/null; then"
                    printf '%s\n' "    ${usage_fn}"
                    printf '%s\n' '    if [[ $- == *i* ]]; then return 0 2>/dev/null || true; else exit 0; fi'
                    printf '%s\n' 'fi'
                    printf '%s\n' "printf '%s\n' \"No help available (missing ${usage_fn}()).\" >&2"
                    printf '%s\n' 'if [[ $- == *i* ]]; then return 2 2>/dev/null || true; else exit 2; fi'
                    return 0
                fi

                printf '%s\n' 'if declare -F usage >/dev/null; then'
                printf '%s\n' '    usage'
                printf '%s\n' '    if [[ $- == *i* ]]; then return 0 2>/dev/null || true; else exit 0; fi'
                printf '%s\n' 'elif declare -F help >/dev/null; then'
                printf '%s\n' '    help'
                printf '%s\n' '    if [[ $- == *i* ]]; then return 0 2>/dev/null || true; else exit 0; fi'
                printf '%s\n' 'fi'
                printf '%s\n' 'printf "%s\n" "No help available (define usage() or help())." >&2'
                printf '%s\n' 'if [[ $- == *i* ]]; then return 2 2>/dev/null || true; else exit 2; fi'
                return 0
            ;;
        esac
    done

    local -A stype=()
    local -A sreq=()
    local -A sdef=()
    local -A sdef_has=()
    local -A set=()
    local -A alias_to=()
    local -A sdisp=()

    local -a order=()
    local -a pos_order=()
    local -a auto_order=()
    local -A auto_has_opt=()

    local kwargs_req=0
    local have_kwargs_schema=0

    parse_args__schema_build schema stype sreq sdef sdef_has alias_to sdisp order pos_order auto_order auto_has_opt kwargs_req have_kwargs_schema

    parse_args__infer_auto_types argv auto_order auto_has_opt alias_to stype
    parse_args__validate_pos_list_last pos_order stype sdisp

    parse_args__init_values order stype
    parse_args__parse_argv argv pos_order stype alias_to sdisp set
    parse_args__apply_defaults order stype sdef sdef_has sdisp set

    parse_args__validate_and_normalize "${scope}" order stype sreq sdisp set
    return 0

}
parse () {

    local parse_old_die="$(declare -f die 2>/dev/null || true)"

    die () {

        local msg="${1-}"
        local code="${2:-2}"

        printf '❌ %s\n' "${msg}" >&2
        printf 'return %s 2>/dev/null || exit %s\n' "${code}" "${code}"

        exit 0

    }

    parse_args --local "$@"
    local rc=$?

    if [[ -n "${parse_old_die}" ]]; then eval "${parse_old_die}"
    else unset -f die 2>/dev/null || true
    fi

    return "${rc}"

}
