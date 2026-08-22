#!/usr/bin/env bash

cmd_crate_help () {

    info_ln "Crate :\n"

    printf '    %s\n' \
        "active                     * Show current active version" \
        "stable                     * Show stable version" \
        "nightly                    * Show nightly version" \
        "msrv                       * Show msrv version" \
        "" \
        "list                       * List of installed cargo tools/crates" \
        "install                    * Install crate/s" \
        "uninstall                  * Uninstall crate/s" \
        "install-update             * Install/Update cargo tool/s into latest version" \
        "installed                  * Installed List of cargo tools" \
        "show                       * Show <package/tool/crate> info, version if installed" \
        "" \
        "add                        * Add new crate/s into <--package *>" \
        "remove                     * remove crate/s from <--package *>" \
        "update                     * Update crate/s" \
        "upgrade                    * Upgrade crate/s into latest version" \
        "info                       * Information about <*crate-name*>" \
        "search                     * Search in crates store <*crate-name*>" \
        "" \
        "new                        * Scaffold a crate in the workspace anatomy and register it everywhere" \
        "build                      * Build the whole workspace, or a single crate if specified" \
        "run                        * Run a binary (use -p/--package to pick a crate, or pass a bin name)" \
        "clean                      * Clean Cargo" \
        "clean-cache                * Clean cache ( cargo-ci-cache-clean )" \
        "tree                       * Show list of cargo tree dependencies (cargo tree -e normal)" \
        "tree-files                 * Show tree files structures of workspace (tree -a)" \
        "has                        * Check if workspace/package has a specific dependency" \
        "expand                     * Expand crate code ( expand macros/derive )" \
        "" \
        "check                      * Run compile checks for all crates and targets (no binaries produced)" \
        "test                       * Run the full test suite (workspace-wide or a single crate)" \
        "bench                      * Run benchmarks (workspace-wide or a single crate)" \
        "example                    * Run an example target by name, forwarding extra args after --" \
        "" \
        "doc-check                  * Check docs after build it strictly (workspace or single crate)" \
        "doc-test                   * Test docs by Run documentation tests (doctests)" \
        "doc-open                   * Open docs in your browser after build it" \
        "doc-clean                  * Clean docs" \
        ''

}

cmd_active () {

    active_version

}
cmd_stable () {

    stable_version

}
cmd_nightly () {

    nightly_version

}
cmd_msrv () {

    msrv_version

}

cmd_list () {

    ensure cargo
    run cargo --list "$@"

}
cmd_install () {

    source <(parse "$@" -- :name:list)
    run_cargo install "${name[@]}" "${kwargs[@]}"

}
cmd_uninstall () {

    source <(parse "$@" -- :name:list)
    run_cargo uninstall "${name[@]}" "${kwargs[@]}"

}
cmd_install_update () {

    source <(parse "$@" -- :name:list="-a")
    ensure cargo-update cargo-install-update
    run_cargo install-update "${name[@]}" "${kwargs[@]}"

}
cmd_installed () {

    run_cargo install --list "$@"

}
cmd_show () {

    source <(parse "$@" -- :name:str)

    local resolved=""
    resolved="$(resolve_cmd "${name}")" || true

    [[ -n "${resolved}" ]] || { error "${name}: Not found."; return 1; }

    local -a cmd=()
    read -r -a cmd <<< "${resolved}"

    "${cmd[@]}" --version >/dev/null 2>&1 && { "${cmd[@]}" --version; return 0; }
    "${cmd[@]}" -V        >/dev/null 2>&1 && { "${cmd[@]}" -V;        return 0; }
    "${cmd[@]}" version   >/dev/null 2>&1 && { "${cmd[@]}" version;   return 0; }

    success "${resolved}: Installed."
    warn "${resolved}: can not detect version."

    return 0

}

