#!/usr/bin/env bash

[[ "${BASH_SOURCE[0]}" != "${0}" ]] || { printf '%s\n' "tool.sh: this file should not be run externally." >&2; exit 2; }
[[ -n "${TOOL_LOADED:-}" ]] && return 0
TOOL_LOADED=1

__dir="${BASH_SOURCE[0]%/*}"
[[ "${__dir}" == "${BASH_SOURCE[0]}" ]] && __dir="."
__core_dir="$(cd -- "${__dir}" && pwd -P)"
source "${__core_dir}/pkg.sh"

tool_docflags_deny () {

    local cur="${RUSTDOCFLAGS:-}"

    [[ "${cur}" == *"-Dwarnings"* ]] && { printf '%s' "${cur}"; return 0; }
    [[ -n "${cur}" ]] && { printf '%s -Dwarnings' "${cur}"; return 0; }

    printf '%s' "-Dwarnings"

}
tool_export_cargo_bin () {

    local cargo_home=""
    cargo_home="$(unix_path "${CARGO_HOME:-${HOME}/.cargo}")"

    path_prepend "${cargo_home}/bin"

    [[ -n "${GITHUB_PATH:-}" ]] && printf '%s\n' "${cargo_home}/bin" >> "${GITHUB_PATH}"
    return 0

}
tool_pick_sort_locale () {

    local line=""

    if has locale; then

        while IFS= read -r line; do
            case "${line}" in
                C.UTF-8)     printf '%s\n' "C.UTF-8"; return 0 ;;
                en_US.UTF-8) printf '%s\n' "en_US.UTF-8"; return 0 ;;
            esac
        done < <( locale -a 2>/dev/null || true )

    fi

    printf '%s\n' "C"

}
tool_pick_sort_bin () {

    ensure_pkg sort 1>&2

    LC_ALL=C sort -V </dev/null >/dev/null 2>&1 && { printf '%s\n' "sort"; return 0; }
    die "sort: no version sort (-V) support; install GNU coreutils and re-run" 2

}
tool_sort_ver () {

    local loc="" sbin=""

    loc="$(tool_pick_sort_locale)"
    sbin="$(tool_pick_sort_bin)"

    LC_ALL="${loc}" "${sbin}" -V

}
tool_sort_uniq () {

    local loc="" sbin=""

    loc="$(tool_pick_sort_locale)"
    sbin="$(tool_pick_sort_bin)"

    LC_ALL="${loc}" "${sbin}" -u

}
tool_normalize_version () {

    local tc="${1:-}"
    tc="${tc#v}"

    case "${tc}" in
        stable|beta|nightly) printf '%s\n' "${tc}"; return 0 ;;
        nightly-????-??-??)  printf '%s\n' "${tc}"; return 0 ;;
    esac

    [[ "${tc}" =~ ^[0-9]+\.[0-9]+$ ]] && tc="${tc}.0"
    [[ "${tc}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Invalid version: ${1}" 2

    printf '%s\n' "${tc}"

}
tool_active_version () {

    local out=""
    out="$(rustup show active-toolchain 2>/dev/null || true)"

    out="${out%%$'\n'*}"
    printf '%s\n' "${out%% *}"

}
tool_stable_version () {

    tool_normalize_version "${RUST_STABLE:-stable}"

}
tool_nightly_version () {

    tool_normalize_version "${RUST_NIGHTLY:-nightly}"

}
tool_msrv_version () {

    ensure_pkg jq awk sed tail sort 1>&2

    local tc="" want="" have=""

    if [[ -n "${RUST_MSRV:-}" ]]; then

        tc="$(tool_normalize_version "${RUST_MSRV}")"
        [[ "${tc}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Invalid RUST_MSRV (need x.y.z): ${RUST_MSRV}" 2

        printf '%s\n' "${tc}"
        return 0

    fi

    have="$(rustc -V 2>/dev/null | awk '{print $2}' | sed 's/[^0-9.].*$//' || true)"
    [[ -n "${have}" ]] || die "rustc not available to detect current version" 2

    if has cargo; then

        want="$(
            cargo metadata --no-deps --format-version 1 2>/dev/null \
                | jq -r '.packages[].rust_version // empty' \
                | tool_sort_ver \
                | tail -n 1
        )"

    fi

    [[ -n "${want}" ]] || { printf '%s\n' "${have}"; return 0; }

    tc="$(tool_normalize_version "${want}")"

    [[ "${tc}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Invalid workspace rust_version (need x.y.z): ${want}" 2
    [[ "$(printf '%s\n%s\n' "${tc}" "${have}" | tool_sort_ver | awk 'NR==1{print;exit}')" == "${tc}" ]] || die "Rust too old: need >= ${tc}, have ${have}" 2

    printf '%s\n' "${tc}"

}
tool_resolve_chain () {

    local tc="${1:-}"
    [[ -n "${tc}" ]] || die "tool_resolve_chain: needs a toolchain" 2

    case "${tc}" in
        stable)  tc="$(tool_stable_version)" ;;
        nightly) tc="$(tool_nightly_version)" ;;
        msrv)    tc="$(tool_msrv_version)" ;;
    esac

    printf '%s\n' "${tc}"

}
tool_has_component () {

    local tc="${1-}" comp="${2-}" line=""

    [[ -n "${tc}" && -n "${comp}" ]] || return 1

    while IFS= read -r line; do

        line="${line%$'\r'}"
        [[ "${line}" == "${comp}" || "${line}" == "${comp}-"* ]] && return 0

    done < <( rustup component list --toolchain "${tc}" --installed 2>/dev/null || true )

    return 1

}
tool_setup_chain () {

    local tc=""
    tc="$(tool_resolve_chain "${1:-}")"
    [[ -n "${tc}" ]] || die "tool_setup_chain: empty toolchain" 2

    rustup run "${tc}" rustc -V >/dev/null 2>&1 && return 0
    run rustup toolchain install "${tc}" --profile minimal
    run rustup run "${tc}" rustc -V >/dev/null 2>&1 || die "rustc not working after install: ${tc}" 2

}

ensure_python () {

    ensure_pkg python pip
    pkg_hash_clear

    local py=""

    if has python3; then py="python3"
    elif has python; then py="python"
    else die "ensure_python: python not found after install" 2
    fi

    run "${py}" -c 'import sys; sys.exit(0)' >/dev/null 2>&1 || die "ensure_python: python exists but failed to run" 2
    run "${py}" -m pip --version >/dev/null 2>&1 || run "${py}" -m ensurepip --upgrade >/dev/null 2>&1 || true
    run "${py}" -m pip --version >/dev/null 2>&1 || die "ensure_python: pip not available" 2
    run "${py}" -m pip install --user --upgrade pip setuptools wheel --disable-pip-version-check >/dev/null 2>&1 || true
    run "${py}" -m pip --version >/dev/null 2>&1 || die "ensure_python: pip exists but failed to run" 2

}
ensure_node () {

    local want="${1:-25}" v="" major="" volta_dir=""

    if has node; then

        v="$(node --version 2>/dev/null || true)"
        v="${v#v}"
        major="${v%%.*}"

        [[ "${major}" =~ ^[0-9]+$ ]] || die "Can't parse Node.js version: ${v}" 2
        (( major >= want )) && has npx && npx --version >/dev/null 2>&1 && return 0

    fi
    case "$(os_name)" in
        windows)
            has volta || {
                has winget || die "volta: winget not found" 2
                run powershell.exe -NoProfile -Command \
                    'winget install -e --id Volta.Volta --accept-source-agreements --accept-package-agreements --disable-interactivity' \
                    || die "Failed to install Volta (winget)" 2
            }

            volta_dir="$(unix_path "${PROGRAMFILES:-${ProgramFiles:-C:\\Program Files}}")/Volta"
            path_prepend "${volta_dir}"
        ;;
        *)
            ensure_pkg curl 1>&2

            export VOLTA_HOME="${VOLTA_HOME:-${HOME}/.volta}"
            volta_dir="${VOLTA_HOME}/bin"
            path_prepend "${volta_dir}"

            if ! has volta; then run curl --proto '=https' --tlsv1.2 -fsSL https://get.volta.sh | bash || die "Failed to install Volta." 2; fi
            path_prepend "${volta_dir}"
        ;;
    esac

    has volta || die "Volta installed but not found in PATH." 2
    run volta install "node@${want}" || die "Failed to install Node via Volta." 2

    pkg_hash_clear

    has node || die "Node install finished but 'node' not found in PATH." 2
    has npx  || die "npx not found after Node install." 2
    npx --version >/dev/null 2>&1 || die "npx exists but failed to run. Check environment/PATH." 2

    v="$(node --version 2>/dev/null || true)"
    v="${v#v}"
    major="${v%%.*}"

    [[ "${major}" =~ ^[0-9]+$ ]] || die "Can't parse Node.js version after install: ${v}" 2
    (( major >= want )) || die "Node install did not satisfy requirement (need ${want}+, found v${v})." 2

    [[ -n "${GITHUB_PATH:-}" && -n "${volta_dir}" ]] && printf '%s\n' "${volta_dir}" >> "${GITHUB_PATH}"

    return 0

}
ensure_rust () {

    ensure_pkg curl 1>&2

    local stable="" nightly="" msrv="" os=""

    stable="$(tool_stable_version)"
    nightly="$(tool_nightly_version)"
    msrv="$(tool_msrv_version)"
    os="$(os_name)"

    tool_export_cargo_bin

    if ! has rustup; then

        case "${os}" in
            windows)
                local tmp_dir="" tmp="" arch=""

                case "$(uname -m 2>/dev/null || true)" in
                    x86_64|amd64)  arch="x86_64" ;;
                    aarch64|arm64) arch="aarch64" ;;
                    i686|i386)     arch="i686" ;;
                    *) die "rustup: unsupported Windows architecture: $(uname -m 2>/dev/null || printf '%s' unknown)" 2 ;;
                esac

                fs_tmp_dir tmp_dir "rustx-rustup"
                tmp="${tmp_dir}/rustup-init.exe"

                run curl --proto '=https' --tlsv1.2 -fsSL -o "${tmp}" "https://win.rustup.rs/${arch}" || die "Failed to download rustup-init.exe" 2
                run "${tmp}" -y --profile minimal --default-toolchain "${stable}" || die "Failed to install rustup (Windows)" 2

                fs_tmp_release "${tmp_dir}"
            ;;
            mac|linux)
                run curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
                    | sh -s -- -y --profile minimal --default-toolchain "${stable}" \
                    || die "Failed to install rustup." 2
            ;;
            *)
                die "Unsupported OS for rustup install: ${os}" 2
            ;;
        esac

        tool_export_cargo_bin

        if [[ -f "${HOME}/.cargo/env" ]]; then
            source "${HOME}/.cargo/env"
        fi

        has rustup || die "rustup installed but not found in PATH (check ~/.cargo/bin)." 2

    fi

    tool_setup_chain "${stable}"
    tool_setup_chain "${nightly}"
    tool_setup_chain "${msrv}"

    run rustup run "${stable}"  cargo -V >/dev/null 2>&1 || die "cargo (stable) not working after install." 2
    run rustup run "${nightly}" rustc -V >/dev/null 2>&1 || die "rustc (nightly) not working after install." 2
    run rustup run "${msrv}"    rustc -V >/dev/null 2>&1 || die "rustc (msrv) not working after install." 2

}
ensure_component () {

    local comp="${1:-}" tc="${2:-}"

    [[ -n "${comp}" ]] || die "ensure_component: requires a component name" 2
    [[ -z "${tc}" ]] && tc="stable"

    has rustup || ensure_rust

    tc="$(tool_resolve_chain "${tc}")"
    tool_setup_chain "${tc}"

    if [[ "${comp}" == "llvm-tools-preview" ]]; then

        tool_has_component "${tc}" llvm-tools-preview && return 0
        tool_has_component "${tc}" llvm-tools && return 0

        run rustup component add --toolchain "${tc}" llvm-tools-preview 2>/dev/null || run rustup component add --toolchain "${tc}" llvm-tools
        return 0

    fi

    tool_has_component "${tc}" "${comp}" && return 0
    run rustup component add --toolchain "${tc}" "${comp}"

}
ensure_crate () {

    local crate="${1:-}" bin="${2:-}"
    shift 2 || true

    [[ -n "${crate}" ]] || die "ensure_crate requires <crate>" 2
    [[ -n "${bin}" ]]   || die "ensure_crate requires <bin>" 2

    has cargo || ensure_rust
    has cargo-binstall || run cargo install cargo-binstall || die "Failed to install cargo-binstall" 2

    tool_export_cargo_bin

    { has "${bin}" || has "${bin#cargo-}"; } && return 0

    if (( $# == 0 )); then

        local -a extra=()
        is_ci && extra+=( --no-confirm --force )

        run cargo binstall "${crate}" "${extra[@]}"

        { has "${bin}" || has "${bin#cargo-}"; } && return 0

    fi

    run cargo install "${crate}" "$@"

}
tool_sha256 () {

    local file="${1:-}"
    [[ -f "${file}" ]] || die "sha256: file not found: ${file}" 2

    if has sha256sum; then
        sha256sum "${file}" | awk '{ print $1 }'
        return "${PIPESTATUS[0]}"
    fi
    if has shasum; then
        shasum -a 256 "${file}" | awk '{ print $1 }'
        return "${PIPESTATUS[0]}"
    fi
    if has openssl; then
        openssl dgst -sha256 "${file}" | awk '{ print $NF }'
        return "${PIPESTATUS[0]}"
    fi

    die "sha256: need sha256sum, shasum, or openssl" 2

}
tool_release_asset () {

    local tool="${1:-}" version="${2:-}" os="" arch=""
    os="$(os_name)"
    arch="$(uname -m 2>/dev/null || true)"
    [[ -n "${tool}" && -n "${version}" ]] || die "release asset: missing tool/version" 2

    case "${arch}" in
        x86_64|amd64) arch="amd64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) die "${tool}: unsupported architecture: ${arch}" 2 ;;
    esac

    case "${tool}:${os}:${arch}" in
        trivy:linux:amd64)    printf 'trivy_%s_Linux-64bit.tar.gz\n' "${version}" ;;
        trivy:linux:arm64)    printf 'trivy_%s_Linux-ARM64.tar.gz\n' "${version}" ;;
        trivy:mac:amd64)      printf 'trivy_%s_macOS-64bit.tar.gz\n' "${version}" ;;
        trivy:mac:arm64)      printf 'trivy_%s_macOS-ARM64.tar.gz\n' "${version}" ;;
        trivy:windows:amd64)  printf 'trivy_%s_windows-64bit.zip\n' "${version}" ;;
        trivy:windows:arm64)  printf 'trivy_%s_windows-ARM64.zip\n' "${version}" ;;

        syft:linux:amd64)     printf 'syft_%s_linux_amd64.tar.gz\n' "${version}" ;;
        syft:linux:arm64)     printf 'syft_%s_linux_arm64.tar.gz\n' "${version}" ;;
        syft:mac:amd64)       printf 'syft_%s_darwin_amd64.tar.gz\n' "${version}" ;;
        syft:mac:arm64)       printf 'syft_%s_darwin_arm64.tar.gz\n' "${version}" ;;
        syft:windows:amd64)   printf 'syft_%s_windows_amd64.zip\n' "${version}" ;;
        syft:windows:arm64)   printf 'syft_%s_windows_arm64.zip\n' "${version}" ;;

        gitleaks:linux:amd64)    printf 'gitleaks_%s_linux_x64.tar.gz\n' "${version}" ;;
        gitleaks:linux:arm64)    printf 'gitleaks_%s_linux_arm64.tar.gz\n' "${version}" ;;
        gitleaks:mac:amd64)      printf 'gitleaks_%s_darwin_x64.tar.gz\n' "${version}" ;;
        gitleaks:mac:arm64)      printf 'gitleaks_%s_darwin_arm64.tar.gz\n' "${version}" ;;
        gitleaks:windows:amd64)  printf 'gitleaks_%s_windows_x64.zip\n' "${version}" ;;
        gitleaks:windows:arm64)  printf 'gitleaks_%s_windows_arm64.zip\n' "${version}" ;;

        *) die "${tool}: unsupported platform: ${os}/${arch}" 2 ;;
    esac

}
ensure_release_tool () {

    local tool="${1:-}" repo="" version="" manifest_sha=""
    [[ -n "${tool}" ]] || die "ensure_release_tool: missing tool" 2

    has "${tool}" && return 0

    case "${tool}" in
        trivy)
            repo="aquasecurity/trivy"
            version="0.73.0"
            manifest_sha="36890275ffdff13025e9bd9fe039724c6e36bf58e698499856b801f619046fe2"
        ;;
        syft)
            repo="anchore/syft"
            version="1.50.0"
            manifest_sha="bb8824a06c27c625fc103db5d7e9d7131ba2cc6e7c7a79318ee71686ede3c3f0"
        ;;
        gitleaks)
            repo="gitleaks/gitleaks"
            version="8.30.1"
            manifest_sha="061476c21adaf5441516f96f185c1a4706a83cd6329b9b38762271b3d4a52fae"
        ;;
        *) die "release tool: unsupported tool: ${tool}" 2 ;;
    esac

    ensure curl awk chmod mkdir mktemp mv rm

    local asset="" temp="" got_manifest="" expected="" got_asset=""
    local manifest="${tool}_${version}_checksums.txt"
    local base="https://github.com/${repo}/releases/download/v${version}"
    local bin_dir="${RUSTX_BIN_DIR:-${HOME}/.local/bin}"
    local exe="${tool}"

    asset="$(tool_release_asset "${tool}" "${version}")" || return $?
    [[ "$(os_name)" == "windows" ]] && exe+=".exe"

    fs_tmp_dir temp "rustx-${tool}"

    run curl --proto '=https' --tlsv1.2 -fsSL -o "${temp}/${manifest}" "${base}/${manifest}"
    run curl --proto '=https' --tlsv1.2 -fsSL -o "${temp}/${asset}" "${base}/${asset}"

    got_manifest="$(tool_sha256 "${temp}/${manifest}")" || die "${tool}: failed to hash release checksum manifest" 2
    [[ "${got_manifest}" == "${manifest_sha}" ]] || die "${tool}: release checksum manifest mismatch" 2

    expected="$(awk -v asset="${asset}" '$2 == asset || $2 == "*" asset { print $1; exit }' "${temp}/${manifest}")"
    got_asset="$(tool_sha256 "${temp}/${asset}")" || die "${tool}: failed to hash release asset" 2

    [[ -n "${expected}" ]] || die "${tool}: asset missing from checksum manifest: ${asset}" 2
    [[ "${got_asset}" == "${expected}" ]] || die "${tool}: release asset checksum mismatch" 2

    local unpack="${temp}/unpack"
    run mkdir -p -- "${unpack}"

    case "${asset}" in
        *.tar.gz)
            ensure tar
            run tar -xzf "${temp}/${asset}" -C "${unpack}" "${exe}"
        ;;
        *.zip)
            ensure unzip
            run unzip -j -q "${temp}/${asset}" "${exe}" -d "${unpack}"
        ;;
        *) die "${tool}: unsupported archive: ${asset}" 2 ;;
    esac

    [[ -f "${unpack}/${exe}" ]] || die "${tool}: binary missing after extraction" 2

    run mkdir -p -- "${bin_dir}"
    run chmod 0755 -- "${unpack}/${exe}"
    run mv -f -- "${unpack}/${exe}" "${bin_dir}/${exe}"

    path_prepend "${bin_dir}"
    [[ -n "${GITHUB_PATH:-}" ]] && printf '%s\n' "${bin_dir}" >> "${GITHUB_PATH}"

    fs_tmp_release "${temp}"

    has "${tool}" || die "${tool}: installed but not found in PATH" 2

    run "${tool}" --version >/dev/null 2>&1 \
        || run "${tool}" version >/dev/null 2>&1 \
        || die "${tool}: installed at ${bin_dir}/${exe} but will not run" 2

    return 0

}
ensure () {

    local yes=0 quiet=0 verbose=0 want=""

    source <(parse "$@" -- --yes:bool --quiet:bool --verbose:bool :wants:list)

    (( ${#wants[@]} )) || return 0

    (( YES_ENV || yes )) && YES_ENV=1
    (( QUIET_ENV || quiet )) && QUIET_ENV=1
    (( VERBOSE_ENV || verbose )) && VERBOSE_ENV=1

    for want in "${wants[@]}"; do

        case "${want}" in
            python*) has python || has python3 || ensure_python ;;
            pip*) has pip || has pip3 || ensure_python ;;
            node|nodejs|npx|npm|pnpm|volta) has node || ensure_node ;;
            cargo|rust|rustc|rustup) has cargo || ensure_rust ;;
            rustfmt|rust-src) ensure_component "${want}"; ensure_component "${want}" nightly ;;
            miri) ensure_component "${want}" nightly ;;
            clippy|llvm-tools-preview) ensure_component "${want}" ;;
            taplo) ensure_crate taplo-cli taplo ;;
            typos) ensure_crate typos-cli typos --locked --version 1.49.0 ;;
            trivy|syft|gitleaks) ensure_release_tool "${want}" ;;
            cargo-audit) ensure_crate cargo-audit cargo-audit --features fix ;;
            cargo-upgrade|cargo-edit) ensure_crate cargo-edit cargo-upgrade ;;
            samply|flamegraph|cargo-*) ensure_crate "${want}" "${want}" ;;
            *) has "${want}" || ensure_pkg "${want}" 1>&2 ;;
        esac

    done

}
