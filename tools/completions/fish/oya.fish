# tools/completions/fish/oya.fish — fish shell completion for the oya CLI wrapper
#
# Installation (manual):
#   cp tools/completions/fish/oya.fish ~/.config/fish/completions/oya.fish
# Installation (system):
#   cp tools/completions/fish/oya.fish /usr/share/fish/vendor_completions.d/oya.fish

# Disable file completions for the main command
complete -c oya -f

# Top-level subcommands
complete -c oya -n '__fish_use_subcommand' -a git                  -d 'Drop-in git surface with local ledger side channel'
complete -c oya -n '__fish_use_subcommand' -a vcs                  -d 'Coordination ratchet compatibility surface'
complete -c oya -n '__fish_use_subcommand' -a gate                 -d 'Run foundry quality gates'
complete -c oya -n '__fish_use_subcommand' -a governance-gates     -d 'Governance-layer gate checks'
complete -c oya -n '__fish_use_subcommand' -a foundation-audit-gates -d 'Foundation audit gate set'
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
complete -c oya -n '__fish_seen_subcommand_from vcs' -a claim    -d 'Claim a policy-ratchet scope'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a work     -d 'Start policy-ratchet work'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a verify   -d 'Verify policy-ratchet evidence'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a done     -d 'Emit a policy-ratchet ChangeBundle'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a status   -d 'Read policy-ratchet status'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a symbols  -d 'List policy-ratchet symbols'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a queue    -d 'Read policy-ratchet queue projection'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a watch    -d 'Watch policy-ratchet events'
complete -c oya -n '__fish_seen_subcommand_from vcs' -a promote  -d 'Promote a policy-ratchet bundle'

# git subcommands
complete -c oya -n '__fish_seen_subcommand_from git' -a status    -d 'Show working tree status'
complete -c oya -n '__fish_seen_subcommand_from git' -a add       -d 'Add file contents to the index'
complete -c oya -n '__fish_seen_subcommand_from git' -a commit    -d 'Record changes'
complete -c oya -n '__fish_seen_subcommand_from git' -a diff      -d 'Show changes'
complete -c oya -n '__fish_seen_subcommand_from git' -a log       -d 'Show commit log'
complete -c oya -n '__fish_seen_subcommand_from git' -a show      -d 'Show objects'
complete -c oya -n '__fish_seen_subcommand_from git' -a branch    -d 'List or create branches'
complete -c oya -n '__fish_seen_subcommand_from git' -a checkout  -d 'Switch branches or restore paths'
complete -c oya -n '__fish_seen_subcommand_from git' -a switch    -d 'Switch branches'
complete -c oya -n '__fish_seen_subcommand_from git' -a restore   -d 'Restore working tree files'
complete -c oya -n '__fish_seen_subcommand_from git' -a stash     -d 'Stash changes'
complete -c oya -n '__fish_seen_subcommand_from git' -a fetch     -d 'Download objects and refs'
complete -c oya -n '__fish_seen_subcommand_from git' -a pull      -d 'Fetch and integrate'
complete -c oya -n '__fish_seen_subcommand_from git' -a push      -d 'Update remote refs'
complete -c oya -n '__fish_seen_subcommand_from git' -a merge     -d 'Join histories'
complete -c oya -n '__fish_seen_subcommand_from git' -a rebase    -d 'Reapply commits'
complete -c oya -n '__fish_seen_subcommand_from git' -a tag       -d 'Create or list tags'
complete -c oya -n '__fish_seen_subcommand_from git' -a remote    -d 'Manage remotes'
complete -c oya -n '__fish_seen_subcommand_from git' -a rev-parse -d 'Pick out and massage parameters'

# doc subcommands
complete -c oya -n '__fish_seen_subcommand_from doc' -a adr-index       -d 'Generate ADR index'
complete -c oya -n '__fish_seen_subcommand_from doc' -a mdbook          -d 'Build mdBook documentation'
complete -c oya -n '__fish_seen_subcommand_from doc' -a openapi         -d 'Generate OpenAPI spec'
complete -c oya -n '__fish_seen_subcommand_from doc' -a rustdoc         -d 'Generate rustdoc'
complete -c oya -n '__fish_seen_subcommand_from doc' -a milestone-audit -d 'Audit milestone completion'
complete -c oya -n '__fish_seen_subcommand_from doc' -a product-index   -d 'Generate product index'
