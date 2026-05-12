use clap::CommandFactory;
use cortex_cli::cli::Cli;

#[test]
fn root_help_documents_utility_help_syntax_split() {
    let help = Cli::command().render_long_help().to_string();

    assert!(
        help.contains("Commands without subcommands only support --help"),
        "root help should explain why `cortex github help` works while single-purpose utilities use `--help`:\n{help}"
    );
}
