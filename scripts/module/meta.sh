#!/usr/bin/env bash

META_TOKEN_OLD=""
META_TOKEN_OLD_SET=0
META_TOKEN_XTRACE=0

meta_token_check () {

    local token="${1-}"

    [[ -n "${token}" ]] || die "Missing registry token. Use --token or set CARGO_REGISTRY_TOKEN." 2

    if [[ "${token}" =~ [[:space:]] ]]; then
        die "Invalid registry token: contains whitespace." 2
    fi

    return 0

}
meta_token_export () {

    local token="${1-}"

    META_TOKEN_OLD=""
    META_TOKEN_OLD_SET=0
    META_TOKEN_XTRACE=0

    if [[ -n "${CARGO_REGISTRY_TOKEN+x}" ]]; then

        META_TOKEN_OLD_SET=1
        META_TOKEN_OLD="${CARGO_REGISTRY_TOKEN}"

    fi

    [[ $- == *x* ]] && { META_TOKEN_XTRACE=1; set +x; }

    export CARGO_REGISTRY_TOKEN="${token}"

}
meta_token_restore () {

    if (( META_TOKEN_OLD_SET )); then
        export CARGO_REGISTRY_TOKEN="${META_TOKEN_OLD}"
    else
        unset CARGO_REGISTRY_TOKEN
    fi

    (( META_TOKEN_XTRACE )) && set -x

    META_TOKEN_OLD=""
    META_TOKEN_OLD_SET=0
    META_TOKEN_XTRACE=0

    return 0

}

cmd_meta_help () {

    info_ln "Meta :\n"

    printf '    %s\n' \
        "version                    * Show root Cargo.toml version" \
        "meta                       * Show workspace metadata (members, names, packages, publishable set)" \
        "conform                    * Verify every crate matches the workspace anatomy" \
        ""\
        "is-publishable             * Check if <crate-name> is publishable or not" \
        "is-published               * Check if <crate-name> is published or not" \
        "can-publish                * Check if workspace or -p/--package available to publish now or not" \
        ""\
        "publish                    * Publish crates in dependency order (workspace publish)" \
        "yank                       * Yank a published version (or undo yank)" \
        ''

}

