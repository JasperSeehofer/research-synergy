use clap::Args;

#[derive(Args, Debug)]
pub struct ServeArgs {}

pub async fn run(_args: ServeArgs) -> anyhow::Result<()> {
    println!("Web server not yet implemented (coming in Phase 8)");
    Ok(())
}
