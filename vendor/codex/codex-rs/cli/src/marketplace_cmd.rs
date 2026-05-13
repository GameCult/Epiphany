use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use codex_utils_cli::CliConfigOverrides;

#[derive(Debug, Parser)]
pub struct MarketplaceCli {
    #[clap(flatten)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    subcommand: MarketplaceSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum MarketplaceSubcommand {
    Add(DisabledMarketplaceArgs),
    Upgrade(DisabledMarketplaceArgs),
    Remove(DisabledMarketplaceArgs),
}

#[derive(Debug, Parser)]
struct DisabledMarketplaceArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    _args: Vec<String>,
}

impl MarketplaceCli {
    pub async fn run(self) -> Result<()> {
        let _ = self;
        bail!(
            "Codex plugin marketplace commands are disabled in Epiphany; the Codex organ is limited to OpenAI auth and model routing."
        )
    }
}
