#!/usr/bin/env bash

perf_help () {

    info_ln "Perf :\n"

    printf '    %s\n' \
        "bloat                      * Check bloat for (binary size)" \
        "coverage                   * Coverage via cargo llvm-cov (lcov/codecov)" \
        "" \
        "samply                     * CPU profiling via samply (Firefox Profiler UI) for one target" \
        "samply-load                * Load saved samply profile (default: out/samply.json)" \
        "" \
        "flame                      * CPU flamegraph via cargo flamegraph (output: SVG)" \
        "flame-open                 * Open saved flamegraph SVG (default: out/flamegraph.svg)" \
        ''

}
cmd_perf_help () {

    info_ln "Perf :\n"

    printf '    %s\n' \
        "bloat                      * Check bloat for (binary size)" \
        "    --package                Package name/s (repeatable)" \
        "    --out                    Output file, Default: out/bloat.info" \
        "    --max-size               Max size limit, Default: 10MB" \
        "    --all                    Select all workspace packages" \
        "    --release                Build mode, Default: true" \
        "" \
        "coverage                   * Coverage via cargo llvm-cov (lcov/codecov)" \
        "    --mode                   Mode <lcov|codecov|json>, Default: lcov" \
        "    --package                Package name/s (repeatable)" \
        "    --out                    Output file (default depends on mode)" \
        "    --upload                 Upload report (codecov mode)" \
        "    --token                  Token value (default: CODECOV_TOKEN)" \
        "    --name                   Upload name" \
        "    --flags                  Upload flags (default: crates)" \
        "    --version                Codecov cli version (default: latest)" \
        "" \
        "samply                     * CPU profiling via samply (Firefox Profiler UI) for one target" \
        "    --package                Package name (supports at most one package)" \
        "    --bin                    Run a bin target" \
        "    --test                   Run a test target" \
        "    --bench                  Run a bench target" \
        "    --example                Run an example target" \
        "    --toolchain              Toolchain override (stable/nightly/msrv/<custom>)" \
        "    --stable                 Use stable toolchain" \
        "    --nightly                Use nightly toolchain" \
        "    --msrv                   Use msrv toolchain" \
        "    --out                    Output file, Default: out/samply.json" \
        "    --save-only              Save profile only (no UI)" \
        "    --rate                   Sampling rate" \
        "    --address                Server address" \
        "    --duration               Profiling duration" \
        "" \
        "samply-load                * Load saved samply profile (default: out/samply.json)" \
        "    --file                   Profile file, Default: out/samply.json" \
        "" \
        "flame                      * CPU flamegraph via cargo flamegraph (output: SVG)" \
        "    --package                Package name (supports at most one package)" \
        "    --bin                    Run a bin target" \
        "    --test                   Run a test target" \
        "    --bench                  Run a bench target" \
        "    --example                Run an example target" \
        "    --toolchain              Toolchain override (stable/nightly/msrv/<custom>)" \
        "    --stable                 Use stable toolchain" \
        "    --nightly                Use nightly toolchain" \
        "    --msrv                   Use msrv toolchain" \
        "    --out                    Output file, Default: out/flamegraph.svg" \
        "" \
        "flame-open                 * Open saved flamegraph SVG (default: out/flamegraph.svg)" \
        "    --file                   SVG file, Default: out/flamegraph.svg" \
        ''

}

