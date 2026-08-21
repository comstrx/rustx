#!/usr/bin/env bash

cmd_meta_help () {

    info_ln "Meta :\n"

    printf '    %s\n' \
        "version                    * Show root Cargo.toml version" \
        "meta                       * Show workspace metadata (members, names, packages, publishable set)" \
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
    local meta="$(run_cargo metadata --no-deps --format-version 1)" || die "Error: failed to read cargo metadata." 2

    if [[ -z "${name}" ]]; then

        local ws_root="$(jq -r '.workspace_root' <<<"${meta}")"
        local root_manifest="${ws_root}/Cargo.toml"

        local v="$(jq -r --arg m "${root_manifest}" '.packages[] | select(.manifest_path == $m) | .version' <<<"${meta}" 2>/dev/null || true)"

        if [[ -z "${v}" || "${v}" == "null" ]]; then

            local id="$(jq -r '.workspace_members[0]' <<<"${meta}")"
            v="$(jq -r --arg id "${id}" '.packages[] | select(.id == $id) | .version' <<<"${meta}")"

        fi

        [[ -n "${v}" && "${v}" != "null" ]] || die "Error: workspace version not found." 2

        printf '%s\n' "${v}"
        return 0

    fi

    local v="$(jq -r --arg n "${name}" '.packages[] | select(.name == $n) | .version' <<<"${meta}" 2>/dev/null | head -n 1)"

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
    local base_ws_local='
        . as $m
        | ($m.workspace_members) as $ws
        | $m.packages[]
        | select(.id as $id | $ws | index($id) != null)
        | select(.source == null)
    '

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
            filter="${jq_prelude}${base_ws_local}${publishable_filter} | select(.name == \$p)"
        else
            filter=".packages[] | select(.name == \$p)"
        fi

    else

        local stream=""

        if (( only_published )); then
            stream="${jq_prelude}${base_ws_local}${publishable_filter}"
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
        return 0
    fi

    run_cargo metadata "${cargo_args[@]}" | jq "${jq_args[@]}" "${filter}"

}

cmd_is_publishable () {

    ensure grep tr
    source <(parse "$@" -- :name)

    local needle="$(printf '%s' "${name}" | tr '[:upper:]' '[:lower:]')"

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

    local version="$(cmd_version "${name}")"
    local name_lc="${name,,}"
    local n="${#name_lc}"
    local path=""

    if (( n == 1 )); then path="1/${name_lc}"
    elif (( n == 2 )); then path="2/${name_lc}"
    elif (( n == 3 )); then path="3/${name_lc:0:1}/${name_lc}"
    else path="${name_lc:0:2}/${name_lc:2:2}/${name_lc}"; fi

    local tmp="$(mktemp "${TMPDIR:-/tmp}/rust.XXXXXX" 2>/dev/null || printf '%s' "${TMPDIR:-/tmp}/rust.$$")"
    trap 'rm -f -- "${tmp}" 2>/dev/null || true; trap - RETURN' RETURN

    local code="$(curl -sSL --connect-timeout 5 --max-time 20 -o "${tmp}" -w '%{http_code}' "https://index.crates.io/${path}" 2>/dev/null || true)"
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

    if [[ -n "${name}" ]]; then

        [[ "$(cmd_is_published "${name}")" == "yes" ]] && { echo "no"; return 0; }

        echo "yes"
        return 0

    fi

    local p=""
    local -a pkgs=()

    while IFS= read -r line; do pkgs+=( "${line}" ); done < <(publishable_pkgs)
    [[ ${#pkgs[@]} -gt 0 ]] || { echo "no"; return 0; }

    for p in "${pkgs[@]}"; do
        [[ "$(cmd_is_published "${p}")" == "yes" ]] && { echo "no"; return 0; }
    done

    echo "yes"

}
cmd_publish () {

    source <(parse "$@" -- token allow_dirty:bool dry_run:bool package:list)

    local old_token="" old_token_set=0 xtrace=0 i=0 p=""
    local -a cargo_args=()

    token="${token:-${CARGO_REGISTRY_TOKEN-}}"

    [[ -n "${token}" ]] || die "Missing registry token. Use --token or set CARGO_REGISTRY_TOKEN." 2
    [[ "${token}" =~ [[:space:]] ]] && die "Invalid token: ${token}." 2

    (( dry_run )) && cargo_args+=( --dry-run )

    if is_ci && ! is_ci_push; then
        die "Refusing publish in CI." 2
    fi
    if (( ! allow_dirty )) && has git && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then

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
    if [[ -n "${CARGO_REGISTRY_TOKEN+x}" ]]; then

        old_token_set=1
        old_token="${CARGO_REGISTRY_TOKEN}"

    fi
    if [[ -n "${token}" ]]; then

        [[ $- == *x* ]] && { xtrace=1; set +x; }
        export CARGO_REGISTRY_TOKEN="${token}"

        trap '
            if (( old_token_set )); then
                export CARGO_REGISTRY_TOKEN="${old_token}"
            else
                unset CARGO_REGISTRY_TOKEN
            fi

            (( xtrace )) && set -x

            trap - RETURN
        ' RETURN

    fi
    if [[ ${#package[@]} -gt 0 ]]; then

        for p in "${package[@]}"; do [[ "$(cmd_can_publish "${p}")" == "yes" ]] || die "Package: ${p} already published" 2; done
        for p in "${package[@]}"; do run_cargo publish --package "${p}" "${cargo_args[@]}" "${kwargs[@]}"; done

        return 0

    fi

    [[ "$(cmd_can_publish)" == "yes" ]] || die "There is some packages already published" 2
    run_cargo publish --workspace "${cargo_args[@]}" "${kwargs[@]}"

}
cmd_yank () {

    source <(parse "$@" -- :package :version token undo:bool)

    local old_token="" old_token_set=0 xtrace=0

    version="${version#v}"
    token="${token:-${CARGO_REGISTRY_TOKEN-}}"

    [[ -n "${token}" ]] || die "Missing registry token. Use --token or set CARGO_REGISTRY_TOKEN." 2
    [[ "${token}" =~ [[:space:]] ]] && die "Invalid token: ${token}." 2

    if is_ci && ! is_ci_push; then
        die "Refusing yank in CI." 2
    fi
    if ! is_ci; then

        (( undo )) || confirm "About to yank ${package} v${version}. Continue?" || die "Aborted." 1
        (( undo )) && confirm "About to undo yank ${package} v${version}. Continue?" || die "Aborted." 1

    fi
    if [[ -n "${CARGO_REGISTRY_TOKEN+x}" ]]; then
        old_token_set=1
        old_token="${CARGO_REGISTRY_TOKEN}"
    fi
    if [[ -n "${token}" ]]; then

        [[ $- == *x* ]] && { xtrace=1; set +x; }
        export CARGO_REGISTRY_TOKEN="${token}"

        trap '
            if (( old_token_set )); then
                export CARGO_REGISTRY_TOKEN="${old_token}"
            else
                unset CARGO_REGISTRY_TOKEN
            fi

            (( xtrace )) && set -x

            trap - RETURN
        ' RETURN

    fi
    if (( undo )); then
        run_cargo yank -p "${package}" --version "${version}" --undo "${kwargs[@]}"
        return 0
    fi

    run_cargo yank -p "${package}" --version "${version}" "${kwargs[@]}"

}
