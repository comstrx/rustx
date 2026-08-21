#!/usr/bin/env bash

git_help () {

    info_ln "Git :\n"

    printf '    %s\n' \
        "init                       * Initialize git repo + remote config" \
        "push                       * Commit/tag then push (branch/tag/release)" \
        "remote                     * Manage git remote (add/set/show)" \
        "changelog                  * Prepend a CHANGELOG entry for a tag" \
        "" \
        "all-branches               * List branches (local/remote)" \
        "all-tags                   * List tags (local/remote)" \
        "" \
        "default-branch             * Print default branch name" \
        "current-branch             * Print current checked-out branch" \
        "switch-branch              * Switch/create/track a branch" \
        "" \
        "new-release                * Create & push a new release tag" \
        "remove-release             * Delete a release tag (local/remote)" \
        "" \
        "new-branch                 * Create a new branch (local/remote)" \
        "remove-branch              * Delete a branch (local/remote)" \
        ''

}
cmd_git_help () {

    info_ln "Git :\n"

    printf '    %s\n' \
        "init                       * Init git with a repo, repo will created if not exists" \
        "    --repo                   Repository full name to init <user/reponame>" \
        "    --branch                 Branch name, Default: current (fallback: main)" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --auth                   Auth type <auto|ssh|http>, Default: auto" \
        "" \
        "push                       * Commit/tag then push (branch/tag/release)" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --branch                 Branch name, Default: current (fallback: main)" \
        "    --message                Commit/tag message" \
        "    --auth                   Auth type <auto|ssh|http>, Default: auto" \
        "    --key                    SSH key path (ssh mode only)" \
        "    --token                  Token value (http mode)" \
        "    --token-env              Token env var name (default: GITHUB_TOKEN)" \
        "    --tag                    Tag name (auto normalizes to vX.Y.Z if semver)" \
        "    --release                Create & push release tag (auto from project version)" \
        "    --force                  Force push (with lease) + overwrite tag" \
        "    --changelog              Prepend CHANGELOG entry (requires tag/release)" \
        "" \
        "remote                     * Show current remote" \
        "    --remote                 Remote origin name, Default: origin" \
        "" \
        "changelog                  * Prepend CHANGELOG entry for a tag" \
        "    --tag                    Tag name (auto normalizes to vX.Y.Z if semver)" \
        "    --message                Commit/tag message" \
        "" \
        "all-branches               * List branches (local/remote)" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --only-local             Only local branches" \
        "" \
        "all-tags                   * List tags (local/remote)" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --only-local             Only local tags" \
        "" \
        "default-branch             * Print default branch name" \
        "" \
        "current-branch             * Print current checked-out branch" \
        "" \
        "switch-branch              * Switch/create/track a branch" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --branch                 Branch name, Default: current (fallback: main)" \
        "    --auth                   Auth type <auto|ssh|http>, Default: auto" \
        "    --key                    SSH key path (ssh mode only)" \
        "    --token                  Token value (http mode)" \
        "    --token-env              Token env var name (default: GITHUB_TOKEN)" \
        "    --track                  Create local tracking branch" \
        "    --create                 Create branch if not exists" \
        "" \
        "new-release                * Create a new tag/release and push it" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --branch                 Branch name, Default: current (fallback: main)" \
        "    --message                Commit/tag message" \
        "    --auth                   Auth type <auto|ssh|http>, Default: auto" \
        "    --key                    SSH key path (ssh mode only)" \
        "    --token                  Token value (http mode)" \
        "    --token-env              Token env var name (default: GITHUB_TOKEN)" \
        "    --tag                    Tag name (auto normalizes to vX.Y.Z if semver)" \
        "    --force                  Force push (with lease) + overwrite tag" \
        "    --changelog              Prepend CHANGELOG entry (requires tag/release)" \
        "" \
        "remove-release             * Remove tag/release from remote" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --tag                    Tag name (auto normalizes to vX.Y.Z if semver)" \
        "    --auth                   Auth type <auto|ssh|http>, Default: auto" \
        "    --key                    SSH key path (ssh mode only)" \
        "    --token                  Token value (http mode)" \
        "    --token-env              Token env var name (default: GITHUB_TOKEN)" \
        "" \
        "new-branch                 * Create branch on remote" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --branch                 Branch name, Default: current (fallback: main)" \
        "    --auth                   Auth type <auto|ssh|http>, Default: auto" \
        "    --key                    SSH key path (ssh mode only)" \
        "    --token                  Token value (http mode)" \
        "    --token-env              Token env var name (default: GITHUB_TOKEN)" \
        "" \
        "remove-branch              * Remove branch from remote" \
        "    --remote                 Remote origin name, Default: origin" \
        "    --branch                 Branch name, Default: current (fallback: main)" \
        "    --auth                   Auth type <auto|ssh|http>, Default: auto" \
        "    --key                    SSH key path (ssh mode only)" \
        "    --token                  Token value (http mode)" \
        "    --token-env              Token env var name (default: GITHUB_TOKEN)" \
        ''

}

