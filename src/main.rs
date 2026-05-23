use std::io::IsTerminal;

fn main() -> anyhow::Result<()> {
    if std::io::stdout().is_terminal() {
        bashcards::ui::run_tui()
    } else {
        println!("bashcards TUI is ready. Run in an interactive terminal to play.");
        Ok(())
    }
}