cmd_version () {

    ensure jq

    local name="${1:-}"
    local meta="" v=""

    meta="$(run_cargo metadata --no-deps --format-version 1)" || die "Error: failed to read cargo metadata." 2

    if [[ -z "${name}" ]]; then

        local ws_root="" root_manifest=""

        ws_root="$(jq -r '.workspace_root' <<<"${meta}")"
        root_manifest="${ws_root}/Cargo.toml"

        v="$(jq -r --arg m "${root_manifest}" '.packages[] | select(.manifest_path == $m) | .version' <<<"${meta}" 2>/dev/null || true)"

        if [[ -z "${v}" || "${v}" == "null" ]]; then

            local id=""
            id="$(jq -r '.workspace_members[0]' <<<"${meta}")"

            v="$(jq -r --arg id "${id}" '.packages[] | select(.id == $id) | .version' <<<"${meta}")"

        fi

        [[ -n "${v}" && "${v}" != "null" ]] || die "Error: workspace version not found." 2

        printf '%s\n' "${v}"
        return 0

    fi

    v="$(jq -r --arg n "${name}" '.packages[] | select(.name == $n) | .version' <<<"${meta}" 2>/dev/null | head -n 1)"

    [[ -n "${v}" && "${v}" != "null" ]] || die "Error: package ${name} not found." 2
    printf '%s\n' "${v}"

}
cmd_meta () {

    ensure jq tee

    local full=0
    local mode="pretty"
    local package=""
    local out=""
    local jq_color=0
    local jq_compact=0
    local only_published=0
    local members_names=0
    local registries=()
    local registries_set=0

    while [[ $# -gt 0 ]]; do
        case "${1}" in
            --full)
                full=1
                shift || true
            ;;
            --no-deps)
                full=0
                shift || true
            ;;
            -p|--package)
                shift || true
                package="${1:-}"
                [[ -n "${package}" ]] || die "Error: -p/--package requires a value" 2
                shift || true
            ;;
            --members)
                mode="members"
                shift || true
            ;;
            --names)
                mode="members"
                members_names=1
                shift || true
            ;;
            --packages)
                mode="packages"
                shift || true
            ;;
            --only-publishable)
                only_published=1
                shift || true
            ;;
            --registries|--registry)
                shift || true
                local raw="${1:-}"
                [[ -n "${raw}" ]] || die "Error: --registries requires a value" 2
                shift || true

                registries_set=1

                local tmp="${raw// /}"
                local parts=()
                local old_ifs="${IFS}"

                IFS=',' read -r -a parts <<< "${tmp}"
                IFS="${old_ifs}"

                local p=""
                for p in "${parts[@]}"; do
                    [[ -n "${p}" ]] || continue
                    registries+=( "${p}" )
                done
            ;;
            --compact|-c)
                jq_compact=1
                shift || true
            ;;
            --color|-C)
                jq_color=1
                shift || true
            ;;
            --out)
                shift || true
                out="${1:-}"
                [[ -n "${out}" ]] || die "Error: --out requires a value" 2
                shift || true
            ;;
            --)
                shift || true
                break
            ;;
            *)
                break
            ;;
        esac
    done

    if [[ -n "${package}" && "${mode}" == "members" ]]; then
        die "Error: -p/--package cannot be used with --members/--names" 2
    fi
    if (( registries_set )); then

        (( ${#registries[@]} )) || die "Error: --registries requires at least one registry name" 2
        only_published=1

    fi
    if (( only_published )) && (( registries_set == 0 )); then
        registries=( "crates-io" )
    fi

    local cargo_args=( --format-version=1 )
    local jq_args=()

    (( full )) || cargo_args+=( --no-deps )
    (( jq_compact )) && jq_args+=( -c )
    (( jq_color )) && jq_args+=( -C )

    local jq_prelude=""
    local publishable_filter=""
    local regs_json="[]"
    local filter="."

    if (( only_published )); then

        regs_json="$(printf '%s\n' "${registries[@]}" | jq -Rn '[inputs]')"
        jq_args+=( --argjson regs "${regs_json}" )

        jq_prelude='
            def publish_allows:
                if .publish == null then
                    true
                elif .publish == false then
                    false
                elif (.publish | type) != "array" then
                    false
                elif (.publish | length) == 0 then
                    false
                elif ($regs | index("*")) != null then
                    true
                else
                    (.publish | any(. as $r | $regs | index($r) != null))
                end;
        '

        publishable_filter='
            | select(publish_allows)
        '

    fi

    if [[ -n "${package}" ]]; then

        jq_args+=( --arg p "${package}" )

        if (( only_published )); then
            filter="${jq_prelude}${WS_MEMBERS_JQ}${publishable_filter} | select(.name == \$p)"
        else
            filter=".packages[] | select(.name == \$p)"
        fi

    else

        local stream=""

        if (( only_published )); then
            stream="${jq_prelude}${WS_MEMBERS_JQ}${publishable_filter}"
        else
            stream=".packages[]"
        fi

        case "${mode}" in

            members)
                jq_args+=( -r )
                if (( members_names )); then
                    filter="${stream} | .name"
                else
                    filter="${stream} | .id"
                fi
            ;;
            packages)
                filter="${stream} | {name, version, publish, manifest_path}"
            ;;
            *)
                filter="${stream}"
            ;;

        esac

    fi

    if [[ -n "${out}" ]]; then
        run_cargo metadata "${cargo_args[@]}" | tee "${out}" | jq "${jq_args[@]}" "${filter}"
        return $?
    fi

    run_cargo metadata "${cargo_args[@]}" | jq "${jq_args[@]}" "${filter}"

}

