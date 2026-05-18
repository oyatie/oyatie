# tools/completions/fish/oya.fish — fish shell completion for the oya CLI wrapper
#
# Installation (manual):
#   cp tools/completions/fish/oya.fish ~/.config/fish/completions/oya.fish
# Installation (system):
#   cp tools/completions/fish/oya.fish /usr/share/fish/vendor_completions.d/oya.fish

# Disable file completions for the main command
complete -c oya -f

# Top-level subcommands
complete -c oya -n '__fish_use_subcommand' -a vcs                  -d 'Version control operations (canonical; replaces grit/rtk)'
complete -c oya -n '__fish_use_subcommand' -a gate                 -d 'Run foundry quality gates'
complete -c oya -n '__fish_use_subcommand' -a governance-gates     -d 'Governance-layer gate checks'
complete -c oya -n '__fish_use_subcommand' -a foundation-audit-gates -d 'Foundation audit gate suite'
complete -c oya -n '__fish_use_subcommand' -a catalog              -d 'Crate and product catalog operations'
complete -c oya -n '__fish_use_subcommand' -a check                -d 'Run individual check validators'
complete -c oya -n '__fish_use_subcommand' -a demo                 -d 'Demo scaffolding helpers'
complete -c oya -n '__fish_use_subcommand' -a doc                  -d 'Documentation generation'
complete -c oya -n '__fish_use_subcommand' -a lint                 -d 'Lint checks across workspace'
complete -c oya -n '__fish_use_subcommand' -a onprem               -d 'On-premises deployment helpers'
complete -c oya -n '__fish_use_subcommand' -a ops                  -d 'Operations commands'
complete -c oya -n '__fish_use_subcommand' -a submit               -d 'Submit artifacts to the Foundry pipeline'
complete -c oya -n '__fish_use_subcommand' -a supply-chain         -d 'Supply chain verification'
complete -c oya -n '__fish_use_subcommand' -a verify               -d 'Verification suite'
complete -c oya -n '__fish_use_subcommand' -a help                 -d 'Print help for a subcommand'

# vcs subcommands
complete -c oya -n '__fish_seen_subcommand_from vcs' -a status   -d 'Show working tree status'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a log      -d 'Show commit log'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a diff     -d 'Show changes'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a commit   -d 'Record changes'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a push     -d 'Push branch to remote'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a pull     -d 'Pull from remote'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a branch   -d 'List or create branches'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a checkout -d 'Switch branches'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a merge    -d 'Merge branches'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a rebase   -d 'Rebase onto branch'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a pr       -d 'Pull request operations'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a review   -d 'Trigger review pipeline'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a admit    -d 'Admission gate check'

# doc subcommands
complete -c oya -n '__fish_seen_subcommand_from doc' -a adr-index       -d 'Generate ADR index'
complete -c oya -n '__fish_seen_subcommand_from doc' -a mdbook          -d 'Build mdBook documentation'
complete -c oya -n '__fish_seen_subcommand_from doc' -a openapi         -d 'Generate OpenAPI spec'
complete -c oya -n '__fish_seen_subcommand_from doc' -a rustdoc         -d 'Generate rustdoc'
complete -c oya -n '__fish_seen_subcommand_from doc' -a milestone-audit -d 'Audit milestone completion'
complete -c oya -n '__fish_seen_subcommand_from doc' -a product-index   -d 'Generate product index'
