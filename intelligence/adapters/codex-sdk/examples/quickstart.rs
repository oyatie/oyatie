use intelligence_codex_sdk::{Codex, ThreadOptions, TurnOptions};

fn main() -> intelligence_codex_sdk::Result<()> {
    let codex = Codex::default();
    let mut thread = codex.start_thread(ThreadOptions::default());
    let turn = thread.run(
        "Explain this repository in three bullets.",
        TurnOptions::default(),
    )?;
    println!("{}", turn.final_response);
    Ok(())
}
