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
        "new                        * Create a new crate and (optionally) add it to the workspace, for not publish add (--no-publish)" \
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

    local resolved="$(resolve_cmd "${name}")" || true
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

cmd_new () {

    ensure perl
    source <(parse "$@" -- :name:str dir:str="crates" kind:str="--lib" publish:bool=true workspace:bool=true )

    local path="${dir}/${name}"

    [[ -e "${path}" ]] && die "Crate already exists: ${path}" 2
    [[ "${name}" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]] || die "Invalid crate name: ${name}" 2

    mkdir -p -- "${dir}" 2>/dev/null || true
    run_cargo new --vcs none "${kind}" "${kwargs[@]}" "${path}"

    local crate_toml="${path}/Cargo.toml"
    [[ -f "${crate_toml}" ]] || die "Cargo.toml not found: ${crate_toml}" 2

    if (( publish == 0 )); then

        perl -i -ne '
            our $nl;
            $nl //= (/\r\n$/ ? "\r\n" : "\n");

            our $in_pkg;
            our $inserted;

            if (/^\[package\]\s*\r?$/) {
                $in_pkg = 1;
                $inserted = 0;
                print;
                next;
            }
            if ($in_pkg) {

                if (/^\[[^\]]+\]\s*\r?$/) {
                    if (!$inserted) { print "publish = false$nl"; $inserted = 1; }
                    $in_pkg = 0;
                    print;
                    next;
                }
                if (/^[ \t]*publish\s*=/) {
                    next;
                }
                if (!$inserted && /^[ \t]*name\s*=/) {
                    print;
                    print "publish = false$nl";
                    $inserted = 1;
                    next;
                }

                print;
                next;
            }

            print;

            END {
                if ($in_pkg && !$inserted) {
                    print "publish = false$nl";
                }
            }
        ' "${crate_toml}" || die "Failed to set publish=false in ${crate_toml}" 2

    else

        perl -i -ne '
            our $nl;
            $nl //= (/\r\n$/ ? "\r\n" : "\n");

            our $in_pkg;
            our $has_categories;
            our $inserted;

            if (/^\[package\]\s*\r?$/) {
                $in_pkg = 1;
                $has_categories = 0;
                $inserted = 0;
                print;
                next;
            }
            if ($in_pkg) {

                if (/^[ \t]*categories\s*=/) {
                    $has_categories = 1;
                    print;
                    next;
                }
                if (/^\[[^\]]+\]\s*\r?$/) {
                    if (!$has_categories && !$inserted) {
                        print "categories = [\"development-tools\"]$nl";
                        $inserted = 1;
                    }
                    $in_pkg = 0;
                    print;
                    next;
                }
                if (!$has_categories && !$inserted && /^[ \t]*name\s*=/) {
                    print;
                    print "categories = [\"development-tools\"]$nl";
                    $inserted = 1;
                    next;
                }

                print;
                next;

            }

            print;

            END {
                if ($in_pkg && !$has_categories && !$inserted) {
                    print "categories = [\"development-tools\"]$nl";
                }
            }
        ' "${crate_toml}" || die "Failed to set default categories in ${crate_toml}" 2

    fi

    [[ ${workspace} -eq 1 ]] || return 0
    [[ -f Cargo.toml ]] || return 0

    grep -qF "\"${dir}/${name}\"" Cargo.toml 2>/dev/null && return 0

    MEMBER="${dir}/${name}" perl -0777 -i -pe '
        my $m = $ENV{MEMBER};
        my $ws = qr/\[workspace\]/s;

        if ($_ !~ $ws) { next; }

        if ($_ =~ /members\s*=\s*\[(.*?)\]/s) {
            my $block = $1;

            if ($block !~ /\Q$m\E/s) {
                s/(members\s*=\s*\[)(.*?)(\])/$1.$2."\n    \"$m\",\n".$3/se;
            }
        }
        else {
            s/(\[workspace\]\s*)/$1."members = [\n    \"$m\",\n]\n"/se;
        }
    ' Cargo.toml

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

    run_workspace check "$@"

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

    if [[ -f "${ROOT_DIR}/target/doc/index.html" ]]; then open_path "${ROOT_DIR}/target/doc/index.html"
    else open_path "$(find "${ROOT_DIR}/target/doc" -maxdepth 2 -name index.html -print | head -n 1 || true)"
    fi

}
