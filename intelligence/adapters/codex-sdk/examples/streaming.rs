use intelligence_codex_sdk::{Codex, ThreadEvent, ThreadOptions, TurnOptions};

fn main() -> intelligence_codex_sdk::Result<()> {
    let codex = Codex::default();
    let mut thread = codex.start_thread(ThreadOptions::default());
    let streamed = thread.run_streamed("Diagnose the test failure", TurnOptions::default())?;

    for event in streamed.events {
        match event? {
            ThreadEvent::ItemCompleted(event) => println!("item: {:?}", event.item),
            ThreadEvent::TurnCompleted(event) => println!("usage: {:?}", event.usage),
            _ => {}
        }
    }

    Ok(())
}
