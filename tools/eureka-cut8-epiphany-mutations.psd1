# Eureka Cut 8 mutation suite, the Epiphany half: the epoch constant and the
# live registrar. Entries only; the harness is tools/eureka-mutations.ps1.
#
#   powershell -File tools/eureka-mutations.ps1 `
#       -Entries tools/eureka-cut8-epiphany-mutations.psd1 `
#       -Target epiphany-pipeline/src/lib.rs `
#       -Test 'cargo test -p epiphany-pipeline --lib'
#
# E1-E3 are killed by `every_kind_is_at_the_epochs_version`: it reads the
# version segment off the constant and holds every type id to it, so a wrong
# epoch dies whether it is `.v2` or `.v1x`; and it counts what the live
# registrar registers against `PipelineKind::ALL`, so one type fewer and one
# foreign type more both die on the count.
#
# D1 and D2 are the derives the wire carries a batch through, and both are
# killed by `every_document_and_refusal_serialises_and_reads_back`: the tag a
# reader dispatches on is asserted against `PipelineKind::name()` for every
# kind, and every refusal variant is round-tripped whole, so a field that
# leaves the wire comes back empty and is not the refusal that went out.
#
# R1 and R2 are the public reference door, and both are killed by
# `a_ref_validates_as_an_id_of_the_kind_it_declares`: a kind that is not the
# id's kills the door that answers Ok to everything, and a trailing dot kills
# the door that reads the kind segment and nothing else.
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
        },
        @{
            Id   = 'D1'
            Rule = 'The document tag is the kind name: a wire reader dispatches on the same string the key segment carries, so the variant spelling is renamed to it and not left as Rust case.'
            Test = 'tests::every_document_and_refusal_serialises_and_reads_back'
            Old  = '        #[serde(tag = "kind", content = "value", rename_all = "snake_case")]'
            New  = '        #[serde(tag = "kind", content = "value")]'
        },
        @{
            Id   = 'D2'
            Rule = 'A refusal carries its named parts over the wire: the field it refused is on the wire, not dropped and defaulted back on the reader.'
            Test = 'tests::every_document_and_refusal_serialises_and_reads_back'
            Old  = '    FieldBound { field: String, limit: u32, actual: u32 },'
            New  = '    FieldBound { #[serde(skip)] field: String, limit: u32, actual: u32 },'
        },
        @{
            Id   = 'R1'
            Rule = 'The public door is the grammar, not a formality: a door that answers Ok to every reference leaves the read side unable to tell a malformed reference from a missing document.'
            Test = 'tests::a_ref_validates_as_an_id_of_the_kind_it_declares'
            Old  = @'
    pub fn validate_ref(&self) -> Result<(), PipelineRefusal> {
        self.validate("ref")
    }
'@
            New  = @'
    pub fn validate_ref(&self) -> Result<(), PipelineRefusal> {
        Ok(())
    }
'@
        },
        @{
            Id   = 'R2'
            Rule = 'The door delegates to the one grammar: a door that checks the kind segment alone admits a local no writer composes, and the reference grammar has two owners.'
            Test = 'tests::a_ref_validates_as_an_id_of_the_kind_it_declares'
            Old  = @'
    pub fn validate_ref(&self) -> Result<(), PipelineRefusal> {
        self.validate("ref")
    }
'@
            New  = @'
    pub fn validate_ref(&self) -> Result<(), PipelineRefusal> {
        if self.id.0.split(':').nth(1) != Some(self.kind.name()) {
            return Err(format_error("ref.id", &self.id.0));
        }
        Ok(())
    }
'@
        }
    )
}
