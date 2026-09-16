# Eureka Cut 8 mutation suite, the Epiphany half: the epoch constant and the
# live registrar. Entries only; the harness is tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut8-epiphany-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# All three are killed by `every_kind_is_at_the_epochs_version`: it reads the
# version segment off the constant and holds every type id to it, so a wrong
# epoch dies whether it is `.v2` or `.v1x`; and it counts what the live
# registrar registers against `PipelineKind::ALL`, so one type fewer and one
# foreign type more both die on the count.
@{
    Mutations = @(
        @{
            Id   = 'E1'
            Rule = 'The epoch names the version every kind is published at; a bumped epoch fails until the kinds move with it.'
            Test = 'tests::every_kind_is_at_the_epochs_version'
            Old  = 'pub const PIPELINE_SCHEMA_EPOCH: &str = "epiphany.pipeline.epoch.v1";'
            New  = 'pub const PIPELINE_SCHEMA_EPOCH: &str = "epiphany.pipeline.epoch.v2";'
        },
        @{
            Id   = 'E2'
            Rule = 'The live registrar registers every kind: one fewer is refused on the count.'
            Test = 'tests::every_kind_is_at_the_epochs_version'
            Old  = '            $(cache.register_entry_type::<$document>()?;)*'
            New  = '            let mut first = true; $(if !first { cache.register_entry_type::<$document>()?; } first = false;)*'
        },
        @{
            Id   = 'E3'
            Rule = 'The live registrar registers nothing but the kinds: the foreign stand-in registered again under cfg(test) is refused on the count.'
            Test = 'tests::every_kind_is_at_the_epochs_version'
            Old  = @'
            $(cache.register_entry_type::<$document>()?;)*
            Ok(())
'@
            New  = @'
            $(cache.register_entry_type::<$document>()?;)*
            #[cfg(test)]
            cache.register_entry_type::<ForeignDocument>()?;
            Ok(())
'@
        }
    )
}
