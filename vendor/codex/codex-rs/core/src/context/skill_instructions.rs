use super::ContextualUserFragment;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SkillInstructions {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) contents: String,
}

impl ContextualUserFragment for SkillInstructions {
    const ROLE: &'static str = "user";
    const START_MARKER: &'static str = "<skill>";
    const END_MARKER: &'static str = "</skill>";

    fn body(&self) -> String {
        format!(
            "\n<name>{}</name>\n<path>{}</path>\n{}\n",
            self.name, self.path, self.contents
        )
    }
}
