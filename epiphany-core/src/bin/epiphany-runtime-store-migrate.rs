use anyhow::{Result, anyhow};
use epiphany_core::migrate_runtime_store_to_redb;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let source = args.next().ok_or_else(|| anyhow!("usage: epiphany-runtime-store-migrate <source.cc> <destination.redb> <migrated-at-rfc3339>"))?;
    let destination = args
        .next()
        .ok_or_else(|| anyhow!("missing destination.redb"))?;
    let migrated_at = args
        .next()
        .ok_or_else(|| anyhow!("missing migrated-at-rfc3339"))?;
    if args.next().is_some() {
        return Err(anyhow!("unexpected extra migration arguments"));
    }
    let receipt = migrate_runtime_store_to_redb(source, destination, &migrated_at)?;
    println!(
        "{}",
        rmp_serde::to_vec_named(&receipt)?
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    Ok(())
}