cmd_init () {

    ensure_pkg git 2>&1
    source <(parse "$@" -- :repo branch=main remote=origin auth=ssh)

    local host="" path="" url="" parsed=0
    [[ -z "${auth}" ]] && auth="$(get_env "${GITHUB_AUTH_ENV}")"
    case "${auth}" in ssh|http|auto) ;; *) die "Unknown auth: ${auth} (use ssh|http|auto)" 2 ;; esac

    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then

        if git_init_supports_initial_branch; then
            run git init -b "${branch}"
        else
            run git init
            git_set_default_branch "${branch}"
        fi

    fi

    cmd_new_repo "${repo}" || true

    if [[ "${repo}" != *"://"* && "${repo}" != git@*:* && "${repo}" != ssh://* && "${repo}" == */* ]]; then
        host="github.com"
        path="${repo}"
        parsed=1
    else
        if read -r host path < <(git_parse_remote "${repo}"); then
            parsed=1
        fi
    fi

    if (( parsed )); then

        path="$(git_norm_path_git "${path}")"

        if [[ "${auth}" == "ssh" ]]; then url="$(git_build_ssh_url "${host}" "${path}")" || die "Can't build ssh url" 2
        else url="$(git_build_https_url "${host}" "${path}")" || die "Can't build https url" 2
        fi

    else

        url="${repo}"

    fi

    if git remote get-url "${remote}" >/dev/null 2>&1; then run git remote set-url "${remote}" "${url}"
    else run git remote add "${remote}" "${url}"
    fi

    git_set_default_branch "${branch}"
    success "OK: branch='${branch}', remote='${remote}' -> $(git_redact_url "${url}")"

}
cmd_remote () {

    git_repo_guard
    source <(parse "$@" -- remote=origin)

    local url="$(git_remote_url "${remote}")"
    [[ -n "${url}" ]] || die "Remote not found: ${remote}" 2

    info "${remote}: $(git_redact_url "${url}")"

    if [[ "${url}" == https://* || "${url}" == http://* ]]; then
        info "Protocol: HTTPS"
        return 0
    fi
    if [[ "${url}" == git@*:* || "${url}" == ssh://* ]]; then
        info "Protocol: SSH"
        return 0
    fi

    warn "Protocol: unknown"

}
cmd_push () {

    git_repo_guard
    source <(parse "$@" -- remote=origin auth key token token_env branch message tag t force:bool f:bool changelog:bool log:bool release:bool)

    git_require_remote "${remote}"
    local kind="" target="" safe="" ssh_cmd=""

    IFS=$'\t' read -r kind target safe ssh_cmd < <(git_auth_resolve "${auth}" "${remote}" "${key}" "${token}" "${token_env}")
    [[ -n "${kind}" && -n "${target}" ]] || die "Failed to resolve git auth for remote '${remote}'." 2

    (( f )) && force=1
    (( log )) && changelog=1
    [[ -z "${tag}" ]] && tag="${t}"
    (( release )) && [[ -z "${tag}" ]] && tag="auto"

    if [[ -n "${tag}" ]]; then
        [[ "${tag}" == "auto" ]] && tag="v$(git_root_version)"
        tag="$(git_norm_tag "${tag}")"
        [[ -z "${message}" ]] && message="Track ${tag} release."
    fi
    if [[ -z "${branch}" ]]; then
        branch="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
        [[ -n "${branch}" ]] || branch="main"
    fi
    if [[ -z "$message" ]]; then
        message="new commit"
    fi

    git_guard_no_unborn_nested_repos "${ROOT_DIR}"
    run_git "${kind}" "${ssh_cmd}" add -A || die "git add failed." 2

    if run_git "${kind}" "${ssh_cmd}" diff --cached --quiet >/dev/null 2>&1; then

        git_has_commit || die "Nothing to push: no commits yet. Make changes then run: push" 2

    else

        git_require_identity
        run_git "${kind}" "${ssh_cmd}" commit -m "${message}" || die "git commit failed." 2

    fi

    if [[ -n "${tag}" ]]; then

        if git_remote_has_tag "${kind}" "${ssh_cmd}" "${target}" "${tag}" && (( force == 0 )); then

            log "Tag exists on remote (${remote}/${tag}). Use --force to overwrite."
            tag=""; changelog=0

        else

            if (( changelog )); then

                cmd_changelog "${tag}" "${message}"
                run_git "${kind}" "${ssh_cmd}" add -A

                if ! run_git "${kind}" "${ssh_cmd}" diff --cached --quiet >/dev/null 2>&1; then

                    git_require_identity
                    run_git "${kind}" "${ssh_cmd}" commit -m "Track ${tag} release." || die "git commit failed." 2

                fi

            fi

        fi

    fi

    local target_is_url=0
    [[ "${target}" == http://* || "${target}" == https://* ]] && target_is_url=1

    if (( force )); then

        run_git "${kind}" "${ssh_cmd}" fetch "${target}" "${branch}" >/dev/null 2>&1 || true
        run_git "${kind}" "${ssh_cmd}" push --force-with-lease "${target}" "${branch}" || die "push rejected. fetch/pull first." 2

    else

        if (( target_is_url )); then

            run_git "${kind}" "${ssh_cmd}" push "${target}" "${branch}" || die "push rejected. Run: git pull --rebase ${remote} ${branch}" 2

        else

            if git_upstream_exists_for "${branch}"; then
                run_git "${kind}" "${ssh_cmd}" push "${target}" "${branch}" || die "push rejected. Run: git pull --rebase ${remote} ${branch}" 2
            else
                run_git "${kind}" "${ssh_cmd}" push -u "${target}" "${branch}" || die "push rejected. Run: git pull --rebase ${remote} ${branch}" 2
            fi

        fi

    fi

    if [[ -n "${tag}" ]]; then

        run_git "${kind}" "${ssh_cmd}" tag -d "${tag}" >/dev/null 2>&1 || true

        if (( force )); then
            run_git "${kind}" "${ssh_cmd}" push "${target}" --delete "${tag}" >/dev/null 2>&1 || true
        fi

        run_git "${kind}" "${ssh_cmd}" tag -a "${tag}" -m "${message}" || die "tag create failed." 2

        if (( force )); then run_git "${kind}" "${ssh_cmd}" push --force "${target}" "${tag}" || die "tag push failed." 2
        else run_git "${kind}" "${ssh_cmd}" push "${target}" "${tag}" || die "tag push failed." 2
        fi

    fi

    success "OK: pushed via ${kind} -> ${safe}"

}
cmd_changelog () {

    ensure_pkg grep mktemp mv date tail 2>&1

    local tag="${1:-unreleased}"
    local msg="${2:-}"

    [[ "${tag}" =~ ^v[0-9] ]] && tag="${tag#v}"
    [[ -n "${msg}" ]] || msg="Track ${tag} release."

    msg="${msg//$'\r'/ }"
    msg="${msg//$'\n'/ }"

    local day="$(date -u +%Y-%m-%d)"
    local header="## 💥 ${tag} ( ${day} )"
    local block="${header}"$'\n\n'"- ${msg}"
    local file="${ROOT_DIR}/CHANGELOG.md"
    local tmp="$(mktemp "${TMPDIR:-/tmp}/git.XXXXXX")"

    if [[ -f "${file}" ]]; then

        local top=""
        IFS= read -r top < "${file}" 2>/dev/null || true

        if [[ "${top}" != "# Changelog" ]]; then
            { printf '%s\n\n' "# Changelog"; cat "${file}"; } > "${tmp}"
            mv -f "${tmp}" "${file}"
            tmp="$(mktemp)" || die "changelog: mktemp failed" 2
        fi

        local first="$(tail -n +2 "${file}" 2>/dev/null | grep -m1 -E '^[[:space:]]*## ' || true)"

        if [[ "${first}" == "${header}" ]]; then
            log "changelog: already written -> skip"
            return 0
        fi

        {
            printf '%s\n\n' "# Changelog"
            printf '%s\n' "${block}"
            tail -n +2 "${file}"
        } > "${tmp}"

    else

        {
            printf '%s\n\n' "# Changelog"
            printf '%s\n' "${block}"
        } > "${tmp}"

    fi

    mv -f "${tmp}" "${file}"
    success "changelog: updated ${file}"

}

cmd_all_branches () {

    git_repo_guard
    source <(parse "$@" -- remote=origin only_local:bool)

    if (( only_local )); then
        git for-each-ref --format='%(refname:short)' "refs/heads"
        return 0
    fi

    ensure_pkg awk 2>&1
    git_require_remote "${remote}"

    GIT_TERMINAL_PROMPT=0 git fetch --prune "${remote}" >/dev/null 2>&1 || true
    git for-each-ref --format='%(refname:short)' "refs/heads" "refs/remotes/${remote}" | awk '!/\/HEAD$/'

}
cmd_all_tags () {

    git_repo_guard
    source <(parse "$@" -- remote=origin only_local:bool auth key token token_env)

    if (( only_local )); then
        git tag --list
        return 0
    fi

    git_require_remote "${remote}"

    local kind="" target="" safe="" ssh_cmd=""
    IFS=$'\t' read -r kind target safe ssh_cmd < <(git_auth_resolve "${auth}" "${remote}" "${key}" "${token}" "${token_env}")

    ensure_pkg awk 2>&1
    run_git "${kind}" "${ssh_cmd}" ls-remote --tags --refs "${target}" | awk '{ sub("^refs/tags/","",$2); print $2 }'

}

cmd_default_branch () {

    git_repo_guard

    local b="$(git_default_branch "origin")" || die "Can't detect default branch." 2
    [[ -n "${b}" ]] || die "No branch checked out." 2

    info "${b}"

}
cmd_current_branch () {

    git_repo_guard

    local b="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
    [[ -n "${b}" ]] || die "No branch checked out." 2

    info "${b}"

}
cmd_switch_branch () {

    git_repo_guard
    source <(parse "$@" -- :branch remote=origin auth key token token_env create:bool track:bool=true)

    if git show-ref --verify --quiet "refs/heads/${branch}"; then
        git_switch "${branch}"
        return 0
    fi

    local have_remote=0
    git remote get-url "${remote}" >/dev/null 2>&1 && have_remote=1

    if (( track )) && (( have_remote )); then

        local kind="" target="" safe="" ssh_cmd=""

        IFS=$'\t' read -r kind target safe ssh_cmd < <(git_auth_resolve "${auth}" "${remote}" "${key}" "${token}" "${token_env}")
        [[ -n "${kind}" && -n "${target}" ]] || die "Failed to resolve git auth for remote '${remote}'." 2

        if git_remote_has_branch "${kind}" "${ssh_cmd}" "${target}" "${branch}"; then

            run_git "${kind}" "${ssh_cmd}" fetch "${target}" "refs/heads/${branch}:refs/remotes/${remote}/${branch}" >/dev/null 2>&1 || true
            git_switch -c "${branch}" --track "${remote}/${branch}"
            return 0

        fi

    fi

    (( create )) || die "Branch not found: ${branch}. Use --create to create locally." 2
    git_switch -c "${branch}"

}

cmd_new_release () {

    git_repo_guard
    source <(parse "$@" -- :tag)
    cmd_push --tag "${tag}" "${kwargs[@]}"

}
cmd_remove_release () {

    git_repo_guard
    source <(parse "$@" -- :tag remote=origin auth key token token_env)

    tag="$(git_norm_tag "${tag}")"
    confirm "Delete tag '${tag}' locally and on '${remote}'?" || return 0

    run git tag -d "${tag}" >/dev/null 2>&1 || true

    local have_remote=0
    git remote get-url "${remote}" >/dev/null 2>&1 && have_remote=1
    (( have_remote )) || return 0

    local kind="" target="" safe="" ssh_cmd=""
    IFS=$'\t' read -r kind target safe ssh_cmd < <(git_auth_resolve "${auth}" "${remote}" "${key}" "${token}" "${token_env}")

    run_git "${kind}" "${ssh_cmd}" push "${target}" --delete "${tag}" >/dev/null 2>&1 || true

}

cmd_new_branch () {

    git_repo_guard
    source <(parse "$@" -- :branch remote=origin auth key token token_env)

    if git show-ref --verify --quiet "refs/heads/${branch}"; then
        git_switch "${branch}"
        return 0
    fi

    local have_remote=0
    git remote get-url "${remote}" >/dev/null 2>&1 && have_remote=1

    if (( have_remote )); then

        local kind="" target="" safe="" ssh_cmd=""
        IFS=$'\t' read -r kind target safe ssh_cmd < <(git_auth_resolve "${auth}" "${remote}" "${key}" "${token}" "${token_env}")

        if git_remote_has_branch "${kind}" "${ssh_cmd}" "${target}" "${branch}"; then

            run_git "${kind}" "${ssh_cmd}" fetch "${target}" "refs/heads/${branch}:refs/remotes/${remote}/${branch}" >/dev/null 2>&1 || true
            git_switch -c "${branch}" --track "${remote}/${branch}"
            return 0

        fi

    fi

    git_switch -c "${branch}"

}
cmd_remove_branch () {

    git_repo_guard
    source <(parse "$@" -- :branch remote=origin auth key token token_env)

    local cur="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
    [[ "${cur}" != "${branch}" ]] || die "Can't delete current branch: ${branch}" 2

    confirm "Delete branch '${branch}' locally and on '${remote}'?" || return 0
    run git branch -D "${branch}" >/dev/null 2>&1 || true

    local have_remote=0
    git remote get-url "${remote}" >/dev/null 2>&1 && have_remote=1
    (( have_remote )) || return 0

    local kind="" target="" safe="" ssh_cmd=""
    IFS=$'\t' read -r kind target safe ssh_cmd < <(git_auth_resolve "${auth}" "${remote}" "${key}" "${token}" "${token_env}")

    run_git "${kind}" "${ssh_cmd}" push "${target}" --delete "${branch}" >/dev/null 2>&1 || true

}
