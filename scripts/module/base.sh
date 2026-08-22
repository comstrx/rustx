#!/usr/bin/env bash

WS_MEMBERS_JQ='
    . as $m
    | ($m.workspace_members) as $ws
    | $m.packages[]
    | select(.id as $id | $ws | index($id) != null)
    | select(.source == null)
'

active_version () {

    tool_active_version

}
stable_version () {

    tool_stable_version

}
nightly_version () {

    tool_nightly_version

}
msrv_version () {

    tool_msrv_version

}
host_triple () {

    local tc="${1-}" vv="" host=""
    local -a cmd=( rustc )

    [[ -n "${tc}" ]] && cmd+=( "+${tc}" )
    cmd+=( -vV )

    vv="$( "${cmd[@]}" 2>/dev/null )" || die "Failed to read rustc -vV${tc:+ for toolchain ${tc}}." 2
    host="$(awk '{ sub(/\r$/, "") } /^host:/ { print $2; exit }' <<< "${vv}")"

    [[ -n "${host}" ]] || die "Failed to detect the host target triple${tc:+ for toolchain ${tc}}." 2

    printf '%s\n' "${host}"

}
rust_sysroot () {

    local tc="${1-}" sysroot=""
    local -a cmd=( rustc )

    [[ -n "${tc}" ]] && cmd+=( "+${tc}" )
    cmd+=( --print sysroot )

    sysroot="$( "${cmd[@]}" 2>/dev/null )" || return 1
    [[ -n "${sysroot}" ]] || return 1

    if [[ "$(os_name)" == "windows" ]] && has cygpath; then
        sysroot="$(cygpath -u "${sysroot}" 2>/dev/null)" || return 1
    fi

    printf '%s\n' "${sysroot}"

}

