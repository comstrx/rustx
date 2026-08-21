#!/usr/bin/env bash

github_help () {

    info_ln "GitHub :\n"

    printf '    %s\n' \
        "new-repo                   * Create a new repository" \
        "remove-repo                * Remove a repository" \
        "" \
        "new-env                    * Create an environment" \
        "remove-env                 * Delete an environment" \
        "" \
        "add-var                    * Add/Update a variable" \
        "remove-var                 * Remove a variable" \
        "add-secret                 * Add/Update a secret" \
        "remove-secret              * Remove a secret" \
        "" \
        "sync-vars                  * Sync variables from file (.env/.var... by default)" \
        "sync-secrets               * Sync secrets from file (.secret by default)" \
        "clear-vars                 * Delete all variables keys" \
        "clear-secrets              * Delete all secrets keys" \
        "" \
        "env-list                   * List environments" \
        "var-list                   * List variables" \
        "secret-list                * List secrets" \
        ''

}
cmd_github_help () {

    info_ln "GitHub :\n"

    printf '    %s\n' \
        "new-repo                   * Create a new repository" \
        "    --name                   Base/Full name of repo (or owner/repo)" \
        "    --private                Set repo as private" \
        "" \
        "remove-repo                * Remove a repository" \
        "    --name                   Base/Full name of repo (or owner/repo)" \
        "    --force                  Skip confirm (implies --yes)" \
        "" \
        "new-env                    * Create an environment" \
        "    --name                   Environment name (ex: production)" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "" \
        "remove-env                 * Remove an environment" \
        "    --name                   Environment name" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --force                  Skip confirm (implies --yes)" \
        "" \
        "add-var                    * Add/Update a variable" \
        "    --name                   Variable key name" \
        "    --value                  Variable value (optional: will prompt if omitted)" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped variable)" \
        "" \
        "remove-var                 * Remove a variable" \
        "    --name                   Variable key name" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped variable)" \
        "    --force                  Skip confirm (implies --yes)" \
        "" \
        "add-secret                 * Add/Update a secret" \
        "    --name                   Secret key name" \
        "    --value                  Secret value (optional: will prompt if omitted)" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped secret)" \
        "    --app                    Secret app (actions|codespaces|dependabot). Default: actions" \
        "" \
        "remove-secret              * Remove a secret" \
        "    --name                   Secret key name" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped secret)" \
        "    --app                    Secret app (actions|codespaces|dependabot). Default: actions" \
        "    --force                  Skip confirm (implies --yes)" \
        "" \
        "sync-vars                  * Sync variables from file (.env/.var... by default)" \
        "    --file                   Vars file (default: .var, .variables, .env, .env-local, .env-production)" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped variables)" \
        "    --force                  Delete remote keys not present in file" \
        "" \
        "sync-secrets               * Sync secrets from file (.secret by default)" \
        "    --file                   Secrets file (default: .secret)" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped secrets)" \
        "    --app                    Secret app (actions|codespaces|dependabot). Default: actions" \
        "    --force                  Delete remote keys not present in file" \
        "" \
        "clear-vars                 * Delete all variables keys" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: clear env-scoped variables)" \
        "" \
        "clear-secrets              * Delete all secrets keys" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: clear env-scoped secrets)" \
        "    --app                    Secret app (actions|codespaces|dependabot). Default: actions" \
        "" \
        "env-list                   * List environments" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --name                   Filter: print only this environment if exists" \
        "" \
        "var-list                   * List variables" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped variable)" \
        "" \
        "secret-list                * List secrets" \
        "    --repo                   Target repo (owner/repo). Default: current" \
        "    --env                    Environment name (optional: env-scoped secret)" \
        "    --app                    Secret app (actions|codespaces|dependabot). Default: actions" \
        ''

}

cmd_new_repo () {

    gh_new_repo "$@"

}
cmd_remove_repo () {

    gh_remove_repo "$@"

}

cmd_new_env () {

    gh_new_env "$@"

}
cmd_remove_env () {

    gh_remove_env "$@"

}
cmd_env_list () {

    gh_env_list "$@"

}
cmd_var_list () {

    gh_var_list variable "$@"

}
cmd_secret_list () {

    gh_var_list secret "$@"

}

cmd_add_var () {

    source <(parse "$@" -- :name value repo)
    gh_var_action add variable "${repo}" "${name}" "${value}" "${kwargs[@]}"

}
cmd_remove_var () {

    source <(parse "$@" -- :name value repo)
    gh_var_action remove variable "${repo}" "${name}" "${value}" "${kwargs[@]}"

}
cmd_add_secret () {

    source <(parse "$@" -- :name value repo)
    gh_var_action add secret "${repo}" "${name}" "${value}" "${kwargs[@]}"

}
cmd_remove_secret () {

    source <(parse "$@" -- :name value repo)
    gh_var_action remove secret "${repo}" "${name}" "${value}" "${kwargs[@]}"

}
cmd_sync_vars () {

    source <(parse "$@" -- file repo)
    gh_var_action sync variable "${repo}" --file "${file}" "${kwargs[@]}"

}
cmd_sync_secrets () {

    source <(parse "$@" -- file repo)
    gh_var_action sync secret "${repo}" --file "${file}" "${kwargs[@]}"

}

cmd_clear_vars () {

    gh_clear_vars variable "$@"

}
cmd_clear_secrets () {

    gh_clear_vars secret "$@"

}
