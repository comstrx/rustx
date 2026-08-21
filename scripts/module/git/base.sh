#!/usr/bin/env bash

GITHUB_SSH_KEY_ENV="${GITHUB_SSH_KEY_ENV:-GITHUB_SSH_KEY}"
GITHUB_TOKEN_ENV="${GITHUB_TOKEN_ENV:-GITHUB_TOKEN}"
GITHUB_AUTH_ENV="${GITHUB_AUTH_ENV:-GITHUB_AUTH}"

run_git () {

    local kind="${1:-ssh}"
    local ssh_cmd="${2:-}"
    shift 2 || true

    if (( VERBOSE_ENV )) && [[ "${kind}" == "http" ]]; then

        local old="${VERBOSE_ENV}"
        VERBOSE_ENV=0

        if [[ -n "${ssh_cmd}" ]]; then
            GIT_TERMINAL_PROMPT=0 GIT_SSH_COMMAND="${ssh_cmd}" run git "$@"
        else
            GIT_TERMINAL_PROMPT=0 run git "$@"
        fi

        VERBOSE_ENV="${old}"
        return $?

    fi
    if [[ -n "${ssh_cmd}" ]]; then
        GIT_TERMINAL_PROMPT=0 GIT_SSH_COMMAND="${ssh_cmd}" run git "$@"
        return $?
    fi

    GIT_TERMINAL_PROMPT=0 run git "$@"

}
git_repo_guard () {

    ensure_pkg git 2>&1
    git rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "Not a git repository." 2

}
git_has_switch () {

    git switch -h >/dev/null 2>&1

}
git_switch () {

    if git_has_switch; then

        git switch "$@"
        return $?

    fi
    if [[ "${1:-}" == "-c" ]]; then

        shift || true
        local b="${1:-}"
        shift || true

        if [[ "${1:-}" == "--track" ]]; then

            shift || true
            local upstream="${1:-}"
            shift || true

            git checkout -b "${b}" --track "${upstream}" "$@"
            return $?

        fi

        git checkout -b "${b}" "$@"
        return $?

    fi

    git checkout "$@"

}
git_has_commit () {

    git rev-parse --verify HEAD >/dev/null 2>&1

}
git_require_remote () {

    local remote="${1:-origin}"
    git remote get-url "${remote}" >/dev/null 2>&1 || die "Remote not found: ${remote}. Run: init <user/repo>" 2

}
git_require_identity () {

    local n="" e=""

    n="$(git config user.name  2>/dev/null || true)"
    e="$(git config user.email 2>/dev/null || true)"

    [[ -n "${n}" && -n "${e}" ]] && return 0

    die "Missing git identity. Set: git config user.name \"Your Name\" && git config user.email \"you@example.com\"" 2

}
git_is_semver () {

    local v="${1:-}"
    local main="" rest="" pre="" build=""

    [[ -n "${v}" ]] || return 1

    if [[ "${v}" == *+* ]]; then
        main="${v%%+*}"
        build="${v#*+}"
    else
        main="${v}"
        build=""
    fi

    if [[ "${main}" == *-* ]]; then
        rest="${main%%-*}"
        pre="${main#*-}"
    else
        rest="${main}"
        pre=""
    fi

    if [[ "${rest}" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
        :
    else
        return 1
    fi

    if [[ -n "${pre}" ]]; then

        IFS='.' read -r -a ids <<< "${pre}"
        ((${#ids[@]})) || return 1

        local id=""
        for id in "${ids[@]}"; do

            [[ -n "${id}" ]] || return 1
            [[ "${id}" =~ ^[0-9A-Za-z-]+$ ]] || return 1

            if [[ "${id}" =~ ^[0-9]+$ ]]; then
                [[ "${id}" == "0" || "${id}" =~ ^[1-9][0-9]*$ ]] || return 1
            fi

        done

    fi

    if [[ -n "${build}" ]]; then

        IFS='.' read -r -a ids <<< "${build}"
        ((${#ids[@]})) || return 1

        local id=""
        for id in "${ids[@]}"; do
            [[ -n "${id}" ]] || return 1
            [[ "${id}" =~ ^[0-9A-Za-z-]+$ ]] || return 1
        done

    fi

    return 0

}
git_norm_tag () {

    local t="${1:-}"
    local core="${t}"

    [[ -n "${t}" ]] || { printf '%s\n' ""; return 0; }

    if [[ "${t}" == v* ]]; then

        core="${t#v}"
        git_is_semver "${core}" && { printf 'v%s\n' "${core}"; return 0; }
        printf '%s\n' "${t}"
        return 0

    fi

    git_is_semver "${t}" && { printf 'v%s\n' "${t}"; return 0; }
    printf '%s\n' "${t}"

}
git_redact_url () {

    local url="${1:-}"
    local proto="" rest=""

    [[ -n "${url}" ]] || { printf ''; return 0; }

    if [[ "${url}" == http://* || "${url}" == https://* ]]; then

        proto="${url%%://*}://"
        rest="${url#*://}"

        if [[ "${rest}" == *@* ]]; then
            printf '%s***@%s\n' "${proto}" "${rest#*@}"
            return 0
        fi

    fi

    printf '%s\n' "${url}"

}
git_remote_url () {

    local remote="${1:-origin}"
    git remote get-url "${remote}" 2>/dev/null || true

}
git_parse_remote () {

    local url="${1:-}"
    local rest="" host="" path=""

    [[ -n "${url}" ]] || return 1

    if [[ "${url}" == git@*:* ]]; then
        rest="${url#git@}"
        host="${rest%%:*}"
        path="${rest#*:}"
        printf '%s %s\n' "${host}" "${path}"
        return 0
    fi
    if [[ "${url}" == ssh://* ]]; then
        rest="${url#ssh://}"
        rest="${rest#git@}"
        host="${rest%%/*}"
        path="${rest#*/}"
        printf '%s %s\n' "${host}" "${path}"
        return 0
    fi
    if [[ "${url}" == http://* || "${url}" == https://* ]]; then
        rest="${url#*://}"
        [[ "${rest}" == *@* ]] && rest="${rest#*@}"
        host="${rest%%/*}"
        path="${rest#*/}"
        printf '%s %s\n' "${host}" "${path}"
        return 0
    fi

    return 1

}
git_build_https_token_url () {

    local token="${1:-}"
    local host="${2:-}"
    local path="${3:-}"

    [[ -n "${token}" && -n "${host}" && -n "${path}" ]] || return 1
    printf 'https://x-access-token:%s@%s/%s\n' "${token}" "${host}" "${path}"

}
git_upstream_exists_for () {

    local b="${1:-}"
    [[ -n "${b}" ]] || return 1
    git rev-parse --abbrev-ref --symbolic-full-name "${b}@{u}" >/dev/null 2>&1

}
git_remote_has_tag () {

    local kind="${1:-ssh}"
    local ssh_cmd="${2:-}"
    local target="${3:-origin}"
    local tag="${4:-}"

    [[ -n "${tag}" ]] || return 1

    run_git "${kind}" "${ssh_cmd}" ls-remote --exit-code --tags --refs "${target}" "refs/tags/${tag}" >/dev/null 2>&1

}
git_remote_has_branch () {

    local kind="${1:-ssh}"
    local ssh_cmd="${2:-}"
    local target="${3:-origin}"
    local b="${4:-}"

    [[ -n "${b}" ]] || return 1
    run_git "${kind}" "${ssh_cmd}" ls-remote --exit-code --heads "${target}" "${b}" >/dev/null 2>&1

}
git_auth_resolve () {

    local auth="${1:-auto}"
    local remote="${2:-origin}"
    local key="${3:-}"
    local token="${4:-}"
    local token_env="${5:-${GITHUB_TOKEN_ENV}}"

    local kind="" target="" safe="" ssh_cmd=""

    if [[ -z "${auth}" || "${auth}" == "auto" ]]; then
        local env_auth=""
        env_auth="$(get_env "${GITHUB_AUTH_ENV}")"
        [[ -n "${env_auth}" ]] && auth="${env_auth}" || auth="auto"
    fi
    if [[ "${auth}" == "auto" ]]; then
        if is_ci && { [[ -n "${token}" ]] || [[ -n "$(get_env "${token_env}")" ]]; }; then
            auth="http"
        else
            auth="ssh"
        fi
    fi
    if [[ "${auth}" == "ssh" ]]; then

        kind="ssh"
        target="${remote}"
        safe="${remote}"

        if [[ -z "${key}" ]]; then
            key="$(get_env "${GITHUB_SSH_KEY_ENV}")"
            [[ -n "${key}" ]] || key="${SSH_KEY:-}"
        fi
        if [[ -z "${key}" && -f "${HOME}/.ssh/id_ed25519" ]]; then
            key="${HOME}/.ssh/id_ed25519"
        fi

        ssh_cmd='ssh -o StrictHostKeyChecking=accept-new -o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=2'

        if [[ -n "${key}" ]]; then

            key="${key/#\~/${HOME}}"
            [[ -f "${key}" ]] || die "Missing ssh key file: ${key}" 2

            printf -v ssh_cmd 'ssh -i %q -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new -o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=2' "${key}"

        fi

        printf '%s\t%s\t%s\t%s\n' "${kind}" "${target}" "${safe}" "${ssh_cmd}"
        return 0

    fi
    if [[ "${auth}" == "http" ]]; then

        kind="http"

        [[ -n "${token}" ]] || token="$(get_env "${token_env}")"
        [[ -n "${token}" ]] || die "Missing token. Use --token or --token-env <VAR> (default: ${GITHUB_TOKEN_ENV})." 2

        local cur="" host="" path="" url=""

        cur="$(git_remote_url "${remote}")"
        [[ -n "${cur}" ]] || die "Remote not found: ${remote}" 2

        read -r host path < <(git_parse_remote "${cur}") || die "Can't parse remote url: $(git_redact_url "${cur}")" 2
        url="$(git_build_https_token_url "${token}" "${host}" "${path}")" || die "Can't build token url" 2

        target="${url}"
        safe="https://***@${host}/${path}"

        printf '%s\t%s\t%s\t%s\n' "${kind}" "${target}" "${safe}" ""
        return 0

    fi

    die "Unknown auth: ${auth} (use auto|ssh|http)" 2

}
git_build_ssh_url () {

    local host="${1:-}"
    local path="${2:-}"

    [[ -n "${host}" && -n "${path}" ]] || return 1
    printf 'git@%s:%s\n' "${host}" "${path}"

}
git_build_https_url () {

    local host="${1:-}"
    local path="${2:-}"

    [[ -n "${host}" && -n "${path}" ]] || return 1
    printf 'https://%s/%s\n' "${host}" "${path}"

}
git_norm_path_git () {

    local p="${1:-}"
    [[ -n "${p}" ]] || { printf ''; return 0; }

    p="${p%/}"
    p="${p#/}"
    p="${p%.git}"

    printf '%s.git\n' "${p}"

}
git_init_supports_initial_branch () {

    ( git init -h 2>&1 || true ) | grep -q -- '--initial-branch'

}
git_set_default_branch () {

    local branch="${1:-main}"

    git branch -M "${branch}" >/dev/null 2>&1 && return 0
    git symbolic-ref HEAD "refs/heads/${branch}" >/dev/null 2>&1 && return 0

    return 0

}
git_guard_no_unborn_nested_repos () {

    ensure_pkg find 2>&1

    local root="${1:-.}"
    local d="" repo=""

    while IFS= read -r -d '' d; do

        repo="${d%/.git}"

        [[ "${repo}" == "${ROOT_DIR}" ]] && continue
        git -C "${repo}" rev-parse --verify HEAD >/dev/null 2>&1 && continue

        die "Nested git repo with no commit checked out: ${repo}. Remove its .git or initialize/commit it." 2

    done < <(find "${root}" -mindepth 2 -name .git -type d -print0 2>/dev/null)

}
git_root_version () {

    ensure_pkg awk 2>&1

    local v="" toml="${ROOT_DIR}/Cargo.toml"
    [[ -f "${toml}" ]] || die "Can't detect version: missing file: ${toml}" 2

    v="$(
        awk '
            BEGIN {
                sect=""
                ws=""
                pkg=""
            }

            /^\[workspace\.package\][[:space:]]*$/ { sect="ws"; next }
            /^\[package\][[:space:]]*$/           { sect="pkg"; next }
            /^\[[^]]+\][[:space:]]*$/             { sect=""; next }

            sect=="ws"  && ws==""  && match($0, /^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)"/, m) { ws=m[1]; next }
            sect=="pkg" && pkg=="" && match($0, /^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)"/, m) { pkg=m[1]; next }

            END {
                if (ws != "")  { print ws;  exit 0 }
                if (pkg != "") { print pkg; exit 0 }
                exit 1
            }
        ' "${toml}" 2>/dev/null
    )" || die "Can't detect version from ${toml}." 2

    [[ -n "${v}" ]] || die "Can't detect version from ${toml}." 2
    printf '%s\n' "${v}"

}
git_default_branch () {

    local remote="${1:-origin}"
    local auth="${2:-auto}"
    local key="${3:-}"
    local token="${4:-}"
    local token_env="${5:-${GITHUB_TOKEN_ENV}}"

    git_repo_guard
    git_require_remote "${remote}"

    local b="$(git symbolic-ref -q --short "refs/remotes/${remote}/HEAD" 2>/dev/null || true)"

    if [[ -n "${b}" ]]; then
        printf '%s\n' "${b#${remote}/}"
        return 0
    fi

    local kind="" target="" safe="" ssh_cmd="" line="" sym=""

    IFS=$'\t' read -r kind target safe ssh_cmd < <(
        git_auth_resolve "${auth}" "${remote}" "${key}" "${token}" "${token_env}"
    )

    while IFS= read -r line; do
        case "${line}" in
            "ref: refs/heads/"*" HEAD")
                sym="${line#ref: }"
                sym="${sym% HEAD}"
                break
            ;;
        esac
    done < <(run_git "${kind}" "${ssh_cmd}" ls-remote --symref "${target}" HEAD 2>/dev/null || true)

    if [[ -n "${sym}" ]]; then
        printf '%s\n' "${sym#refs/heads/}"
        return 0
    fi

    local def="$(git config --get init.defaultBranch 2>/dev/null || true)"

    if [[ -n "${def}" ]] && git show-ref --verify --quiet "refs/heads/${def}"; then
        printf '%s\n' "${def}"
        return 0
    fi

    for def in main master trunk production prod; do
        git show-ref --verify --quiet "refs/heads/${def}" && { printf '%s\n' "${def}"; return 0; }
    done

    def="$(git symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
    [[ -n "${def}" ]] && { printf '%s\n' "${def}"; return 0; }

    return 1

}
