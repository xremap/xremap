use xremap::{xremap_cli, NoopPlugin};

fn main() -> anyhow::Result<()> {
    xremap_cli(NoopPlugin {})
}