ws_query () {

    local prelude="${1-}" filter="${2-}"

    ensure cargo jq

    run cargo metadata --format-version=1 --no-deps | jq -r "${prelude}${WS_MEMBERS_JQ}${filter}" | tool_sort_uniq

}
publishable_pkgs () {

    ws_query '
        def publish_list:
            if .publish == null then ["crates-io"]
            elif .publish == false then []
            elif (.publish | type) == "array" then .publish
            else []
            end;
    ' '
        | select((publish_list | length) > 0)
        | select(publish_list | index("crates-io") != null)
        | .name
    '

}
workspace_pkgs () {

    ws_query '' '
        | .name
    '

}
ensure_workspace_pkg () {

    (( $# > 0 )) || die "ensure_workspace_pkg: missing package name(s)" 2

    local -a ws_pkgs=()
    mapfile -t ws_pkgs < <(workspace_pkgs)

    (( ${#ws_pkgs[@]} > 0 )) || die "ensure_workspace_pkg: no workspace packages found" 2

    local -A ws_set=()
    local -A miss_set=()
    local -a missing=()
    local x="" p=""

    for x in "${ws_pkgs[@]-}"; do
        ws_set["${x}"]=1
    done

    for p in "$@"; do

        [[ -n "${p}" ]] || continue
        [[ -n "${ws_set[${p}]-}" ]] && continue
        [[ -n "${miss_set[${p}]-}" ]] && continue

        miss_set["${p}"]=1
        missing+=( "${p}" )

    done

    (( ${#missing[@]} == 0 )) || die "Unknown workspace package(s): ${missing[*]}" 2

}

resolve_cmd () {

    source <(parse "$@" -- :name:str)

    case "${name}" in
        taplo-cli) name="taplo" ;;
        fd|fd-find) name="fdfind" ;;
        ripgrep) name="rg" ;;
        rust) name="rustc" ;;
        bat) name="batcat" ;;
        ci-cache-clean|semver-checks ) name="cargo-${name}" ;;
    esac

    local n="${name}" n1="${name//_/-}" n2="${name//-/_}"

    command -v -- "${n}"  >/dev/null 2>&1 && { printf '%s\n' "${n}"; return 0; }
    command -v -- "${n1}" >/dev/null 2>&1 && { printf '%s\n' "${n1}"; return 0; }
    command -v -- "${n2}" >/dev/null 2>&1 && { printf '%s\n' "${n2}"; return 0; }

    if [[ "${n}" != cargo-* ]]; then

        [[ "${n}" == "miri" ]] && command -v -- cargo-miri >/dev/null 2>&1 && { printf '%s\n' "cargo +nightly miri"; return 0; }

        command -v -- "cargo-${n}"  >/dev/null 2>&1 && { printf '%s\n' "cargo ${n}";  return 0; }
        command -v -- "cargo-${n1}" >/dev/null 2>&1 && { printf '%s\n' "cargo ${n1}"; return 0; }
        command -v -- "cargo-${n2}" >/dev/null 2>&1 && { printf '%s\n' "cargo ${n2}"; return 0; }

    fi

    return 1

}
stage_file () {

    local __stage_ref="${1-}" out="${2-}" __stage_dir=""

    [[ -n "${__stage_ref}" ]] || die "stage_file: usage: stage_file <out-var> <path>" 2
    [[ -n "${out}" ]] || die "stage_file: missing output path" 2

    [[ "${out}" == */* ]] && ensure_dir "${out%/*}"

    fs_tmp_dir __stage_dir rustx-stage

    local -n __staged="${__stage_ref}"
    __staged="${__stage_dir}/${out##*/}"

    return 0

}
check_max_size () {

    local file="${1-}" max_size="${2-}" bytes="" limit_bytes="" s=""
    [[ -n "${file}" && -n "${max_size}" && -f "${file}" ]] || return 0

    s="${max_size}"
    s="${s#"${s%%[![:space:]]*}"}"
    s="${s%"${s##*[![:space:]]}"}"
    s="${s//[[:space:]]/}"

    [[ "${s}" =~ ^([0-9]+(\.[0-9]+)?)([A-Za-z]*)$ ]] || die "bloat: invalid max_size: ${max_size}" 2

    local val="${BASH_REMATCH[1]}" unit="" mul=1
    unit="${BASH_REMATCH[3],,}"

    case "${unit}" in
        ""|b|bytes) mul=1 ;;
        k|kb) mul=1024 ;;
        m|mb) mul=$(( 1024 * 1024 )) ;;
        g|gb) mul=$(( 1024 * 1024 * 1024 )) ;;
        *) die "bloat: invalid max_size unit '${BASH_REMATCH[3]}': ${max_size}" 2 ;;
    esac

    if [[ "${val}" == *.* ]]; then

        has awk || die "bloat: awk is required for a fractional max_size: ${max_size}" 2

        limit_bytes="$(awk -v v="${val}" -v m="${mul}" 'BEGIN { printf "%.0f", int(v * m + 0.5) }' 2>/dev/null || true)"

    else
        limit_bytes=$(( 10#${val} * mul ))
    fi

    [[ "${limit_bytes}" =~ ^[0-9]+$ ]] || die "bloat: invalid max_size: ${max_size}" 2

    bytes="$(file_size "${file}")"
    [[ "${bytes}" =~ ^[0-9]+$ ]] || die "bloat: failed to read file size: ${file}" 2

    (( bytes > limit_bytes )) || return 0

    error "bloat: max_size exceeded: ${file} (${bytes} bytes > ${max_size})"
    return 1

}

run_cargo () {

    ensure cargo

    local sub="${1:-}" tc="" mode="stable" use_plus=0 need_docflags=0
    local -a pass=()

    [[ -n "${sub}" ]] || die "run_cargo requires a cargo subcommand." 2

    shift || true
    has rustup && use_plus=1

    case "${sub}" in
        add|rm|bench|build|check|test|clean|doc|fetch|fix|generate-lockfile|help|init|install|locate-project|login|logout|metadata|new|info) : ;;
        owner|package|pkgid|publish|remove|report|run|rustc|rustdoc|search|tree|uninstall|update|upgrade|vendor|verify-project|version|yank) : ;;
        clippy|taplo|miri|samply|flamegraph) ensure "${sub}" ;;
        fmt|rustfmt) ensure rustfmt ;;
        *) ensure "cargo-${sub}" ;;
    esac

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -n) mode="nightly"; shift || true ;;
            -m) mode="msrv"; shift || true ;;
            -s) mode="stable"; shift || true ;;
            *) break ;;
        esac
    done

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --nightly) mode="nightly"; shift || true ;;
            --msrv|--min) mode="msrv"; shift || true ;;
            --stable) mode="stable"; shift || true ;;
            --) pass+=( "--" ); shift || true; pass+=( "$@" ); break ;;
            *) pass+=( "$1" ); shift || true ;;
        esac
    done

    if (( use_plus )); then

        if [[ "${mode}" == "nightly" ]]; then tc="$(tool_nightly_version)"
        elif [[ "${mode}" == "msrv" ]]; then tc="$(tool_msrv_version)"
        else tc="$(tool_stable_version)"
        fi

    else

        [[ "${mode}" == "stable" ]] || die "rustup not found: Use --stable or install rustup." 2

    fi

    if [[ "${sub}" == "doc" || "${sub}" == "rustdoc" ]]; then

        need_docflags=1

    elif [[ "${sub}" == "test" ]]; then

        local a=""

        for a in "${pass[@]}"; do
            [[ "${a}" == "--doc" ]] && { need_docflags=1; break; }
        done

    fi

    if (( need_docflags )); then

        local docflags=""
        docflags="$(tool_docflags_deny)"

        if (( use_plus )); then
            RUSTDOCFLAGS="${docflags}" run cargo +"${tc}" "${sub}" "${pass[@]}"
            return $?
        fi

        RUSTDOCFLAGS="${docflags}" run cargo "${sub}" "${pass[@]}"
        return $?

    fi
    if (( use_plus )); then

        run cargo +"${tc}" "${sub}" "${pass[@]}"
        return $?

    fi

    run cargo "${sub}" "${pass[@]}"

}
run_workspace_scope () {

    local scope="${1:-}" command="${2:-}" features=0 targets=0 no_deps=0 all=0 workspace=1 a="" nested=""
    local -a extra=() scoped=()

    [[ -n "${scope}" ]] || die "run_workspace: missing scope" 2
    [[ -n "${command}" ]] || die "run_workspace: missing sub-command" 2

    shift 2 || true

    if [[ "${1-}" == "features-on" || "${1-}" == "features-off" ]]; then
        [[ "${1}" == "features-on" ]] && features=1
        shift || true
    fi
    if [[ "${1-}" == "targets-on" || "${1-}" == "targets-off" ]]; then
        [[ "${1}" == "targets-on" ]] && targets=1
        shift || true
    fi
    if [[ "${1-}" == "deps-on" || "${1-}" == "deps-off" ]]; then
        [[ "${1}" == "deps-off" ]] && no_deps=1
        shift || true
    fi
    if [[ "${1-}" == "all-on" || "${1-}" == "all-off" ]]; then
        [[ "${1}" == "all-on" ]] && all=1
        shift || true
    fi
    if [[ "${command}" == "nextest" || "${command}" == "hack" ]]; then
        if [[ "${1-}" != "" && "${1}" != "--" && "${1}" != -* ]]; then
            nested="${1}"
            shift || true
        fi
    fi

    for a in "$@"; do

        [[ "${a}" == "--" ]] && break

        case "${a}" in
            -p|--package|--package=*|--manifest-path|--manifest-path=*|--workspace|--workspace=*|--all)
                workspace=0
            ;;
        esac
        case "${a}" in
            -F|--features|--features=*|--no-default-features|--all-features)
                features=0
            ;;
        esac
        case "${a}" in
            --lib|--bin|--bin=*|--bins|--example|--example=*|--examples|--test|--test=*|--tests|--bench|--bench=*|--benches|--all-targets)
                targets=0
            ;;
        esac
        case "${a}" in
            --no-deps|--no-deps=*) no_deps=0 ;;
        esac
        case "${a}" in
            --all|--all=*) all=0 ;;
        esac

    done

    (( features )) && extra+=( --all-features )
    (( targets )) && extra+=( --all-targets )
    (( no_deps )) && extra+=( --no-deps )
    (( all )) && extra+=( --all )

    if (( workspace && ! all )); then

        if [[ "${scope}" == "publishable" ]]; then

            local p=""

            while IFS= read -r p; do [[ -n "${p}" ]] && scoped+=( --package "${p}" ); done < <(publishable_pkgs)
            (( ${#scoped[@]} )) || die "No publishable workspace crates found" 2

        else

            scoped=( --workspace )

        fi

    fi

    if [[ -n "${nested}" ]]; then

        run_cargo "${command}" "${nested}" "${scoped[@]}" "${extra[@]}" "$@"
        return $?

    fi

    run_cargo "${command}" "${scoped[@]}" "${extra[@]}" "$@"

}
run_workspace () {

    run_workspace_scope workspace "$@"

}
run_workspace_publishable () {

    run_workspace_scope publishable "$@"

}