cmd_add () {

    source <(parse "$@" -- :crate_name:list :package:str)
    run_cargo add "${crate_name[@]}" --package "${package}" "${kwargs[@]}"

}
cmd_remove () {

    source <(parse "$@" -- :crate_name:list :package:str)
    run_cargo rm "${crate_name[@]}" --package "${package}" "${kwargs[@]}"

}
cmd_update () {

    source <(parse "$@" -- crate_name:list)
    run_cargo update "${crate_name[@]}" "${kwargs[@]}"

}
cmd_upgrade () {

    source <(parse "$@" -- package:list)

    local -a args=()
    local p=""
    for p in "${package[@]}"; do args+=( "--package" "${p}" ); done

    run_cargo upgrade "${args[@]}" "${kwargs[@]}"

}
cmd_info () {

    source <(parse "$@" -- :crate_name:list)
    run_cargo info "${crate_name[@]}" "${kwargs[@]}"

}
cmd_search () {

    run_cargo search "$@"

}

crate_register () {

    local dir="${1-}" pkg="${2-}" version=""

    version="$(cmd_version)" || die "new: failed to read the workspace version" 2

    DIR="${dir}" PKG="${pkg}" VER="${version}" awk '
        BEGIN { member = "\"" ENVIRON["DIR"] "\"" }

        /^members = \[/ && !done_members {
            if (index($0, member) == 0) {
                sub(/\]$/, ", " member "]")
            }
            done_members = 1
        }

        /^\[workspace\.dependencies\]$/ { in_deps = 1; print; next }

        in_deps && /^$/ && !done_deps {
            print ENVIRON["PKG"] " = { path = \"" ENVIRON["DIR"] "\", version = \"" ENVIRON["VER"] "\" }"
            done_deps = 1
            in_deps = 0
        }

        { print }
    ' Cargo.toml > Cargo.toml.new || die "new: failed to register ${pkg}" 2

    run mv -- Cargo.toml.new Cargo.toml

    has taplo && run taplo fmt Cargo.toml >/dev/null 2>&1

    return 0

}
crate_reexport () {

    local facade="${1-}" extern="${2-}" alias="${3-}"
    local lib="${facade}/src/lib.rs" pre="${facade}/src/prelude.rs"

    grep -qF "pub use ${extern} as ${alias};" -- "${lib}" \
        || printf 'pub use %s as %s;\n' "${extern}" "${alias}" >> "${lib}"

    grep -qF "pub use ${extern}::prelude::*;" -- "${pre}" \
        || printf 'pub use %s::prelude::*;\n' "${extern}" >> "${pre}"

    return 0

}
cmd_new () {

    source <(parse "$@" -- :name:str)

    [[ "${name}" =~ ^[a-z][a-z0-9]*(-[a-z0-9]+)*$ ]] || die "new: crate name must be lower-kebab-case: ${name}" 2

    local dir="crates/${name}" pkg="rustx-${name}" extern="rustx_${name//-/_}"
    local facade="crates/rustx" title=""

    [[ -e "${dir}" ]] && die "new: crate already exists: ${dir}" 2
    [[ -d "${facade}" ]] || die "new: facade crate not found: ${facade}" 2

    title="$(printf '%s' "${name:0:1}" | tr '[:lower:]' '[:upper:]')${name:1}"

    ensure_dir "${dir}/src"

    printf '%s\n' \
        "[package]" \
        "name = \"${pkg}\"" \
        "version.workspace = true" \
        "edition.workspace = true" \
        "rust-version.workspace = true" \
        "readme.workspace = true" \
        "license.workspace = true" \
        "repository.workspace = true" \
        "homepage.workspace = true" \
        "authors.workspace = true" \
        "description = \"TODO: one line describing ${pkg}.\"" \
        "categories = [\"rust-patterns\"]" \
        "keywords.workspace = true" \
        "" \
        "[lints]" \
        "workspace = true" \
        "" \
        "[package.metadata.docs.rs]" \
        "all-features = true" \
        "rustdoc-args = [\"--generate-link-to-definition\"]" \
        > "${dir}/Cargo.toml"

    printf '%s\n' \
        "//! TODO: one line describing ${pkg}." \
        "" \
        "#![no_std]" \
        "" \
        "pub mod prelude;" \
        "" \
        "mod ${name//-/_};" \
        > "${dir}/src/lib.rs"

    printf '%s\n' \
        "//! The crate prelude." \
        "//!" \
        "//! Glob-import this module to bring the crate's vocabulary into scope without" \
        "//! naming each type. Everything re-exported here is part of the public" \
        "//! contract and follows the workspace's semver policy." \
        > "${dir}/src/prelude.rs"

    printf '%s\n' "//! TODO: implement ${title}." > "${dir}/src/${name//-/_}.rs"

    run cp -- LICENSE-MIT "${dir}/LICENSE-MIT"
    run cp -- LICENSE-APACHE "${dir}/LICENSE-APACHE"

    crate_register "${dir}" "${pkg}"
    crate_reexport "${facade}" "${extern}" "${name//-/_}"

    success_ln "Created ${pkg} at ${dir}\n"

    cmd_conform

}
cmd_build () {

    source <(parse "$@" -- package:list)

    local -a args=()
    local pkg=""
    for pkg in "${package[@]}"; do [[ -n "${pkg}" ]] && args+=( --package "${pkg}" ); done

    run_workspace build "${args[@]}" "${kwargs[@]}"

}
cmd_run () {

    source <(parse "$@" -- package bin)

    local -a args=()

    [[ -n "${package}" ]] && args+=( --package "${package}" )
    [[ -n "${bin}" ]] && args+=( --bin "${bin}" )

    run_cargo run "${args[@]}" "${kwargs[@]}"

}
cmd_clean () {

    source <(parse "$@" -- package:list)

    local -a args=()
    local pkg=""
    for pkg in "${package[@]}"; do [[ -n "${pkg}" ]] && args+=( --package "${pkg}" ); done

    run_cargo clean "${args[@]}" "${kwargs[@]}"

}
cmd_clean_cache () {

    run_cargo ci-cache-clean "$@"

}
cmd_tree () {

    run_cargo tree "$@"

}
cmd_tree_files () {

    ensure tree
    run tree -a -I ".git|target|Cargo.lock"

}
cmd_has () {

    source <(parse "$@" -- :keyword package:list p:list)

    local -a args=()
    local pkg=""

    for pkg in "${package[@]}"; do args+=( --package "${pkg}" ); done
    for pkg in "${p[@]}"; do args+=( --package "${pkg}" ); done

    run_cargo tree "${args[@]}" "${kwargs[@]}" | grep -nF -- "${keyword}"

}
cmd_expand () {

    source <(parse "$@" -- :package:list)

    local -a args=()
    local pkg=""

    for pkg in "${package[@]}"; do
        args+=( --package "${pkg}" )
    done

    run_cargo expand --nightly "${args[@]}" "${kwargs[@]}"

}