cmd_is_publishable () {

    ensure grep tr
    source <(parse "$@" -- :name)

    local needle=""
    needle="$(printf '%s' "${name}" | tr '[:upper:]' '[:lower:]')"

    if publishable_pkgs | tr '[:upper:]' '[:lower:]' | grep -Fxq -- "${needle}"; then
        printf '%s\n' "yes"
        return 0
    fi

    printf '%s\n' "no"
    return 1

}
cmd_is_published () {

    ensure grep curl
    source <(parse "$@" -- :name)

    [[ "$(cmd_is_publishable "${name}")" == "yes" ]] || die "Error: package ${name} is not publishable." 2

    local version="" name_lc="${name,,}" path="" tmp="" code=""
    local n="${#name_lc}"

    version="$(cmd_version "${name}")" || die "Error: failed to read version of ${name}." 2

    if (( n == 1 )); then path="1/${name_lc}"
    elif (( n == 2 )); then path="2/${name_lc}"
    elif (( n == 3 )); then path="3/${name_lc:0:1}/${name_lc}"
    else path="${name_lc:0:2}/${name_lc:2:2}/${name_lc}"; fi

    fs_tmp_file tmp "" rustx
    trap 'fs_tmp_release "${tmp}"; trap - RETURN' RETURN

    code="$(curl -sSL --connect-timeout 5 --max-time 20 -o "${tmp}" -w '%{http_code}' "https://index.crates.io/${path}" 2>/dev/null || true)"
    [[ "${code}" =~ ^[0-9]{3}$ ]] || die "Error: crates.io request failed (network?)" 2

    if [[ "${code}" == "404" ]]; then
        echo "no"
        return 0
    fi
    if [[ "${code}" != "200" ]]; then
        die "Error: crates.io index request failed for ${name} (HTTP ${code})." 2
    fi
    if grep -Fq "\"vers\":\"${version}\"" "${tmp}"; then
        echo "yes"
        return 0
    fi

    echo "no"

}
cmd_can_publish () {

    source <(parse "$@" -- name)

    local p="" line="" state=""
    local -a pkgs=()

    if [[ -n "${name}" ]]; then

        state="$(cmd_is_published "${name}")" || die "Error: failed to check the published state of ${name}." 2
        [[ "${state}" == "yes" ]] && { echo "no"; return 0; }

        echo "yes"
        return 0

    fi

    while IFS= read -r line; do pkgs+=( "${line}" ); done < <(publishable_pkgs)
    [[ ${#pkgs[@]} -gt 0 ]] || { echo "no"; return 0; }

    for p in "${pkgs[@]}"; do

        state="$(cmd_is_published "${p}")" || die "Error: failed to check the published state of ${p}." 2
        [[ "${state}" == "yes" ]] && { echo "no"; return 0; }

    done

    echo "yes"

}
cmd_publish () {

    source <(parse "$@" -- token allow_dirty:bool dry_run:bool package:list)

    local p="" state=""
    local -a cargo_args=()

    token="${token:-${CARGO_REGISTRY_TOKEN-}}"

    if (( dry_run )); then
        cargo_args+=( --dry-run --allow-dirty )
    else
        meta_token_check "${token}"
    fi

    if is_ci && ! is_ci_push; then
        die "Refusing publish in CI." 2
    fi
    if (( ! allow_dirty )) && (( ! dry_run )) && has git && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then

        if [[ -n "$(git status --porcelain --untracked-files=normal 2>/dev/null)" ]]; then
            die "Refusing publish with a dirty git working tree. Commit/stash changes, or pass --allow-dirty." 2
        fi

    fi
    if (( ! dry_run )) && ! is_ci; then

        local msg="About to publish "

        if [[ ${#package[@]} -gt 0 ]]; then
            msg+="package(s): ${package[*]}"
        else
            msg+="workspace"
        fi

        confirm "${msg}. Continue?" || die "Aborted." 1

    fi

    meta_token_export "${token}"
    trap 'meta_token_restore; trap - RETURN' RETURN

    if [[ ${#package[@]} -gt 0 ]]; then

        for p in "${package[@]}"; do

            state="$(cmd_can_publish "${p}")" || die "Error: publish gate failed for ${p}." 2
            [[ "${state}" == "yes" ]] || die "Package: ${p} already published" 2

        done

        for p in "${package[@]}"; do run_cargo publish --package "${p}" "${cargo_args[@]}" "${kwargs[@]}"; done

        return 0

    fi

    state="$(cmd_can_publish)" || die "Error: publish gate failed." 2
    [[ "${state}" == "yes" ]] || die "There is some packages already published" 2

    run_cargo publish --workspace "${cargo_args[@]}" "${kwargs[@]}"

}
cmd_yank () {

    source <(parse "$@" -- :package :version token undo:bool)

    version="${version#v}"
    token="${token:-${CARGO_REGISTRY_TOKEN-}}"

    meta_token_check "${token}"

    if is_ci && ! is_ci_push; then
        die "Refusing yank in CI." 2
    fi
    if ! is_ci; then

        local msg="About to yank"
        (( undo )) && msg="About to undo yank"

        confirm "${msg} ${package} v${version}. Continue?" || die "Aborted." 1

    fi

    meta_token_export "${token}"
    trap 'meta_token_restore; trap - RETURN' RETURN

    if (( undo )); then
        run_cargo yank -p "${package}" --version "${version}" --undo "${kwargs[@]}"
        return $?
    fi

    run_cargo yank -p "${package}" --version "${version}" "${kwargs[@]}"

}

conform_fail () {

    local -n __c_r_errors="${1}"
    shift || true

    __c_r_errors+=( "${*}" )

}
conform_crate () {

    local pkg="${1-}" dir="${2-}" facade="${3-}" errors_ref="${4-}"
    local -n __c_r_out="${errors_ref}"
    local want="rustx-${dir##*/}" toml="${dir}/Cargo.toml"

    [[ "${dir##*/}" == "rustx" ]] && want="rustx"

    [[ "${pkg}" == "${want}" ]] || conform_fail __c_r_out "${toml}: package is '${pkg}', the directory requires '${want}'"

    grep -qE '^\[lints\]' -- "${toml}" && grep -qE '^workspace = true' -- "${toml}" \
        || conform_fail __c_r_out "${toml}: missing '[lints] workspace = true'"

    grep -qE '^\[package\.metadata\.docs\.rs\]' -- "${toml}" \
        || conform_fail __c_r_out "${toml}: missing '[package.metadata.docs.rs]'"

    local file=""

    for file in src/lib.rs src/prelude.rs LICENSE-MIT LICENSE-APACHE; do
        [[ -f "${dir}/${file}" ]] || conform_fail __c_r_out "${dir}: missing ${file}"
    done

    grep -qF "\"${dir}\"" -- Cargo.toml || conform_fail __c_r_out "Cargo.toml: '${dir}' is not a workspace member"

    [[ "${pkg}" == "rustx" ]] && return 0

    grep -qE "^${pkg} = \{" -- Cargo.toml || conform_fail __c_r_out "Cargo.toml: '${pkg}' is not in [workspace.dependencies]"

    local alias="${dir##*/}" extern="${pkg//-/_}"

    grep -qF "pub use ${extern} as ${alias};" -- "${facade}/src/lib.rs" \
        || conform_fail __c_r_out "${facade}/src/lib.rs: missing 'pub use ${extern} as ${alias};'"

    grep -qF "pub use ${extern}::prelude::*;" -- "${facade}/src/prelude.rs" \
        || conform_fail __c_r_out "${facade}/src/prelude.rs: missing 'pub use ${extern}::prelude::*;'"

    return 0

}
cmd_conform () {

    local meta="" line="" pkg="" dir="" facade="crates/rustx"
    local -a errors=()

    meta="$(run_cargo metadata --no-deps --format-version 1)" || die "conform: failed to read cargo metadata" 2

    while IFS=$'\t' read -r pkg dir; do

        [[ -n "${pkg}" ]] || continue

        if [[ "${dir}" == crates/* ]]; then
            conform_crate "${pkg}" "${dir}" "${facade}" errors
        else
            grep -qE '^publish = false' -- "${dir}/Cargo.toml" \
                || conform_fail errors "${dir}/Cargo.toml: support crate must set 'publish = false'"

            grep -qE '^\[lints\]' -- "${dir}/Cargo.toml" \
                || conform_fail errors "${dir}/Cargo.toml: missing '[lints] workspace = true'"
        fi

    done < <(jq -r --arg root "${ROOT_DIR%/}/" '
        .packages[] | [ .name, ((.manifest_path | sub("^" + $root; "")) | sub("/Cargo.toml$"; "")) ] | @tsv
    ' <<< "${meta}")

    if (( ${#errors[@]} )); then

        for line in "${errors[@]}"; do error "${line}"; done

        die "conform: ${#errors[@]} anatomy violation(s)" 2

    fi

    success_ln "Conform: every crate matches the workspace anatomy.\n"

}
