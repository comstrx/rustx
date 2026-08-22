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

    local __p_k="${1-}"

    __p_k="${__p_k#--}"
    __p_k="${__p_k#-}"
    __p_k="${__p_k//-/_}"

    [[ -n "${__p_k}" ]] || die "parse: empty key" 2
    [[ "${__p_k}" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || die "parse: invalid key '${__p_k}'" 2

    printf '%s' "${__p_k}"
    return 0

}
parse_try_norm_key () {

    local __p_k="${1-}"

    __p_k="${__p_k#--}"
    __p_k="${__p_k#-}"
    __p_k="${__p_k//-/_}"

    [[ -n "${__p_k}" ]] || return 1
    [[ "${__p_k}" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || return 1

    printf '%s' "${__p_k}"
    return 0

}
parse_is_schema_token () {

    local __p_s="${1-}"
    [[ "${__p_s}" =~ ^:?(--|-)?[a-zA-Z_][a-zA-Z0-9_-]*(\|(--|-)?[a-zA-Z_][a-zA-Z0-9_-]*)*(:(int|float|str|char|bool|list|any))?([=].*)?$ ]]

}
parse_is_int () {

    [[ "${1-}" =~ ^[+-]?[0-9]+$ ]]

}
parse_is_float () {

    [[ "${1-}" =~ ^[+-]?([0-9]+([.][0-9]+)?|[.][0-9]+)$ ]]

}
parse_is_neg_number_token () {

    local __p_v="${1-}"

    [[ "${__p_v}" =~ ^-[0-9]+$ ]] && return 0
    [[ "${__p_v}" =~ ^-[0-9]+[.][0-9]+$ ]] && return 0
    [[ "${__p_v}" =~ ^-[.][0-9]+$ ]] && return 0

    return 1

}
parse_is_option_like () {

    local __p_v="${1-}"

    [[ "${__p_v}" == "--" ]] && return 1
    [[ "${__p_v}" == --* ]] && return 0
    [[ "${__p_v}" == -* && "${__p_v}" != "-" ]] && return 0

    return 1

}
parse_args__is_known_opt_token () {

    local __p_tok="${1-}"
    local -n __p_r_alias_to="${2}"
    local -n __p_r_stype="${3}"

    local __p_key="" __p_kn="" __p_k=""

    case "${__p_tok}" in
        --no-*|-no-*)
            __p_key="${__p_tok#--no-}"
            __p_key="${__p_key#-no-}"

            __p_kn="$(parse_try_norm_key "${__p_key}" || true)"
            [[ -n "${__p_kn}" ]] || return 1

            __p_k="${__p_r_alias_to[${__p_kn}]-}"
            [[ -n "${__p_k}" ]] || return 1
            [[ "${__p_r_stype[${__p_k}]-}" == "bool" ]] || return 1

            return 0
        ;;
        --*=*|-*=*)
            __p_key="${__p_tok%%=*}"

            __p_key="${__p_key#--}"
            __p_key="${__p_key#-}"

            __p_kn="$(parse_try_norm_key "${__p_key}" || true)"
            [[ -n "${__p_kn}" ]] || return 1

            __p_k="${__p_r_alias_to[${__p_kn}]-}"
            [[ -n "${__p_k}" ]] || return 1

            return 0
        ;;
        --*|-*)
            [[ "${__p_tok}" == "-" || "${__p_tok}" == "--" ]] && return 1

            __p_key="${__p_tok#--}"
            __p_key="${__p_key#-}"

            __p_kn="$(parse_try_norm_key "${__p_key}" || true)"
            [[ -n "${__p_kn}" ]] || return 1

            __p_k="${__p_r_alias_to[${__p_kn}]-}"
            [[ -n "${__p_k}" ]] || return 1

            return 0
        ;;
    esac

    return 1

}
parse_int_norm () {

    local __p_v="${1-}"
    local __p_label="${2-int}"

    [[ -n "${__p_v}" ]] || die "parse: '${__p_label}' must be an integer" 2

    parse_is_int "${__p_v}" && { printf '%s' "${__p_v}"; return 0; }

    if [[ "${__p_v}" =~ ^([+-]?[0-9]+)[.](0+)$ ]]; then
        printf '%s' "${BASH_REMATCH[1]}"
        return 0
    fi

    die "parse: '${__p_label}' must be an integer" 2

}
parse_bool_norm () {

    local __p_v="${1-}"
    local __p_label="${2-bool}"

    [[ -n "${__p_v}" ]] || die "parse: '${__p_label}' must be 'true' or 'false' (or 1/0)" 2

    __p_v="${__p_v,,}"

    case "${__p_v}" in
        1|true|yes|y|on|t)  printf '1' ;;
        0|false|no|n|off|f) printf '0' ;;
        *) die "parse: '${__p_label}' must be 'true' or 'false' (or 1/0)" 2 ;;
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

    local -n __p_r_argv="${1}"
    local -n __p_r_schema="${2}"
    shift 2 || true

    __p_r_argv=()
    __p_r_schema=()

    local -a __p_all=( "$@" )
    local __p_sep=-1
    local __p_i=0

    for (( __p_i=${#__p_all[@]}-1; __p_i>=0; __p_i-- )); do
        if [[ "${__p_all[$__p_i]}" == "--" ]]; then
            __p_sep=$__p_i
            break
        fi
    done

    (( __p_sep >= 0 )) || die "parse: missing '--' separator" 2

    __p_r_argv=( "${__p_all[@]:0:$__p_sep}" )
    __p_r_schema=( "${__p_all[@]:$(( __p_sep + 1 ))}" )

    (( ${#__p_r_schema[@]} )) || die "parse: missing schema" 2

    return 0

}
parse_emit_scalar () {

    local __p_scope="${1-}"
    local __p_name="${2-}"
    local __p_value="${3-}"

    if [[ "${__p_scope}" == "local" ]]; then
        printf 'local %s=%q\n' "${__p_name}" "${__p_value}"
        return 0
    fi

    printf '%s=%q\n' "${__p_name}" "${__p_value}"
    return 0

}
parse_emit_array () {

    local __p_scope="${1-}"
    local __p_name="${2-}"
    shift 2 || true

    if [[ "${__p_scope}" == "local" ]]; then

        if (( $# == 0 )); then
            printf 'local -a %s=()\n' "${__p_name}"
            return 0
        fi

        printf 'local -a %s=(' "${__p_name}"

        local __p_x=""
        for __p_x in "$@"; do
            printf ' %q' "${__p_x}"
        done

        printf ' )\n'
        return 0

    fi

    if (( $# == 0 )); then
        printf '%s=()\n' "${__p_name}"
        return 0
    fi

    printf '%s=(' "${__p_name}"

    local __p_x=""
    for __p_x in "$@"; do
        printf ' %q' "${__p_x}"
    done

    printf ' )\n'
    return 0

}
parse_is_reserved_key () {

    local __p_k="${1-}"

    case "${__p_k}" in
        ""|kwargs|argv|schema|stype|sreq|sdef|sdef_has|set|alias_to|sdisp|order|pos_order|auto_order|auto_has_opt)
            return 0
        ;;
        IFS|OPTIND|OPTARG|REPLY|PWD|OLDPWD|PATH|HOME|SHELL|UID|EUID|PPID|BASHPID)
            return 0
        ;;
        ROOT_DIR|YES_ENV|QUIET_ENV|VERBOSE_ENV|ERR_HANDLER|MODULE_DIR|MODULE_CMD|MODULE_ARGS)
            return 0
        ;;
    esac

    return 1

}
parse_args__schema_build () {

    local -n __p_r_schema="${1}"
    local -n __p_r_stype="${2}"
    local -n __p_r_sreq="${3}"
    local -n __p_r_sdef="${4}"
    local -n __p_r_sdef_has="${5}"
    local -n __p_r_alias_to="${6}"
    local -n __p_r_sdisp="${7}"
    local -n __p_r_order="${8}"
    local -n __p_r_pos_order="${9}"
    local -n __p_r_auto_order="${10}"
    local -n __p_r_auto_has_opt="${11}"
    local -n __p_r_kwargs_req="${12}"
    local -n __p_r_have_kwargs_schema="${13}"

    local __p_spec="" __p_raw="" __p_names="" __p_canon="" __p_nk="" __p_kind="" __p_t=""
    local __p_def_raw="" __p_def_has=0
    local __p_req=0 __p_has_opt=0

    local -a __p_name_list=()
    local __p_nm="" __p_ak=""

    __p_r_kwargs_req=0
    __p_r_have_kwargs_schema=0

    for __p_spec in "${__p_r_schema[@]}"; do

        parse_is_schema_token "${__p_spec}" || die "parse: bad schema token '${__p_spec}'" 2

        __p_raw="${__p_spec}"
        __p_req=0
        __p_def_has=0
        __p_def_raw=""

        if [[ "${__p_raw}" == :* ]]; then
            __p_req=1
            __p_raw="${__p_raw#:}"
        fi
        if [[ "${__p_raw}" == *"="* ]]; then
            __p_def_raw="${__p_raw#*=}"
            __p_raw="${__p_raw%%=*}"
            __p_def_has=1
        fi

        if [[ "${__p_raw}" == *:* ]]; then
            __p_t="${__p_raw##*:}"
            __p_names="${__p_raw%:*}"
        else
            __p_t="__auto__"
            __p_names="${__p_raw}"
        fi

        __p_name_list=()
        IFS='|' read -r -a __p_name_list <<< "${__p_names}"
        (( ${#__p_name_list[@]} )) || die "parse: bad schema '${__p_spec}'" 2

        __p_canon="${__p_name_list[0]}"
        [[ "${__p_canon}" != --no-* && "${__p_canon}" != -no-* ]] || die "parse: schema name '${__p_canon}' is reserved (no- prefix)" 2

        __p_nk="$(parse_norm_key "${__p_canon}")"
        [[ "${__p_nk}" != __* ]] || die "parse: key '${__p_canon}' is reserved (internal prefix)" 2

        if [[ "${__p_nk}" == "kwargs" ]]; then

            [[ "${__p_canon}" != --* && "${__p_canon}" != -* ]] || die "parse: kwargs must be positional (no -/-- prefix)" 2
            (( ${#__p_name_list[@]} == 1 )) || die "parse: kwargs must not have aliases" 2
            (( __p_def_has )) && die "parse: kwargs does not support default value" 2

            __p_r_have_kwargs_schema=1
            __p_r_kwargs_req="${__p_req}"

            continue

        fi

        parse_is_reserved_key "${__p_nk}" && die "parse: key '${__p_canon}' is reserved" 2

        if [[ "${__p_t}" == "__auto__" ]]; then

            __p_has_opt=0
            for __p_nm in "${__p_name_list[@]-}"; do
                if [[ "${__p_nm}" == --* || "${__p_nm}" == -* ]]; then
                    __p_has_opt=1
                    break
                fi
            done

            __p_r_auto_order+=( "${__p_nk}" )
            __p_r_auto_has_opt["${__p_nk}"]="${__p_has_opt}"

        fi

        case "${__p_t}" in
            __auto__|int|float|str|char|bool|list|any) ;;
            *) die "parse: unknown type '${__p_t}' for '${__p_spec}'" 2 ;;
        esac

        [[ -z "${__p_r_stype[${__p_nk}]-}" ]] || die "parse: duplicate name '${__p_nk}'" 2

        __p_r_stype["${__p_nk}"]="${__p_t}"
        __p_r_sreq["${__p_nk}"]="${__p_req}"
        __p_r_sdisp["${__p_nk}"]="${__p_canon}"

        if (( __p_def_has )); then
            __p_r_sdef["${__p_nk}"]="${__p_def_raw}"
            __p_r_sdef_has["${__p_nk}"]=1
        fi

        __p_r_order+=( "${__p_nk}" )

        __p_kind="pos"
        if [[ "${__p_canon}" == --* ]]; then
            __p_kind="long"
        elif [[ "${__p_canon}" == -* ]]; then
            __p_kind="short"
        fi

        [[ "${__p_kind}" == "pos" ]] && __p_r_pos_order+=( "${__p_nk}" )

        for __p_nm in "${__p_name_list[@]-}"; do

            [[ "${__p_nm}" != --no-* && "${__p_nm}" != -no-* ]] || die "parse: schema alias '${__p_nm}' is reserved (no- prefix)" 2

            __p_ak="$(parse_norm_key "${__p_nm}")"
            [[ "${__p_ak}" != __* ]] || die "parse: schema alias '${__p_nm}' is reserved (internal prefix)" 2

            if [[ -n "${__p_r_alias_to[${__p_ak}]-}" ]]; then
                [[ "${__p_r_alias_to[${__p_ak}]}" == "${__p_nk}" ]] || die "parse: duplicate alias '${__p_nm}'" 2
                continue
            fi

            __p_r_alias_to["${__p_ak}"]="${__p_nk}"

        done

    done

    __p_r_stype["kwargs"]="list"
    __p_r_sdisp["kwargs"]="kwargs"
    __p_r_sreq["kwargs"]="${__p_r_kwargs_req}"

    return 0

}
parse_args__infer_auto_types () {

    local -n __p_r_argv="${1}"
    local -n __p_r_auto_order="${2}"
    local -n __p_r_auto_has_opt="${3}"
    local -n __p_r_alias_to="${4}"
    local -n __p_r_stype="${5}"

    (( ${#__p_r_auto_order[@]} )) || return 0

    local -A __p_auto_has_value=()
    local -A __p_auto_no_value=()

    local __p_ai=0
    local __p_arg="" __p_key="" __p_kn="" __p_kk="" __p_nxt="" __p_akey=""

    while (( __p_ai < ${#__p_r_argv[@]} )); do

        __p_arg="${__p_r_argv[$__p_ai]}"
        __p_ai=$(( __p_ai + 1 ))

        [[ "${__p_arg}" == "--" ]] && break

        case "${__p_arg}" in
            --no-*|-no-*)
                __p_key="${__p_arg#--no-}"
                __p_key="${__p_key#-no-}"

                __p_kn="$(parse_try_norm_key "${__p_key}" || true)"
                [[ -n "${__p_kn}" ]] || continue

                __p_kk="${__p_r_alias_to[${__p_kn}]-}"
                [[ -n "${__p_kk}" ]] || continue
                [[ "${__p_r_stype[${__p_kk}]-}" == "__auto__" ]] || continue

                __p_auto_no_value["${__p_kk}"]=1
            ;;
            --*=*|-*=*)
                __p_key="${__p_arg%%=*}"
                if [[ "${__p_key}" == --* ]]; then
                    __p_key="${__p_key#--}"
                else
                    __p_key="${__p_key#-}"
                fi

                __p_kn="$(parse_try_norm_key "${__p_key}" || true)"
                [[ -n "${__p_kn}" ]] || continue

                __p_kk="${__p_r_alias_to[${__p_kn}]-}"
                [[ -n "${__p_kk}" ]] || continue
                [[ "${__p_r_stype[${__p_kk}]-}" == "__auto__" ]] || continue

                __p_auto_has_value["${__p_kk}"]=1
            ;;
            --*|-*)
                __p_key="${__p_arg#--}"
                __p_key="${__p_key#-}"

                __p_kn="$(parse_try_norm_key "${__p_key}" || true)"
                [[ -n "${__p_kn}" ]] || continue

                __p_kk="${__p_r_alias_to[${__p_kn}]-}"
                [[ -n "${__p_kk}" ]] || continue
                [[ "${__p_r_stype[${__p_kk}]-}" == "__auto__" ]] || continue

                if (( __p_ai < ${#__p_r_argv[@]} )); then

                    __p_nxt="${__p_r_argv[$__p_ai]}"

                    if [[ "${__p_nxt}" != "--" ]] && { ! parse_is_option_like "${__p_nxt}" || parse_is_neg_number_token "${__p_nxt}"; }; then
                        __p_auto_has_value["${__p_kk}"]=1
                    else
                        __p_auto_no_value["${__p_kk}"]=1
                    fi

                else

                    __p_auto_no_value["${__p_kk}"]=1

                fi
            ;;
        esac

    done

    for __p_akey in "${__p_r_auto_order[@]-}"; do

        if [[ -n "${__p_auto_has_value[${__p_akey}]-}" && -n "${__p_auto_no_value[${__p_akey}]-}" ]]; then
            __p_r_stype["${__p_akey}"]="any"
            continue
        fi

        if [[ -n "${__p_auto_has_value[${__p_akey}]-}" ]]; then
            __p_r_stype["${__p_akey}"]="str"
            continue
        fi

        if [[ -n "${__p_auto_no_value[${__p_akey}]-}" ]]; then
            __p_r_stype["${__p_akey}"]="bool"
            continue
        fi

        if (( ${__p_r_auto_has_opt[${__p_akey}]-0} )); then
            __p_r_stype["${__p_akey}"]="bool"
        else
            __p_r_stype["${__p_akey}"]="str"
        fi

    done

    return 0

}
parse_args__validate_pos_list_last () {

    local -n __p_r_pos_order="${1}"
    local -n __p_r_stype="${2}"
    local -n __p_r_sdisp="${3}"

    local __p_count=${#__p_r_pos_order[@]}
    (( __p_count > 1 )) || return 0

    local __p_i=0 __p_n=""

    for (( __p_i=0; __p_i < __p_count - 1; __p_i++ )); do

        __p_n="${__p_r_pos_order[$__p_i]}"
        [[ "${__p_r_stype[${__p_n}]-}" == "list" ]] || continue

        warn "parse: positional list '${__p_r_sdisp[${__p_n}]}' is not the last positional;" \
             "'${__p_r_sdisp[${__p_r_pos_order[$(( __p_i + 1 ))]}]}' can only be given as an option"

    done

    return 0

}
parse_args__init_values () {

    local -n __p_r_order="${1}"
    local -n __p_r_stype="${2}"

    local __p_n="" __p_tv=""

    for __p_n in "${__p_r_order[@]}"; do

        __p_tv="${__p_r_stype[${__p_n}]}"

        case "${__p_tv}" in
            int)   parse_set_scalar "${__p_n}" "0" ;;
            float) parse_set_scalar "${__p_n}" "0.0" ;;
            bool)  parse_set_scalar "${__p_n}" "0" ;;
            list)  parse_set_array  "${__p_n}" ;;
            char|str|any) parse_set_scalar "${__p_n}" "" ;;
        esac

    done

    parse_set_array kwargs
    return 0

}
parse_args__parse_argv () {

    local -n __p_r_argv="${1}"
    local -n __p_r_pos_order="${2}"
    local -n __p_r_stype="${3}"
    local -n __p_r_alias_to="${4}"
    local -n __p_r_sdisp="${5}"
    local -n __p_r_set="${6}"

    local __p_raw_mode=0 __p_pos_i=0 __p_pos_list=""
    local __p_i=0 __p_arg="" __p_key="" __p_val="" __p_next="" __p_k="" __p_knorm="" __p_tv=""
    local __p_assigned=0 __p_pos_name="" __p_consumed=0

    while (( __p_i < ${#__p_r_argv[@]} )); do

        __p_arg="${__p_r_argv[$__p_i]}"
        __p_i=$(( __p_i + 1 ))

        if (( __p_raw_mode )); then
            parse_array_append kwargs "${__p_arg}"
            continue
        fi
        if [[ "${__p_arg}" == "--" ]]; then

            parse_array_append kwargs "${__p_arg}"

            while (( __p_i < ${#__p_r_argv[@]} )); do
                parse_array_append kwargs "${__p_r_argv[$__p_i]}"
                __p_i=$(( __p_i + 1 ))
            done

            __p_raw_mode=1
            break

        fi
        if [[ -n "${__p_pos_list}" ]]; then

            if parse_is_neg_number_token "${__p_arg}"; then
                parse_array_append "${__p_pos_list}" "${__p_arg}"
                __p_r_set["${__p_pos_list}"]=1
                continue
            fi

            if parse_is_option_like "${__p_arg}" && parse_args__is_known_opt_token "${__p_arg}" "${!__p_r_alias_to}" "${!__p_r_stype}"; then
                :
            else
                parse_array_append "${__p_pos_list}" "${__p_arg}"
                __p_r_set["${__p_pos_list}"]=1
                continue
            fi

        fi
        if [[ "${__p_arg}" == "-" ]]; then

            parse_array_append kwargs "${__p_arg}"
            continue

        fi
        if [[ "${__p_arg}" =~ ^-[0-9] || "${__p_arg}" =~ ^-\.[0-9] ]]; then

            __p_assigned=0
            while (( __p_pos_i < ${#__p_r_pos_order[@]} )); do

                __p_pos_name="${__p_r_pos_order[$__p_pos_i]}"
                [[ -n "${__p_r_set[${__p_pos_name}]-}" ]] && { __p_pos_i=$(( __p_pos_i + 1 )); continue; }

                __p_tv="${__p_r_stype[${__p_pos_name}]}"
                if [[ "${__p_tv}" == "list" ]]; then
                    __p_pos_list="${__p_pos_name}"
                    parse_array_append "${__p_pos_name}" "${__p_arg}"
                    __p_r_set["${__p_pos_name}"]=1
                    __p_assigned=1
                    break
                fi

                case "${__p_tv}" in
                    int)   __p_arg="$(parse_int_norm "${__p_arg}" "${__p_r_sdisp[${__p_pos_name}]}" )" ;;
                    float) parse_is_float "${__p_arg}" || die "parse: '${__p_r_sdisp[${__p_pos_name}]}' must be a float number" 2 ;;
                    bool)  __p_arg="$(parse_bool_norm "${__p_arg}" "${__p_r_sdisp[${__p_pos_name}]}" )" ;;
                    char)  [[ "${#__p_arg}" -eq 1 ]] || die "parse: '${__p_r_sdisp[${__p_pos_name}]}' must be exactly 1 character" 2 ;;
                esac

                parse_set_scalar "${__p_pos_name}" "${__p_arg}"
                __p_r_set["${__p_pos_name}"]=1
                __p_pos_i=$(( __p_pos_i + 1 ))
                __p_assigned=1
                break

            done

            (( __p_assigned )) || parse_array_append kwargs "${__p_arg}"
            continue

        fi

        case "${__p_arg}" in
            --no-*|-no-*)
                __p_key="${__p_arg#--no-}"
                __p_key="${__p_key#-no-}"

                __p_knorm="$(parse_try_norm_key "${__p_key}" || true)"
                __p_k=""
                [[ -n "${__p_knorm}" ]] && __p_k="${__p_r_alias_to[${__p_knorm}]-}"

                if [[ -n "${__p_k}" && "${__p_r_stype[${__p_k}]}" == "bool" ]]; then
                    parse_set_scalar "${__p_k}" "0"
                    __p_r_set["${__p_k}"]=1
                else
                    parse_array_append kwargs "${__p_arg}"
                fi

                continue
            ;;
            --*=*|-*=*)
                __p_key="${__p_arg%%=*}"
                __p_val="${__p_arg#*=}"

                if [[ "${__p_key}" == --* ]]; then
                    __p_key="${__p_key#--}"
                else
                    __p_key="${__p_key#-}"
                fi

                __p_knorm="$(parse_try_norm_key "${__p_key}" || true)"
                __p_k=""
                [[ -n "${__p_knorm}" ]] && __p_k="${__p_r_alias_to[${__p_knorm}]-}"

                if [[ -z "${__p_k}" ]]; then
                    parse_array_append kwargs "${__p_arg}"
                    continue
                fi

                __p_tv="${__p_r_stype[${__p_k}]}"
                if [[ "${__p_tv}" == "bool" ]]; then
                    __p_val="$(parse_bool_norm "${__p_val}" "${__p_r_sdisp[${__p_k}]}" )"
                    parse_set_scalar "${__p_k}" "${__p_val}"
                elif [[ "${__p_tv}" == "int" ]]; then
                    __p_val="$(parse_int_norm "${__p_val}" "${__p_r_sdisp[${__p_k}]}" )"
                    parse_set_scalar "${__p_k}" "${__p_val}"
                elif [[ "${__p_tv}" == "float" ]]; then
                    parse_is_float "${__p_val}" || die "parse: '${__p_r_sdisp[${__p_k}]}' must be a float number" 2
                    parse_set_scalar "${__p_k}" "${__p_val}"
                elif [[ "${__p_tv}" == "char" ]]; then
                    [[ "${#__p_val}" -eq 1 ]] || die "parse: '${__p_r_sdisp[${__p_k}]}' must be exactly 1 character" 2
                    parse_set_scalar "${__p_k}" "${__p_val}"
                elif [[ "${__p_tv}" == "list" ]]; then
                    parse_array_append "${__p_k}" "${__p_val}"

                    while (( __p_i < ${#__p_r_argv[@]} )); do

                        __p_next="${__p_r_argv[$__p_i]}"

                        [[ "${__p_next}" == "--" ]] && break

                        if parse_is_neg_number_token "${__p_next}"; then
                            parse_array_append "${__p_k}" "${__p_next}"
                            __p_i=$(( __p_i + 1 ))
                            continue
                        fi
                        if parse_is_option_like "${__p_next}" && parse_args__is_known_opt_token "${__p_next}" "${!__p_r_alias_to}" "${!__p_r_stype}"; then
                            break
                        fi

                        parse_array_append "${__p_k}" "${__p_next}"
                        __p_i=$(( __p_i + 1 ))

                    done

                else
                    parse_set_scalar "${__p_k}" "${__p_val}"
                fi

                __p_r_set["${__p_k}"]=1
                continue
            ;;
            --*|-*)
                if [[ "${__p_arg}" == --* ]]; then
                    __p_key="${__p_arg#--}"
                else
                    __p_key="${__p_arg#-}"
                fi

                __p_knorm="$(parse_try_norm_key "${__p_key}" || true)"
                __p_k=""
                [[ -n "${__p_knorm}" ]] && __p_k="${__p_r_alias_to[${__p_knorm}]-}"

                if [[ -z "${__p_k}" ]]; then

                    parse_array_append kwargs "${__p_arg}"

                    if (( __p_i < ${#__p_r_argv[@]} )); then
                        __p_next="${__p_r_argv[$__p_i]}"

                        if [[ "${__p_next}" != "--" ]] && { ! parse_is_option_like "${__p_next}" || parse_is_neg_number_token "${__p_next}"; }; then
                            parse_array_append kwargs "${__p_next}"
                            __p_i=$(( __p_i + 1 ))
                        fi
                    fi

                    continue

                fi

                __p_tv="${__p_r_stype[${__p_k}]}"

                if [[ "${__p_tv}" == "bool" ]]; then

                    if (( __p_i < ${#__p_r_argv[@]} )) && [[ "${__p_r_argv[$__p_i]}" != "--" ]] && { ! parse_is_option_like "${__p_r_argv[$__p_i]}" || parse_is_neg_number_token "${__p_r_argv[$__p_i]}"; }; then
                        __p_val="$(parse_bool_norm "${__p_r_argv[$__p_i]}" "${__p_r_sdisp[${__p_k}]}" )"
                        parse_set_scalar "${__p_k}" "${__p_val}"
                        __p_i=$(( __p_i + 1 ))
                    else
                        parse_set_scalar "${__p_k}" "1"
                    fi

                    __p_r_set["${__p_k}"]=1
                    continue

                fi

                if [[ "${__p_tv}" == "any" ]]; then

                    if (( __p_i < ${#__p_r_argv[@]} )) && [[ "${__p_r_argv[$__p_i]}" != "--" ]] && { ! parse_is_option_like "${__p_r_argv[$__p_i]}" || parse_is_neg_number_token "${__p_r_argv[$__p_i]}"; }; then
                        parse_set_scalar "${__p_k}" "${__p_r_argv[$__p_i]}"
                        __p_i=$(( __p_i + 1 ))
                    else
                        parse_set_scalar "${__p_k}" "1"
                    fi

                    __p_r_set["${__p_k}"]=1
                    continue

                fi

                if [[ "${__p_tv}" == "list" ]]; then

                    __p_consumed=0

                    while (( __p_i < ${#__p_r_argv[@]} )); do

                        __p_next="${__p_r_argv[$__p_i]}"

                        [[ "${__p_next}" == "--" ]] && break

                        if parse_is_neg_number_token "${__p_next}"; then
                            parse_array_append "${__p_k}" "${__p_next}"
                            __p_i=$(( __p_i + 1 ))
                            __p_consumed=1
                            continue
                        fi

                        if parse_is_option_like "${__p_next}" && parse_args__is_known_opt_token "${__p_next}" "${!__p_r_alias_to}" "${!__p_r_stype}"; then
                            break
                        fi

                        parse_array_append "${__p_k}" "${__p_next}"
                        __p_i=$(( __p_i + 1 ))
                        __p_consumed=1

                    done

                    (( __p_consumed )) || die "parse: '${__p_arg}' expects a value" 2

                    __p_r_set["${__p_k}"]=1
                    continue

                fi

                (( __p_i < ${#__p_r_argv[@]} )) || die "parse: '${__p_arg}' expects a value" 2
                __p_next="${__p_r_argv[$__p_i]}"

                if [[ "${__p_next}" == "--" ]]; then
                    die "parse: '${__p_arg}' expects a value" 2
                fi
                if parse_is_option_like "${__p_next}"; then

                    if [[ "${__p_tv}" == "int" || "${__p_tv}" == "float" ]] && parse_is_neg_number_token "${__p_next}"; then
                        :
                    else
                        die "parse: '${__p_arg}' expects a value (use ${__p_arg}=VALUE for values starting with '-')" 2
                    fi

                fi

                __p_i=$(( __p_i + 1 ))

                if [[ "${__p_tv}" == "int" ]]; then
                    __p_next="$(parse_int_norm "${__p_next}" "${__p_r_sdisp[${__p_k}]}" )"
                elif [[ "${__p_tv}" == "float" ]]; then
                    parse_is_float "${__p_next}" || die "parse: '${__p_r_sdisp[${__p_k}]}' must be a float number" 2
                elif [[ "${__p_tv}" == "char" ]]; then
                    [[ "${#__p_next}" -eq 1 ]] || die "parse: '${__p_r_sdisp[${__p_k}]}' must be exactly 1 character" 2
                fi

                parse_set_scalar "${__p_k}" "${__p_next}"

                __p_r_set["${__p_k}"]=1
                continue
            ;;
        esac

        __p_assigned=0
        while (( __p_pos_i < ${#__p_r_pos_order[@]} )); do

            __p_pos_name="${__p_r_pos_order[$__p_pos_i]}"
            [[ -n "${__p_r_set[${__p_pos_name}]-}" ]] && { __p_pos_i=$(( __p_pos_i + 1 )); continue; }

            __p_tv="${__p_r_stype[${__p_pos_name}]}"
            if [[ "${__p_tv}" == "list" ]]; then
                __p_pos_list="${__p_pos_name}"
                parse_array_append "${__p_pos_name}" "${__p_arg}"
                __p_r_set["${__p_pos_name}"]=1
                __p_assigned=1
                break
            fi

            case "${__p_tv}" in
                int)   __p_arg="$(parse_int_norm "${__p_arg}" "${__p_r_sdisp[${__p_pos_name}]}" )" ;;
                float) parse_is_float "${__p_arg}" || die "parse: '${__p_r_sdisp[${__p_pos_name}]}' must be a float number" 2 ;;
                bool)  __p_arg="$(parse_bool_norm "${__p_arg}" "${__p_r_sdisp[${__p_pos_name}]}" )" ;;
                char)  [[ "${#__p_arg}" -eq 1 ]] || die "parse: '${__p_r_sdisp[${__p_pos_name}]}' must be exactly 1 character" 2 ;;
            esac

            parse_set_scalar "${__p_pos_name}" "${__p_arg}"
            __p_r_set["${__p_pos_name}"]=1
            __p_pos_i=$(( __p_pos_i + 1 ))
            __p_assigned=1
            break

        done

        (( __p_assigned )) || parse_array_append kwargs "${__p_arg}"

    done

    return 0

}
parse_args__apply_defaults () {

    local -n __p_r_order="${1}"
    local -n __p_r_stype="${2}"
    local -n __p_r_sdef="${3}"
    local -n __p_r_sdef_has="${4}"
    local -n __p_r_sdisp="${5}"
    local -n __p_r_set="${6}"

    local __p_n="" __p_tv="" __p_def_raw=""
    local -a __p_parts=()

    for __p_n in "${__p_r_order[@]}"; do

        [[ -n "${__p_r_set[${__p_n}]-}" ]] && continue
        [[ -n "${__p_r_sdef_has[${__p_n}]-}" ]] || continue

        __p_tv="${__p_r_stype[${__p_n}]}"
        __p_def_raw="${__p_r_sdef[${__p_n}]-}"

        case "${__p_tv}" in
            int)
                __p_def_raw="$(parse_int_norm "${__p_def_raw}" "${__p_r_sdisp[${__p_n}]}" )"
                parse_set_scalar "${__p_n}" "${__p_def_raw}"
            ;;
            float)
                parse_is_float "${__p_def_raw}" || die "parse: '${__p_r_sdisp[${__p_n}]}' default must be a float number" 2
                parse_set_scalar "${__p_n}" "${__p_def_raw}"
            ;;
            bool)
                __p_def_raw="$(parse_bool_norm "${__p_def_raw}" "${__p_r_sdisp[${__p_n}]}" )"
                parse_set_scalar "${__p_n}" "${__p_def_raw}"
            ;;
            char)
                [[ "${#__p_def_raw}" -eq 1 ]] || die "parse: '${__p_r_sdisp[${__p_n}]}' default must be exactly 1 character" 2
                parse_set_scalar "${__p_n}" "${__p_def_raw}"
            ;;
            list)
                if [[ -z "${__p_def_raw}" ]]; then
                    parse_set_array "${__p_n}"
                else
                    __p_parts=()
                    IFS=',' read -r -a __p_parts <<< "${__p_def_raw}"
                    parse_set_array "${__p_n}" "${__p_parts[@]-}"
                fi
            ;;
            str|any)
                parse_set_scalar "${__p_n}" "${__p_def_raw}"
            ;;
        esac

        __p_r_set["${__p_n}"]=1

    done

    return 0

}
parse_args__validate_and_normalize () {

    local __p_scope="${1-}"
    local -n __p_r_order="${2}"
    local -n __p_r_stype="${3}"
    local -n __p_r_sreq="${4}"
    local -n __p_r_sdisp="${5}"
    local -n __p_r_set="${6}"

    local -n __p_r_kwargs="kwargs"

    if (( __p_r_sreq[kwargs] )); then
        (( ${#__p_r_kwargs[@]} )) || die "parse: missing required 'kwargs'" 2
        __p_r_set["kwargs"]=1
    fi

    local __p_n="" __p_tv="" __p_vv="" __p_emit_scope="local"

    for __p_n in "${__p_r_order[@]}"; do

        __p_tv="${__p_r_stype[${__p_n}]}"

        if (( __p_r_sreq[${__p_n}] )); then
            [[ -n "${__p_r_set[${__p_n}]-}" ]] || die "parse: missing required '${__p_r_sdisp[${__p_n}]}'" 2
        fi

        [[ -n "${__p_r_set[${__p_n}]-}" ]] || continue

        case "${__p_tv}" in
            int)
                parse_set_scalar "${__p_n}" "$(parse_int_norm "${!__p_n-}" "${__p_r_sdisp[${__p_n}]}" )"
            ;;
            float)
                parse_is_float "${!__p_n-}" || die "parse: '${__p_r_sdisp[${__p_n}]}' must be a float number" 2
            ;;
            bool)
                parse_set_scalar "${__p_n}" "$(parse_bool_norm "${!__p_n-}" "${__p_r_sdisp[${__p_n}]}" )"
            ;;
            char)
                __p_vv="${!__p_n-}"
                if (( __p_r_sreq[${__p_n}] )); then
                    [[ "${#__p_vv}" -eq 1 ]] || die "parse: '${__p_r_sdisp[${__p_n}]}' must be exactly 1 character" 2
                else
                    [[ -z "${__p_vv}" || "${#__p_vv}" -eq 1 ]] || die "parse: '${__p_r_sdisp[${__p_n}]}' must be exactly 1 character" 2
                fi
            ;;
            str|any)
                if (( __p_r_sreq[${__p_n}] )); then
                    [[ -n "${!__p_n-}" ]] || die "parse: '${__p_r_sdisp[${__p_n}]}' can't be empty" 2
                fi
            ;;
            list)
                if (( __p_r_sreq[${__p_n}] )); then
                    local -n __p_r_val="${__p_n}"
                    (( ${#__p_r_val[@]} )) || die "parse: missing required '${__p_r_sdisp[${__p_n}]}'" 2
                    unset -n __p_r_val
                fi
            ;;
        esac

    done

    if [[ "${__p_scope}" == "assign" ]]; then
        return 0
    fi

    [[ "${__p_scope}" == "global" ]] && __p_emit_scope="global"

    for __p_n in "${__p_r_order[@]}"; do

        __p_tv="${__p_r_stype[${__p_n}]}"

        if [[ "${__p_tv}" == "list" ]]; then
            local -n __p_r_val="${__p_n}"
            parse_emit_array "${__p_emit_scope}" "${__p_n}" "${__p_r_val[@]}"
            unset -n __p_r_val
        else
            parse_emit_scalar "${__p_emit_scope}" "${__p_n}" "${!__p_n-}"
        fi

    done

    parse_emit_array "${__p_emit_scope}" "kwargs" "${__p_r_kwargs[@]}"

    return 0

}
parse_usage_extract () {

    local -n __p_r_schema="${1}"
    local -n __p_r_usage="${2}"

    __p_r_usage=""

    local -a __p_cleaned=()
    local __p_i=0

    while (( __p_i < ${#__p_r_schema[@]} )); do
        case "${__p_r_schema[$__p_i]}" in
            --usage|--help|-h|--h)
                __p_r_usage="${__p_r_schema[$(( __p_i + 1 ))]-}"
                [[ -n "${__p_r_usage}" ]] || die "parse: help/usage flag requires function name" 2
                __p_i=$(( __p_i + 2 ))
                continue
            ;;
            --usage=*)
                __p_r_usage="${__p_r_schema[$__p_i]#--usage=}"
                [[ -n "${__p_r_usage}" ]] || die "parse: help/usage flag requires function name" 2
                __p_i=$(( __p_i + 1 ))
                continue
            ;;
            --help=*)
                __p_r_usage="${__p_r_schema[$__p_i]#--help=}"
                [[ -n "${__p_r_usage}" ]] || die "parse: help/usage flag requires function name" 2
                __p_i=$(( __p_i + 1 ))
                continue
            ;;
            -h=*)
                __p_r_usage="${__p_r_schema[$__p_i]#-h=}"
                [[ -n "${__p_r_usage}" ]] || die "parse: help/usage flag requires function name" 2
                __p_i=$(( __p_i + 1 ))
                continue
            ;;
            --h=*)
                __p_r_usage="${__p_r_schema[$__p_i]#--h=}"
                [[ -n "${__p_r_usage}" ]] || die "parse: help/usage flag requires function name" 2
                __p_i=$(( __p_i + 1 ))
                continue
            ;;
        esac

        __p_cleaned+=( "${__p_r_schema[$__p_i]}" )
        __p_i=$(( __p_i + 1 ))
    done

    __p_r_schema=( "${__p_cleaned[@]}" )

    if [[ -n "${__p_r_usage}" ]]; then
        [[ "${__p_r_usage}" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] || die "parse: invalid usage fn: ${__p_r_usage}" 2
    fi

}
parse_args () {

    local IFS=$' \n\t'

    local __p_scope="assign"

    if [[ "${1-}" == "--local" ]]; then
        __p_scope="local"
        shift || true
    elif [[ "${1-}" == "--global" ]]; then
        __p_scope="global"
        shift || true
    fi

    parse_require_bash

    local -a __p_argv=()
    local -a __p_schema=()

    parse_args_split __p_argv __p_schema "$@"

    local __p_usage_fn="" __p_a=""
    parse_usage_extract __p_schema __p_usage_fn

    for __p_a in "${__p_argv[@]}"; do
        case "${__p_a}" in
            -h|--help)
                if [[ -n "${__p_usage_fn}" ]]; then
                    printf '%s\n' "if declare -F ${__p_usage_fn} >/dev/null; then"
                    printf '%s\n' "    ${__p_usage_fn}"
                    printf '%s\n' '    if [[ $- == *i* ]]; then return 0 2>/dev/null || true; else exit 0; fi'
                    printf '%s\n' 'fi'
                    printf '%s\n' "printf '%s\n' \"No help available (missing ${__p_usage_fn}()).\" >&2"
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

    local -A __p_stype=()
    local -A __p_sreq=()
    local -A __p_sdef=()
    local -A __p_sdef_has=()
    local -A __p_set=()
    local -A __p_alias_to=()
    local -A __p_sdisp=()

    local -a __p_order=()
    local -a __p_pos_order=()
    local -a __p_auto_order=()
    local -A __p_auto_has_opt=()

    local __p_kwargs_req=0
    local __p_have_kwargs_schema=0

    parse_args__schema_build __p_schema __p_stype __p_sreq __p_sdef __p_sdef_has __p_alias_to __p_sdisp __p_order __p_pos_order __p_auto_order __p_auto_has_opt __p_kwargs_req __p_have_kwargs_schema

    parse_args__infer_auto_types __p_argv __p_auto_order __p_auto_has_opt __p_alias_to __p_stype
    parse_args__validate_pos_list_last __p_pos_order __p_stype __p_sdisp

    parse_args__init_values __p_order __p_stype
    parse_args__parse_argv __p_argv __p_pos_order __p_stype __p_alias_to __p_sdisp __p_set
    parse_args__apply_defaults __p_order __p_stype __p_sdef __p_sdef_has __p_sdisp __p_set

    parse_args__validate_and_normalize "${__p_scope}" __p_order __p_stype __p_sreq __p_sdisp __p_set
    return 0

}
parse () {

    local __p_old_die="" __p_rc=0 __p_out=0
    __p_old_die="$(declare -f die 2>/dev/null || true)"

    # The return-line must reach the stream the caller sources even when die
    # fires inside a nested $(...) — FD 1 there is the capture, not the caller.
    exec {__p_out}>&1

    die () {

        local msg="${1-}"
        local code="${2:-2}"

        printf '❌ %s\n' "${msg}" >&2
        printf 'return %s 2>/dev/null || exit %s\n' "${code}" "${code}" >&"${__p_out}"

        # Non-zero so set -e also aborts parse when die fires inside a nested
        # $(...): the substitution's failure kills the outer parse process too.
        exit "${code}"

    }

    parse_args --local "$@"
    __p_rc=$?

    exec {__p_out}>&-

    if [[ -n "${__p_old_die}" ]]; then eval "${__p_old_die}"
    else unset -f die 2>/dev/null || true
    fi

    return "${__p_rc}"

}
