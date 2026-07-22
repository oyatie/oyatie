//! Thin immutable-git adapter for the authority-neutral ADR census.

use corpus_doc_parser::census::census_from_git;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(corpus_commit), Some(parser_commit)) = (args.next(), args.next()) else {
        eprintln!("usage: adr-census <40-hex-corpus-commit> <40-hex-parser-commit>");
        std::process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("usage: adr-census <40-hex-corpus-commit> <40-hex-parser-commit>");
        std::process::exit(2);
    }
    match census_from_git(&corpus_commit, &parser_commit) {
        Ok(receipt) => print!("{}", String::from_utf8_lossy(receipt.canonical_bytes())),
        Err(error) => {
            eprintln!("adr-census: {error}");
            std::process::exit(1);
        }
    }
}