cmd_bloat () {

    source <(parse "$@" -- package:list out="out/bloat.info" max_size=10MB all:bool release:bool=true)

    local -a pkgs=()

    if [[ ${#package[@]} -gt 0 ]]; then pkgs=( "${package[@]}" ); ensure_workspace_pkg "${pkgs[@]}"
    elif (( all )); then mapfile -t pkgs < <(workspace_pkgs)
    else mapfile -t pkgs < <(publishable_pkgs)
    fi

    [[ ${#pkgs[@]} -gt 0 ]] || die "bloat: no packages selected" 2
    [[ "${out}" == */* ]] && run mkdir -p -- "${out%/*}"
    : > "${out}"

    printf '\n%s\n' "---------------------------------------" >> "${out}"
    printf '%s' "- Bloats Report: " >> "${out}"

    [[ -n "${max_size}" ]] && printf '%s' " Max-Size ( ${max_size} )" >> "${out}"

    printf '\n%s\n' "- Version: $(cmd_version)" >> "${out}"
    printf '%s\n\n' "---------------------------------------" >> "${out}"

    local meta="$(run_cargo metadata --no-deps --format-version 1 2>/dev/null)" || die "bloat: failed to get metadata" 2
    local target_dir="$(jq -r '.target_directory' <<<"${meta}" 2>/dev/null || true)"
    [[ -n "${target_dir}" ]] || die "bloat: failed to read target_directory" 2

    local mod="debug" flag="--dev"
    (( release )) && { mod="release"; flag="--release"; }

    local exe="" pkg="" out_text="" i=1 had_error=0
    [[ "$(os_name)" == "windows" ]] && exe=".exe"

    for pkg in "${pkgs[@]}"; do

        printf '%s\n' "Analyzing : ${pkg} ..."

        printf '%d) %s:\n\n' "${i}" "${pkg}" >> "${out}"
        (( i++ ))

        local -a bins=()
        mapfile -t bins < <(jq -r --arg n "${pkg}" '.packages[] | select(.name == $n) | .targets[] | select(.kind | index("bin")) | .name' <<<"${meta}")

        if (( ${#bins[@]} > 0 )); then

            local x="" bin_name="${bins[0]}"
            for x in "${bins[@]}"; do [[ "${x}" == "${pkg}" ]] && { bin_name="${x}"; break; }; done

            local bin_path="${target_dir}/${mod}/${bin_name}${exe}"

            if out_text="$(NO_COLOR=1 CARGO_TERM_COLOR=never run_cargo bloat -p "${pkg}" --bin "${bin_name}" "${flag}" "${kwargs[@]}" 2>&1)"; then

                awk '{ sub(/\r$/, "") } !on && /^[[:space:]]*File[[:space:]]/ { on=1 } on { print }' <<<"${out_text}" >> "${out}"
                printf '\n' >> "${out}"

                [[ -n "${max_size}" ]] && check_max_size "${bin_path}" "${max_size}" || true

            else

                printf 'ERROR: %s\n\n' "can't resolve ${bin_path}" >> "${out}"

            fi

        else

            local manifest_path="" adapter="${pkg}" candidate="" bin_name="" feature="" bin_path="" succeeded=0
            local -a adapters=()

            manifest_path="$(jq -r --arg n "${pkg}" '.packages[] | select(.name == $n) | .manifest_path' <<<"${meta}")"
            if [[ -n "${manifest_path}" && "${manifest_path}" != "null" ]]; then
                adapter="$(basename -- "$(dirname -- "${manifest_path}")")"
            fi

            adapters=( "${adapter}" )
            [[ "${adapter}" == "${pkg}" ]] || adapters+=( "${pkg}" )

            for candidate in "${adapters[@]}"; do
                feature="bloat-${candidate}"

                for bin_name in "bloat-${candidate}" "${candidate}"; do
                    if out_text="$(NO_COLOR=1 CARGO_TERM_COLOR=never run_cargo bloat -p bloats --bin "${bin_name}" --features "${feature}" "${flag}" "${kwargs[@]}" 2>&1)"; then
                        bin_path="${target_dir}/${mod}/${bin_name}${exe}"

                        awk '{ sub(/\r$/, "") } !on && /^[[:space:]]*File[[:space:]]/ { on=1 } on { print }' <<<"${out_text}" >> "${out}"
                        printf '\n' >> "${out}"

                        [[ -n "${max_size}" ]] && check_max_size "${bin_path}" "${max_size}" || true
                        succeeded=1
                        break 2
                    fi
                done
            done

            if (( ! succeeded )); then
                printf 'ERROR: cargo bloat failed for %s (via bloats)\n%s\n\n' "${pkg}" "${out_text}" >> "${out}"
                had_error=1
            fi


        fi

    done

    (( had_error == 0 )) || die "bloat: one or more packages failed; see ${out}" 2
    success "Analyzed: out file -> ${out}"

}
cmd_coverage () {

    ensure llvm-tools-preview jq
    source <(parse "$@" -- mode=lcov name flags version token out upload:bool package:list)

    local -a args=()
    local workspace_pkg=""

    while IFS= read -r workspace_pkg; do
        case "${workspace_pkg}" in
            bloats|fuzz) args+=( --exclude "${workspace_pkg}" ) ;;
        esac
    done < <(workspace_pkgs)

    if [[ ${#package[@]} -gt 0 ]]; then

        local -a all_pkgs=()
        mapfile -t all_pkgs < <(publishable_pkgs)

        local -A ws_set=()
        local -A pick_set=()
        local -A seen=()
        local x="" p=""

        for x in "${all_pkgs[@]}"; do
            ws_set["${x}"]=1
        done

        for p in "${package[@]-}"; do

            [[ -n "${ws_set[${p}]-}" ]] || die "Unknown package: ${p}" 2
            [[ -n "${seen[${p}]-}" ]] && continue

            seen["${p}"]=1
            pick_set["${p}"]=1

        done

        [[ ${#pick_set[@]} -gt 0 ]] || die "No packages selected" 2

        for x in "${all_pkgs[@]}"; do
            [[ -n "${pick_set[${x}]-}" ]] && continue
            args+=( --exclude "${x}" )
        done

    fi

    if [[ "${mode}" == "codecov" || "${mode}" == "json" ]]; then

        args+=( --codecov )

        out="${out:-out/codecov.json}"
        [[ "${out}" == */* ]] && run mkdir -p -- "${out%/*}"
        : > "${out}"

    else

        args+=( --lcov )

        out="${out:-out/lcov.info}"
        [[ "${out}" == */* ]] && run mkdir -p -- "${out%/*}"
        : > "${out}"

    fi

    run_cargo llvm-cov clean --workspace
    run_cargo llvm-cov --workspace --all-targets --all-features "${args[@]}" --output-path "${out}" --remap-path-prefix "${kwargs[@]}"

    if (( upload )); then

        ensure curl chmod mv mkdir

        [[ -n "${flags}" ]] || flags="crates"
        [[ -n "${name}"  ]] || name="coverage-${GITHUB_RUN_ID:-local}"

        [[ -n "${version}" ]] || version="latest"
        [[ -n "${version}" && "${version}" != "latest" && "${version}" != v* ]] && version="v${version}"

        [[ -n "${token}" ]] || token="${CODECOV_TOKEN:-}"
        [[ -n "${token}" ]] || { warn "codecov: CODECOV_TOKEN is missing."; return 0; }
        [[ -f "${out}" ]] || { warn "codecov: file not found: ${out}"; return 0; }

        local os="$(uname -s | tr '[:upper:]' '[:lower:]')"
        local arch="$(uname -m)"
        local dist="linux"

        if [[ "${os}" == "darwin" ]]; then dist="macos"; fi
        if [[ "${dist}" == "linux" && ( "${arch}" == "aarch64" || "${arch}" == "arm64" ) ]]; then dist="linux-arm64"; fi

        local cache_dir="${ROOT_DIR}/.codecov/cache"
        mkdir -p -- "${cache_dir}"

        local resolved="${version}"
        local bin="${cache_dir}/codecov-${dist}-${resolved}"

        if [[ "${version}" == "latest" ]]; then

            local latest_page="$(curl -fsSL "https://cli.codecov.io/${dist}/latest" 2>/dev/null || true)"
            local v="$(printf '%s\n' "${latest_page}" | grep -Eo 'v[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true)"

            [[ -n "${v}" ]] && resolved="${v}"
            bin="${cache_dir}/codecov-${dist}-${resolved}"

        fi
        if [[ ! -x "${bin}" ]]; then

            local base_url="https://cli.codecov.io/${resolved}/${dist}"
            local binary_url="${base_url}/codecov"
            local checksum_url="${base_url}/codecov.SHA256SUM"
            local signature_url="${base_url}/codecov.SHA256SUM.sig"

            local tmp_dir="$(mktemp -d "${cache_dir}/codecov.tmp.XXXXXX" 2>/dev/null || true)"

            if [[ -z "${tmp_dir}" || ! -d "${tmp_dir}" ]]; then
                tmp_dir="${cache_dir}/codecov.tmp.$$"
                mkdir -p -- "${tmp_dir}" || die "Codecov: failed to create temp dir." 2
            fi

            trap 'rm -rf -- "${tmp_dir:-}" 2>/dev/null || true; trap - RETURN' RETURN

            local tmp_bin="${tmp_dir}/codecov"
            local tmp_sha="${tmp_dir}/codecov.SHA256SUM"
            local tmp_sig="${tmp_dir}/codecov.SHA256SUM.sig"

            rm -f -- "${tmp_bin}" "${tmp_sha}" "${tmp_sig}" 2>/dev/null || true

            run curl -fsSL -o "${tmp_bin}" "${binary_url}"
            run curl -fsSL -o "${tmp_sha}" "${checksum_url}"
            curl -fsSL -o "${tmp_sig}" "${signature_url}" 2>/dev/null || rm -f -- "${tmp_sig}" 2>/dev/null || true
            if [[ -f "${tmp_sig}" ]] && has gpg; then

                local keyring="${tmp_dir}/trustedkeys.gpg"
                local keyfile="${tmp_dir}/codecov.pgp.asc"

                run curl -fsSL -o "${keyfile}" "https://keybase.io/codecovsecops/pgp_keys.asc"
                gpg --no-default-keyring --keyring "${keyring}" --import "${keyfile}" >/dev/null 2>&1 || true
                gpg --no-default-keyring --keyring "${keyring}" --verify "${tmp_sig}" "${tmp_sha}" >/dev/null 2>&1 || die "Codecov: SHA256SUM signature verification failed." 2

            fi

            local got="" want="$(awk '$2 ~ /(^|\/)codecov$/ { print $1; exit }' "${tmp_sha}" 2>/dev/null || true)"

            [[ -n "${want}" ]] || die "Codecov: invalid SHA256SUM file." 2

            if has sha256sum; then got="$(sha256sum "${tmp_bin}" 2>/dev/null | awk '{print $1}' || true)"
            elif has shasum; then got="$(shasum -a 256 "${tmp_bin}" 2>/dev/null | awk '{print $1}' || true)"
            elif has openssl; then got="$(openssl dgst -sha256 "${tmp_bin}" 2>/dev/null | awk '{print $NF}' || true)"
            else die "Codecov: no SHA256 tool found (need sha256sum or shasum or openssl)." 2
            fi

            [[ -n "${got}" ]] || die "Codecov: failed to compute checksum." 2
            [[ "${got}" == "${want}" ]] || die "Codecov: checksum mismatch." 2

            run chmod +x "${tmp_bin}"
            run mv -f -- "${tmp_bin}" "${bin}"
            run "${bin}" --version >/dev/null 2>&1

        fi

        local -a args=( --verbose upload-process --disable-search --fail-on-error -t "${token}" -f "${out}" )

        [[ -n "${flags}" ]] && args+=( -F "${flags}" )
        [[ -n "${name}"  ]] && args+=( -n "${name}" )

        run "${bin}" "${args[@]}"
        success "Ok: Codecov file upload."

    fi

    success "OK Codecov processed -> ${out}"

}

cmd_samply () {

    ensure samply
    set_perf_paranoid

    source <(parse "$@" -- \
        bin test bench example toolchain out="out/samply.json" nightly:bool stable:bool msrv:bool save_only:bool \
        rate address duration package:list \
    )

    [[ -z "${bin}"  || -z "${example}" ]] || die "samply: use only one of --bin or --example" 2
    [[ -z "${bench}" || -z "${test}"   ]] || die "samply: use only one of --bench or --test" 2
    [[ -z "${bench}${example}"         ]] || die "samply: use only one of --bench or --example" 2

    local -a args=( samply record )
    local -a cargo=( cargo )
    local -a pkgs=()
    local -A seen=()
    local p=""

    (( stable  )) && toolchain="stable"
    (( nightly )) && toolchain="nightly"
    (( msrv    )) && toolchain="msrv"

    if [[ "${toolchain}" == "stable" ]]; then cargo+=( +"$(stable_version)" )
    elif [[ "${toolchain}" == "nightly" ]]; then cargo+=( +"$(nightly_version)" )
    elif [[ "${toolchain}" == "msrv" ]]; then cargo+=( +"$(msrv_version)" )
    elif [[ -n "${toolchain}" ]]; then cargo+=( +"${toolchain}" )
    fi

    if [[ -n "${bench}" ]]; then cargo+=( bench --bench "${bench}" )
    elif [[ -n "${example}" ]]; then cargo+=( run --example "${example}" )
    elif [[ -n "${test}" ]]; then cargo+=( test --test "${test}" )
    else cargo+=( run ); [[ -n "${bin}" ]] && cargo+=( --bin "${bin}" )
    fi

    for p in "${package[@]-}"; do

        [[ -n "${p}" ]] || continue
        [[ -n "${seen[${p}]-}" ]] && continue

        seen["${p}"]=1
        pkgs+=( -p "${p}" )

    done

    (( ${#seen[@]} <= 1 )) || die "samply: --package supports at most one package" 2

    (( save_only )) && args+=( --save-only )

    [[ -n "${rate}"  ]] && args+=( --rate "${rate}" )
    [[ -n "${address}"  ]] && args+=( --address "${address}" )
    [[ -n "${duration}"  ]] && args+=( --duration "${duration}" )

    [[ -n "${out}"  ]] && args+=( -o "${out}" )
    [[ "${out}" == */* ]] && run mkdir -p -- "${out%/*}"
    : > "${out}"

    CARGO_PROFILE_RELEASE_DEBUG=true \
        RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes -g" \
        run "${args[@]}" -- "${cargo[@]}" "${pkgs[@]}" "${kwargs[@]}"

}
cmd_samply_load () {

    ensure samply
    source <(parse "$@" -- :file="out/samply.json")

    [[ -f "${file}" ]] || die "file not found: ${file}" 2
    run samply load "${file}"

}

cmd_flame () {

    ensure flamegraph
    set_perf_flame

    source <(parse "$@" -- \
        bin test bench example toolchain out="out/flamegraph.svg" nightly:bool stable:bool msrv:bool package:list \
    )

    [[ -z "${bin}"  || -z "${example}" ]] || die "flame: use only one of --bin or --example" 2
    [[ -z "${bench}" || -z "${test}"   ]] || die "flame: use only one of --bench or --test" 2
    [[ -z "${bench}${example}"         ]] || die "flame: use only one of --bench or --example" 2

    local -a cargo=( cargo )
    local -a args=( flamegraph )
    local -a pkgs=()
    local -A seen=()
    local p=""

    (( stable  )) && toolchain="stable"
    (( nightly )) && toolchain="nightly"
    (( msrv    )) && toolchain="msrv"

    if [[ "${toolchain}" == "stable" ]]; then cargo+=( +"$(stable_version)" )
    elif [[ "${toolchain}" == "nightly" ]]; then cargo+=( +"$(nightly_version)" )
    elif [[ "${toolchain}" == "msrv" ]]; then cargo+=( +"$(msrv_version)" )
    elif [[ -n "${toolchain}" ]]; then cargo+=( +"${toolchain}" )
    fi

    if [[ -n "${bench}" ]]; then args+=( --bench "${bench}" )
    elif [[ -n "${example}" ]]; then args+=( --example "${example}" )
    elif [[ -n "${test}" ]]; then args+=( --test "${test}" )
    else [[ -n "${bin}" ]] && args+=( --bin "${bin}" )
    fi

    for p in "${package[@]-}"; do

        [[ -n "${p}" ]] || continue
        [[ -n "${seen[${p}]-}" ]] && continue

        seen["${p}"]=1
        pkgs+=( -p "${p}" )

    done

    (( ${#seen[@]} <= 1 )) || die "flame: --package supports at most one package" 2

    [[ -n "${out}"  ]] && args+=( -o "${out}" )
    [[ "${out}" == */* ]] && run mkdir -p -- "${out%/*}"
    : > "${out}"

    CARGO_PROFILE_RELEASE_DEBUG=true \
        RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes -g" \
        run "${cargo[@]}" "${args[@]}" "${pkgs[@]}" "${kwargs[@]}"

}
cmd_flame_open () {

    ensure flamegraph
    source <(parse "$@" -- :file="out/flamegraph.svg")

    [[ -f "${file}" ]] || die "file not found: ${file}" 2
    open_path "${file}"

}
