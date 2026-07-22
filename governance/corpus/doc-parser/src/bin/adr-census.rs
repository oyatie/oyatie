//! Thin immutable-git adapter for the authority-neutral ADR census.

use corpus_doc_parser::census::census_from_git;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(commit) = args.next().filter(|_| args.next().is_none()) else {
        eprintln!("usage: adr-census <40-hex-commit>");
        std::process::exit(2);
    };
    match census_from_git(&commit) {
        Ok(receipt) => print!("{}", String::from_utf8_lossy(receipt.canonical_bytes())),
        Err(error) => {
            eprintln!("adr-census: {error}");
            std::process::exit(1);
        }
    }
}