cmd_check () {

    run_workspace check targets-on "$@"

}
cmd_test () {

    if has cargo-nextest; then run_workspace nextest run "$@"
    else run_workspace test "$@"
    fi

}
cmd_bench () {

    run_workspace bench features-on "$@"

}
cmd_example () {

    source <(parse "$@" -- :name package p)

    local -a args=()
    package="${package:-${p:-}}"
    [[ -n "${package}" ]] && args+=( -p "${package}" )

    run_cargo run "${args[@]}" --example "${name}" "${kwargs[@]}"

}

cmd_doc_check () {

    run_workspace doc features-on deps-off "$@"

}
cmd_doc_test () {

    run_workspace test features-on --doc "$@"

}
cmd_doc_clean () {

    remove_dir "${ROOT_DIR}/target/doc"

}
cmd_doc_open () {

    run_workspace doc features-on deps-off "$@"

    local doc_dir="${ROOT_DIR}/target/doc"
    local index="${doc_dir}/index.html"

    if [[ ! -f "${index}" ]]; then
        index="$(find "${doc_dir}" -maxdepth 2 -name index.html -print 2>/dev/null | head -n 1 || true)"
    fi

    [[ -f "${index}" ]] || die "doc-open: no index.html generated under ${doc_dir}" 2

    open_path "${index}"

}
