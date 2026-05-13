use super::*;
use crate::JsonSchema;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn create_tool_search_tool_deduplicates_and_renders_enabled_sources() {
    assert_eq!(
        create_tool_search_tool(
            &[
                ToolSearchSourceInfo {
                    name: "Google Drive".to_string(),
                    description: Some(
                        "Use Google Drive as the single entrypoint for Drive, Docs, Sheets, and Slides work."
                            .to_string(),
                    ),
                },
                ToolSearchSourceInfo {
                    name: "Google Drive".to_string(),
                    description: None,
                },
                ToolSearchSourceInfo {
                    name: "docs".to_string(),
                    description: None,
                },
            ],
            /*default_limit*/ 8,
        ),
        ToolSpec::ToolSearch {
            execution: "client".to_string(),
            description: "# Tool discovery\n\nSearches over deferred tool metadata with BM25 and exposes matching tools for the next model call.\n\nYou have access to tools from the following sources:\n- Google Drive: Use Google Drive as the single entrypoint for Drive, Docs, Sheets, and Slides work.\n- docs\nSome of the tools may not have been provided to you upfront, and you should use this tool (`tool_search`) to search for the required tools. For MCP tool discovery, always use `tool_search` instead of `list_mcp_resources` or `list_mcp_resource_templates`.".to_string(),
            parameters: JsonSchema::object(BTreeMap::from([
                    (
                        "limit".to_string(),
                        JsonSchema::number(Some(
                                "Maximum number of tools to return (defaults to 8)."
                                    .to_string(),
                            ),),
                    ),
                    (
                        "query".to_string(),
                        JsonSchema::string(Some("Search query for deferred tools.".to_string()),),
                    ),
                ]), Some(vec!["query".to_string()]), Some(false.into())),
        }
    );
}
