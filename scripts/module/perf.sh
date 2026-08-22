#!/usr/bin/env bash

CODECOV_PGP_URL="${CODECOV_PGP_URL:-https://keybase.io/codecovsecops/pgp_keys.asc}"
CODECOV_PGP_FPR="${CODECOV_PGP_FPR:-27034E7FDB850E0BBC2C62FF806BB28AED779869}"

perf_one_target () {

    local label="${1-}" bin="${2-}" test="${3-}" bench="${4-}" example="${5-}"
    local target="" count=0

    for target in "${bin}" "${test}" "${bench}" "${example}"; do
        [[ -n "${target}" ]] && count=$(( count + 1 ))
    done

    (( count <= 1 )) || die "${label}: use only one of --bin, --test, --bench or --example" 2

    return 0

}
perf_toolchain () {

    local toolchain="${1-}" stable="${2:-0}" nightly="${3:-0}" msrv="${4:-0}"

    (( stable  )) && toolchain="stable"
    (( nightly )) && toolchain="nightly"
    (( msrv    )) && toolchain="msrv"

    case "${toolchain}" in
        "") return 0 ;;
        stable)  printf '+%s\n' "$(stable_version)" ;;
        nightly) printf '+%s\n' "$(nightly_version)" ;;
        msrv)    printf '+%s\n' "$(msrv_version)" ;;
        *)       printf '+%s\n' "${toolchain}" ;;
    esac

}
perf_pkg_args () {

    local -n _out="${1}"
    local label="${2-}"

    shift 2 || true

    local -A seen=()
    local p=""

    _out=()

    for p in "$@"; do

        [[ -n "${p}" ]] || continue
        [[ -n "${seen[${p}]-}" ]] && continue

        seen["${p}"]=1
        _out+=( -p "${p}" )

    done

    (( ${#seen[@]} <= 1 )) || die "${label}: --package supports at most one package" 2

    return 0

}
set_perf_paranoid () {

    [[ "$(os_name)" == "linux" ]] || return 0

    local paranoid_file="/proc/sys/kernel/perf_event_paranoid"
    [[ -r "${paranoid_file}" ]] || return 0

    local current_val=""
    current_val="$(tr -d ' \t\r\n' < "${paranoid_file}" 2>/dev/null || true)"

    [[ -n "${current_val}" ]] || return 0
    [[ "${current_val}" =~ ^-?[0-9]+$ ]] || { warn "perf_event_paranoid: unexpected value '${current_val}'"; return 0; }

    (( current_val <= 1 )) && return 0

    info "Kernel perf_event_paranoid=${current_val} (too restrictive for profiling; need <= 1)."

    has sudo || die "perf_event_paranoid=${current_val} and sudo is not installed. Run: echo 1 | sudo tee ${paranoid_file}" 2

    if run sudo sysctl -w kernel.perf_event_paranoid=1; then
        success "perf_event_paranoid set to 1."
        return 0
    fi

    die "Failed to change perf_event_paranoid. Try: echo 1 | sudo tee ${paranoid_file}" 2

}
set_perf_flame () {

    [[ "$(os_name)" == "linux" ]] || return 0

    ensure linux-tools-common linux-tools-generic linux-cloud-tools-generic

    local k="" real="" link=""

    k="$(uname -r)"
    real="$(readlink -f /usr/lib/linux-tools/*/perf 2>/dev/null | head -n 1 || true)"

    [[ -n "${real}" && -x "${real}" ]] || die "perf: no perf binary under /usr/lib/linux-tools; install linux-tools for kernel ${k}." 2

    link="/usr/lib/linux-tools/${k}/perf"

    if [[ "$(readlink -f -- "${link}" 2>/dev/null || true)" != "${real}" ]]; then

        has sudo || die "perf: sudo is required to link ${real} -> ${link}." 2

        run sudo mkdir -p -- "/usr/lib/linux-tools/${k}" || die "perf: failed to create /usr/lib/linux-tools/${k}." 2
        run sudo ln -sf -- "${real}" "${link}" || die "perf: failed to link ${link}." 2

    fi

    export PERF="${real}"

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

    local staged="" version="" meta="" target_dir=""

    stage_file staged "${out}"
    trap 'fs_tmp_release "${staged%/*}"; trap - RETURN' RETURN

    version="$(cmd_version)" || die "bloat: failed to read workspace version" 2
    new_file "${staged}"

    printf '\n%s\n' "---------------------------------------" >> "${staged}"
    printf '%s' "- Bloats Report: " >> "${staged}"

    [[ -n "${max_size}" ]] && printf '%s' " Max-Size ( ${max_size} )" >> "${staged}"

    printf '\n%s\n' "- Version: ${version}" >> "${staged}"
    printf '%s\n\n' "---------------------------------------" >> "${staged}"

    meta="$(run_cargo metadata --no-deps --format-version 1 2>/dev/null)" || die "bloat: failed to get metadata" 2
    target_dir="$(jq -r '.target_directory' <<<"${meta}" 2>/dev/null || true)"

    [[ -n "${target_dir}" ]] || die "bloat: failed to read target_directory" 2

    local mod="debug" flag="--dev"
    (( release )) && { mod="release"; flag="--release"; }

    local exe="" pkg="" out_text="" i=1 had_error=0
    [[ "$(os_name)" == "windows" ]] && exe=".exe"

    for pkg in "${pkgs[@]}"; do

        printf '%s\n' "Analyzing : ${pkg} ..."

        printf '%d) %s:\n\n' "${i}" "${pkg}" >> "${staged}"
        (( i++ ))

        local -a bins=()
        mapfile -t bins < <(jq -r --arg n "${pkg}" '.packages[] | select(.name == $n) | .targets[] | select(.kind | index("bin")) | .name' <<<"${meta}")

        if (( ${#bins[@]} > 0 )); then

            local x="" bin_name="${bins[0]}"
            for x in "${bins[@]}"; do [[ "${x}" == "${pkg}" ]] && { bin_name="${x}"; break; }; done

            local bin_path="${target_dir}/${mod}/${bin_name}${exe}"

            if out_text="$(NO_COLOR=1 CARGO_TERM_COLOR=never run_cargo bloat -p "${pkg}" --bin "${bin_name}" "${flag}" "${kwargs[@]}" 2>&1)"; then

                awk '{ sub(/\r$/, "") } !on && /^[[:space:]]*File[[:space:]]/ { on=1 } on { print }' <<<"${out_text}" >> "${staged}"
                printf '\n' >> "${staged}"

                if [[ -n "${max_size}" ]]; then
                    check_max_size "${bin_path}" "${max_size}" || had_error=1
                fi

            else

                printf 'ERROR: %s\n\n' "can't resolve ${bin_path}" >> "${staged}"

            fi

        else

            local manifest_path="" adapter="${pkg}" candidate="" bin_name="" feature="" bin_path="" succeeded=0
            local -a adapters=()

            manifest_path="$(jq -r --arg n "${pkg}" '.packages[] | select(.name == $n) | .manifest_path' <<<"${meta}")"

            if [[ -n "${manifest_path}" && "${manifest_path}" != "null" && "${manifest_path}" == *[/\\]* ]]; then

                local crate_dir="${manifest_path%[/\\]*}"
                [[ "${crate_dir}" == *[/\\]* ]] && adapter="${crate_dir##*[/\\]}"

            fi

            adapters=( "${adapter}" )
            [[ "${adapter}" == "${pkg}" ]] || adapters+=( "${pkg}" )

            for candidate in "${adapters[@]}"; do
                feature="bloat-${candidate}"

                for bin_name in "bloat-${candidate}" "${candidate}"; do
                    if out_text="$(NO_COLOR=1 CARGO_TERM_COLOR=never run_cargo bloat -p bloats --bin "${bin_name}" --features "${feature}" "${flag}" "${kwargs[@]}" 2>&1)"; then
                        bin_path="${target_dir}/${mod}/${bin_name}${exe}"

                        awk '{ sub(/\r$/, "") } !on && /^[[:space:]]*File[[:space:]]/ { on=1 } on { print }' <<<"${out_text}" >> "${staged}"
                        printf '\n' >> "${staged}"

                        if [[ -n "${max_size}" ]]; then
                            check_max_size "${bin_path}" "${max_size}" || had_error=1
                        fi

                        succeeded=1
                        break 2
                    fi
                done
            done

            if (( ! succeeded )); then
                printf 'ERROR: cargo bloat failed for %s (via bloats)\n%s\n\n' "${pkg}" "${out_text}" >> "${staged}"
                had_error=1
            fi

        fi

    done

    move_file "${staged}" "${out}"

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

    else

        args+=( --lcov )
        out="${out:-out/lcov.info}"

    fi

    local staged="" tmp_dir=""

    stage_file staged "${out}"

    trap '
        fs_tmp_release "${staged%/*}"
        fs_tmp_release "${tmp_dir}"

        trap - RETURN
    ' RETURN

    run_cargo llvm-cov clean --workspace
    run_cargo llvm-cov --workspace --all-targets --all-features "${args[@]}" --output-path "${staged}" --remap-path-prefix "${kwargs[@]}"

    move_file "${staged}" "${out}"

    if (( upload )); then

        ensure curl chmod mv mkdir

        [[ -n "${flags}" ]] || flags="crates"
        [[ -n "${name}"  ]] || name="coverage-${GITHUB_RUN_ID:-local}"

        [[ -n "${version}" ]] || version="latest"
        [[ -n "${version}" && "${version}" != "latest" && "${version}" != v* ]] && version="v${version}"

        local required=0
        is_ci && ! is_ci_pull && required=1

        [[ -n "${token}" ]] || token="${CODECOV_TOKEN:-}"

        if [[ -z "${token}" ]]; then

            (( required )) && die "codecov: CODECOV_TOKEN is missing." 2

            warn "codecov: CODECOV_TOKEN is missing."
            return 0

        fi
        if [[ ! -f "${out}" ]]; then

            (( required )) && die "codecov: report not found: ${out}" 2

            warn "codecov: file not found: ${out}"
            return 0

        fi

        local arch="" dist="" exe="" os=""

        arch="$(uname -m)"
        os="$(os_name)"

        case "${os}" in
            linux)
                dist="linux"
                if [[ "${arch}" == "aarch64" || "${arch}" == "arm64" ]]; then dist="linux-arm64"; fi
            ;;
            mac)     dist="macos" ;;
            windows) dist="windows"; exe=".exe" ;;
            *)       die "codecov: unsupported platform: ${os}/${arch}" 2 ;;
        esac

        local cache_dir="${ROOT_DIR}/.codecov/cache"
        ensure_dir "${cache_dir}"

        local resolved="${version}"
        local bin="${cache_dir}/codecov-${dist}-${resolved}${exe}"

        if [[ "${version}" == "latest" ]]; then

            local latest_page="" v=""

            latest_page="$(curl -fsSL "https://cli.codecov.io/${dist}/latest" 2>/dev/null || true)"
            v="$(printf '%s\n' "${latest_page}" | grep -Eo 'v[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true)"

            [[ -n "${v}" ]] && resolved="${v}"
            bin="${cache_dir}/codecov-${dist}-${resolved}${exe}"

        fi
        if [[ ! -x "${bin}" ]]; then

            local asset="codecov${exe}"
            local base_url="https://cli.codecov.io/${resolved}/${dist}"
            local binary_url="${base_url}/${asset}"
            local checksum_url="${base_url}/${asset}.SHA256SUM"
            local signature_url="${base_url}/${asset}.SHA256SUM.sig"

            fs_tmp_dir tmp_dir codecov

            local tmp_bin="${tmp_dir}/${asset}"
            local tmp_sha="${tmp_dir}/${asset}.SHA256SUM"
            local tmp_sig="${tmp_dir}/${asset}.SHA256SUM.sig"

            run curl -fsSL -o "${tmp_bin}" "${binary_url}"
            run curl -fsSL -o "${tmp_sha}" "${checksum_url}"

            if (( required )) || { has gpg && curl -fsSL -o "${tmp_sig}" "${signature_url}" 2>/dev/null; }; then

                local keyring="${tmp_dir}/trustedkeys.gpg"
                local keyfile="${tmp_dir}/codecov.pgp.asc"
                local fprs=""

                has gpg || die "Codecov: gpg is required to verify the release signature." 2

                [[ -f "${tmp_sig}" ]] || run curl -fsSL -o "${tmp_sig}" "${signature_url}"
                run curl -fsSL -o "${keyfile}" "${CODECOV_PGP_URL}"

                gpg --no-default-keyring --keyring "${keyring}" --import "${keyfile}" >/dev/null 2>&1 || die "Codecov: failed to import the verification key." 2

                fprs="$(gpg --no-default-keyring --keyring "${keyring}" --with-colons --fingerprint 2>/dev/null || true)"
                [[ "${fprs}" == *"fpr:::::::::${CODECOV_PGP_FPR}:"* ]] || die "Codecov: verification key fingerprint mismatch (expected ${CODECOV_PGP_FPR})." 2

                gpg --no-default-keyring --keyring "${keyring}" --verify "${tmp_sig}" "${tmp_sha}" >/dev/null 2>&1 || die "Codecov: SHA256SUM signature verification failed." 2

            fi

            local got="" want=""
            want="$(awk -v asset="${asset}" '
                { name = $2; sub(/^\*/, "", name); sub(/.*\//, "", name) }
                name == asset { print $1; exit }
            ' "${tmp_sha}" 2>/dev/null || true)"

            [[ -n "${want}" ]] || die "Codecov: invalid SHA256SUM file." 2

            got="$(tool_sha256 "${tmp_bin}")" || die "Codecov: failed to compute checksum." 2

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

    perf_one_target samply "${bin}" "${test}" "${bench}" "${example}"

    local -a args=( samply record )
    local -a cargo=( cargo )
    local -a pkgs=()
    local tc="" staged=""

    tc="$(perf_toolchain "${toolchain}" "${stable}" "${nightly}" "${msrv}")"
    [[ -n "${tc}" ]] && cargo+=( "${tc}" )

    if [[ -n "${bench}" ]]; then cargo+=( bench --bench "${bench}" )
    elif [[ -n "${example}" ]]; then cargo+=( run --example "${example}" )
    elif [[ -n "${test}" ]]; then cargo+=( test --test "${test}" )
    else cargo+=( run ); [[ -n "${bin}" ]] && cargo+=( --bin "${bin}" )
    fi

    perf_pkg_args pkgs samply "${package[@]-}"

    (( save_only )) && args+=( --save-only )

    [[ -n "${rate}"  ]] && args+=( --rate "${rate}" )
    [[ -n "${address}"  ]] && args+=( --address "${address}" )
    [[ -n "${duration}"  ]] && args+=( --duration "${duration}" )

    if [[ -n "${out}" ]]; then

        stage_file staged "${out}"
        trap 'fs_tmp_release "${staged%/*}"; trap - RETURN' RETURN

        args+=( -o "${staged}" )

    fi

    CARGO_PROFILE_RELEASE_DEBUG=true \
        RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes -g" \
        run "${args[@]}" -- "${cargo[@]}" "${pkgs[@]}" "${kwargs[@]}"

    if [[ -n "${staged}" ]]; then
        move_file "${staged}" "${out}"
    fi

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

    perf_one_target flame "${bin}" "${test}" "${bench}" "${example}"

    local -a cargo=( cargo )
    local -a args=( flamegraph )
    local -a pkgs=()
    local tc="" staged=""

    tc="$(perf_toolchain "${toolchain}" "${stable}" "${nightly}" "${msrv}")"
    [[ -n "${tc}" ]] && cargo+=( "${tc}" )

    if [[ -n "${bench}" ]]; then args+=( --bench "${bench}" )
    elif [[ -n "${example}" ]]; then args+=( --example "${example}" )
    elif [[ -n "${test}" ]]; then args+=( --test "${test}" )
    else [[ -n "${bin}" ]] && args+=( --bin "${bin}" )
    fi

    perf_pkg_args pkgs flame "${package[@]-}"

    if [[ -n "${out}" ]]; then

        stage_file staged "${out}"
        trap 'fs_tmp_release "${staged%/*}"; trap - RETURN' RETURN

        args+=( -o "${staged}" )

    fi

    CARGO_PROFILE_RELEASE_DEBUG=true \
        RUSTFLAGS="${RUSTFLAGS:-} -C force-frame-pointers=yes -g" \
        run "${cargo[@]}" "${args[@]}" "${pkgs[@]}" "${kwargs[@]}"

    if [[ -n "${staged}" ]]; then
        move_file "${staged}" "${out}"
    fi

}
cmd_flame_open () {

    ensure flamegraph
    source <(parse "$@" -- :file="out/flamegraph.svg")

    [[ -f "${file}" ]] || die "file not found: ${file}" 2
    open_path "${file}"

}
