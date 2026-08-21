fn main() {
    let code = tooling_agent_read::cli_main(std::env::args().skip(1));
    std::process::exit(code);
}
