#!/usr/bin/env bash

cmd_lint_help () {

    info_ln "Lint :\n"

    printf '    %s\n' \
        "ws-fix                     * Remove trailing whitespace in git-tracked files" \
        "fmt-check                  * Verify formatting --nightly (no changes)" \
        "fmt-fix                    * Auto-format code --nightly" \
        "fmt-stable-check           * Verify formatting checks (no changes)" \
        "fmt-stable-fix             * Auto-format code" \
        "" \
        "taplo-check                * Validate TOML formatting (no changes)" \
        "taplo-fix                  * Auto-format TOML files" \
        "" \
        "prettier-check             * Validate formatting for Markdown/YAML/etc. (no changes)" \
        "prettier-fix               * Auto-format Markdown/YAML/etc." \
        "" \
        "typos-check                * Detect source-code typos (no changes)" \
        "typos-fix                  * Apply unambiguous typo corrections" \
        "" \
        ''

}

cmd_ws_fix () {

    ensure git perl

    git rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "ws-fix: not a git repository" 2

    git ls-files -z | perl -e '
        use strict;
        use warnings;
        use File::Basename qw(dirname);
        use File::Temp qw(tempfile);

        binmode(STDIN);
        local $/ = "\0";
        my $exit_code = 0;

        while (defined(my $path = <STDIN>)) {
            chomp($path);
            next if $path eq "";
            next if -l $path || !-f $path;

            open my $input, "<:raw", $path or do { $exit_code = 1; next; };
            local $/;
            my $data = <$input>;
            close $input;

            next if !defined $data || index($data, "\0") != -1;
            next if !($data =~ s/[ \t]+(?=\r?$)//mg);

            my ($output, $temp) = tempfile(".wsfix.XXXXXX", DIR => dirname($path), UNLINK => 0);
            binmode($output);

            print $output $data or do { close $output; unlink($temp); $exit_code = 1; next; };
            close $output or do { unlink($temp); $exit_code = 1; next; };

            my @stat = stat($path);
            if (@stat) {
                chmod($stat[2] & 07777, $temp);
                eval { chown($stat[4], $stat[5], $temp); 1; };
            }

            if (rename($temp, $path)) {
                next;
            }

            my $backup = $path . ".wsfix.bak.$$";
            if (!rename($path, $backup)) {
                unlink($temp);
                $exit_code = 1;
                next;
            }
            if (!rename($temp, $path)) {
                rename($backup, $path);
                unlink($temp);
                $exit_code = 1;
                next;
            }

            unlink($backup);
        }

        exit($exit_code);
    '

}
cmd_fmt_check () {

    run_cargo fmt --nightly --all -- --check "$@"

}
cmd_fmt_fix () {

    run_cargo fmt --nightly --all "$@"

}
cmd_fmt_stable_check () {

    run_cargo fmt --all -- --check "$@"

}
cmd_fmt_stable_fix () {

    run_cargo fmt --all "$@"

}

cmd_taplo_check () {

    ensure taplo
    run taplo fmt --check "$@"

}
cmd_taplo_fix () {

    ensure taplo
    run taplo fmt "$@"

}

cmd_prettier_check () {

    ensure node

    local -a args=( --check "**/*.{md,mdx,yml,yaml,json,jsonc}" )
    local config=""
    config="$(config_file prettierrc yaml yml)"

    [[ -f "${config}" ]] && args+=( --config "${config}" )
    run npx -y prettier@3.8.1 --no-error-on-unmatched-pattern --ignore-path .gitignore "${args[@]}" "$@"

}
cmd_prettier_fix () {

    ensure node

    local -a args=( --write "**/*.{md,mdx,yml,yaml,json,jsonc}" )
    local config=""
    config="$(config_file prettierrc yaml yml)"

    [[ -f "${config}" ]] && args+=( --config "${config}" )
    run npx -y prettier@3.8.1 --no-error-on-unmatched-pattern --ignore-path .gitignore "${args[@]}" "$@"

}

cmd_typos_check () {

    ensure typos

    local -a args=( --format brief )
    local config=""
    config="$(config_file typos toml)"

    [[ -f "${config}" ]] && args+=( --config "${config}" )
    run typos "${args[@]}" "$@"

}
cmd_typos_fix () {

    ensure typos

    local -a args=( --write-changes )
    local config=""
    config="$(config_file typos toml)"

    [[ -f "${config}" ]] && args+=( --config "${config}" )
    run typos "${args[@]}" "$@"

}

