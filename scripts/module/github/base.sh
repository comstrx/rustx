#!/usr/bin/env bash

gh_resolve_repo () {

    local repo="${1:-}"

    [[ -n "${repo}" ]] || repo="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)"
    [[ -n "${repo}" ]] || die "Cannot detect repo. Use --repo owner/repo" 2

    if [[ "${repo}" != */* ]]; then

        local owner="$(gh api user -q .login 2>/dev/null || true)"
        [[ -n "${owner}" ]] || die "Cannot detect owner. Login to gh or pass --repo owner/repo" 2

        repo="${owner}/${repo}"

    fi

    printf '%s\n' "${repo}"

}
gh_file_keys () {

    local file="${1:-}" line="" k=""

    while IFS= read -r line || [[ -n "${line}" ]]; do

        line="${line%$'\r'}"
        line="${line#"${line%%[![:space:]]*}"}"

        [[ -n "${line}" ]] || continue
        [[ "${line}" == \#* ]] && continue

        [[ "${line}" == export[[:space:]]* ]] && line="${line#export }"
        [[ "${line}" == *"="* ]] || continue

        k="${line%%=*}"
        k="${k%"${k##*[![:space:]]}"}"

        [[ "${k}" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || continue

        printf '%s\n' "${k}"

    done < "${file}"

}
gh_set_var () {

    source <(parse "$@" -- :action :type :repo :name value force:bool)

    if [[ "${action}" == "remove" ]]; then

        (( force )) || confirm "Delete ${type} '${name}' from ${repo}?" || return 0

        gh "${type}" delete "${name}" --repo "${repo}" "${kwargs[@]}"
        return 0

    fi

    gh "${type}" set "${name}" --repo "${repo}" --body "${value}" "${kwargs[@]}"

}
gh_cleanup_vars () {

    source <(parse "$@" -- :type :repo :file)

    local -A keep=()
    local remote_k="" k=""

    while IFS= read -r k || [[ -n "${k}" ]]; do
        keep["${k^^}"]=1
    done < <(gh_file_keys "${file}")

    while IFS= read -r remote_k || [[ -n "${remote_k}" ]]; do

        [[ -n "${remote_k}" && -z "${keep["${remote_k^^}"]+x}" ]] || continue
        gh_set_var remove "${type}" "${repo}" "${remote_k}" "${kwargs[@]}"

    done < <(gh "${type}" list --repo "${repo}" "${kwargs[@]}" --json name -q '.[].name' 2>/dev/null || true)

}
gh_sync_vars () {

    source <(parse "$@" -- :type :repo :file force:bool)
    gh "${type}" set -f "${file}" --repo "${repo}" "${kwargs[@]}"

    (( force )) && gh_cleanup_vars "${type}" "${repo}" "${file}" "${kwargs[@]}" --force
    return 0

}

gh_var_action () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- action type repo name value file force:bool)

    repo="$(gh_resolve_repo "${repo}")"

    if [[ "${action}" == "sync" ]]; then

        if [[ -z "${file}" && "${type}" == "secret" ]]; then

            file="${ROOT_DIR}/.secret"
            [[ -f "${file}" ]] || file="${ROOT_DIR}/.secret.example"

        elif [[ -z "${file}" ]]; then

            file="${ROOT_DIR}/.var"
            [[ -f "${file}" ]] || file="${ROOT_DIR}/.env"
            [[ -f "${file}" ]] || file="${ROOT_DIR}/.var.example"
            [[ -f "${file}" ]] || file="${ROOT_DIR}/.env.example"
            [[ -f "${file}" ]] || file="${ROOT_DIR}/.env-local"
            [[ -f "${file}" ]] || file="${ROOT_DIR}/.env-production"

        fi

        [[ -f "${file}" ]] || die "Missing ${type} file" 2

        gh_sync_vars "${type}" "${repo}" "${file}" "${force}" "${kwargs[@]}"

    else

        case "${action}" in add|remove) ;; *) die "Invalid --action (use add|remove)" 2 ;; esac
        case "${type}" in secret|variable) ;; *) die "Invalid --type (use variable|secret)" 2 ;; esac

        [[ "${name}" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid ${type} key: ${name}" 2

        gh_set_var "${action}" "${type}" "${repo}" "${name}" "${value}" "${force}" "${kwargs[@]}"

    fi

}
gh_clear_vars () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- type repo force)

    repo="$(gh_resolve_repo "${repo}")"
    (( force )) || confirm "Delete all ${type}s from ${repo}?" || return 0

    while IFS= read -r name || [[ -n "${name}" ]]; do

        [[ -n "${name}" ]] || continue
        gh_set_var remove "${type}" "${repo}" "${name}" "${kwargs[@]}" --force

    done < <(gh "${type}" list --repo "${repo}" "${kwargs[@]}" --json name -q '.[].name' 2>/dev/null || true)

}

gh_new_env () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- :name repo)

    repo="$(gh_resolve_repo "${repo}")"
    gh api -X PUT "repos/${repo}/environments/${name}" "${kwargs[@]}"

}
gh_remove_env () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- :name repo force:bool)

    repo="$(gh_resolve_repo "${repo}")"
    (( force )) || confirm "Delete environment '${name}' from ${repo}?" || return 0

    gh api -X DELETE "repos/${repo}/environments/${name}" "${kwargs[@]}"

}
gh_env_list () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- name repo count:bool ids:bool names:bool json:bool)

    repo="$(gh_resolve_repo "${repo}")"
    local mode="full"

    if (( json )); then mode="json"
    elif (( ids )); then mode="ids"
    elif (( names )); then mode="names"
    fi

    if (( count )); then
        
        if [[ -n "${name}" ]]; then gh api "repos/${repo}/environments/${name}" >/dev/null 2>&1 && printf '1\n' || printf '0\n'
        else gh api "repos/${repo}/environments" --jq '.total_count'
        fi

        return 0

    fi
    if [[ -n "${name}" ]]; then

        case "${mode}" in
            ids) gh api "repos/${repo}/environments/${name}" "${kwargs[@]}" --jq '.id' ;;
            names) gh api "repos/${repo}/environments/${name}" "${kwargs[@]}" --jq '.name' ;;
            *) gh api "repos/${repo}/environments/${name}" "${kwargs[@]}" ;;
        esac

        return 0

    fi

    case "${mode}" in
        ids) gh api "repos/${repo}/environments" "${kwargs[@]}" --jq '.environments[].id' ;;
        names) gh api "repos/${repo}/environments" "${kwargs[@]}" --jq '.environments[].name' ;;
        *) gh api "repos/${repo}/environments" "${kwargs[@]}" ;;
    esac

}
gh_var_list () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- type name repo names:bool values:bool json:bool info:bool)

    repo="$(gh_resolve_repo "${repo}")"
    local mode="full"

    if (( info )); then mode="info"
    elif (( json )); then mode="json"
    elif (( names && values )); then mode="full"
    elif (( names )); then mode="names"
    elif (( values )); then mode="values"
    fi

    if [[ -n "${name}" ]]; then

        if [[ "${type}" == "secret" ]]; then

            case "${mode}" in
                names) gh secret list --repo "${repo}" "${kwargs[@]}" --json name -q ".[] | select(.name == \"${name^^}\") | .name" ;;
                values) gh secret list --repo "${repo}" "${kwargs[@]}" --json name -q ".[] | select(.name == \"${name^^}\") | \"******\"" ;;
                json) gh secret list --repo "${repo}" "${kwargs[@]}" --json name ;;
                info) gh secret list --repo "${repo}" "${kwargs[@]}" ;;
                *) gh secret list --repo "${repo}" "${kwargs[@]}" --json name -q ".[] | select(.name == \"${name^^}\") | \"\(.name) = ******\"" ;;
            esac

        else

            case "${mode}" in
                names) gh variable get "${name^^}" --repo "${repo}" "${kwargs[@]}" --json name -q '.name' ;;
                values) gh variable get "${name^^}" --repo "${repo}" "${kwargs[@]}" --json value -q '.value' ;;
                json) gh variable get "${name^^}" --repo "${repo}" "${kwargs[@]}" --json name,value ;;
                info) gh variable get "${name^^}" --repo "${repo}" "${kwargs[@]}" ;;
                *) gh variable get "${name^^}" --repo "${repo}" "${kwargs[@]}" --json name,value -q '"\(.name) = \(.value)"' ;;
            esac

        fi

        return 0

    fi
    if [[ "${type}" == "secret" ]]; then

        case "${mode}" in
            names) gh secret list --repo "${repo}" "${kwargs[@]}" --json name -q '.[].name' ;;
            values) gh secret list --repo "${repo}" "${kwargs[@]}" --json name -q '.[].name | "******"' ;;
            json) gh secret list --repo "${repo}" "${kwargs[@]}" --json name ;;
            info) gh secret list --repo "${repo}" "${kwargs[@]}" ;;
            *) gh secret list --repo "${repo}" "${kwargs[@]}" --json name -q '.[] | "\(.name) = ******"' ;;
        esac

        return 0

    fi

    case "${mode}" in
        names) gh variable list --repo "${repo}" "${kwargs[@]}" --json name -q '.[].name' ;;
        values) gh variable list --repo "${repo}" "${kwargs[@]}" --json value -q '.[].value' ;;
        json) gh variable list --repo "${repo}" "${kwargs[@]}" --json name,value ;;
        info) gh variable list --repo "${repo}" "${kwargs[@]}" ;;
        *) gh variable list --repo "${repo}" "${kwargs[@]}" --json name,value -q '.[] | "\(.name) = \(.value)"' ;;
    esac

}

gh_new_repo () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- :name private:bool)

    local full="${name}"

    if [[ "${full}" != */* ]]; then

        local owner="$(gh api user -q .login 2>/dev/null || true)"
        [[ -n "${owner}" ]] || die "repo: use owner/repo (cannot detect owner)" 2

        full="${owner}/${full}"

    fi

    (( private )) &&  kwargs+=( --private ) || kwargs+=( --public )

    gh repo view "${full}" >/dev/null 2>&1 || gh repo create "${full}" "${kwargs[@]}"
    git remote get-url origin >/dev/null 2>&1 || git remote add origin "git@github.com:${full}.git"

}
gh_remove_repo () {

    ensure_pkg gh 2>&1
    source <(parse "$@" -- :name force:bool)

    local full="${name}"
    (( YES_ENV || force )) && kwargs+=( --yes )

    if [[ "${full}" != */* ]]; then

        local owner="$(gh api user -q .login 2>/dev/null || true)"
        [[ -n "${owner}" ]] || die "repo: use owner/repo (cannot detect owner)" 2

        full="${owner}/${full}"

    fi

    (( force )) || confirm "Delete repository: '${full}'?" || return 0
    gh repo delete "${full}" "${kwargs[@]}"

}
