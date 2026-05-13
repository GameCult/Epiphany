use crate::JsonSchema;
use crate::LoadableToolSpec;
use crate::ResponsesApiNamespace;
use crate::ResponsesApiNamespaceTool;
use crate::ToolName;
use crate::ToolSpec;
use crate::default_namespace_description;
use crate::mcp_tool_to_deferred_responses_api_tool;
use std::collections::BTreeMap;

pub const TOOL_SEARCH_TOOL_NAME: &str = "tool_search";
pub const TOOL_SEARCH_DEFAULT_LIMIT: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolSearchSourceInfo {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolSearchSource<'a> {
    pub server_name: &'a str,
    pub connector_name: Option<&'a str>,
    pub connector_description: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolSearchResultSource<'a> {
    pub server_name: &'a str,
    pub tool_namespace: &'a str,
    pub tool_name: &'a str,
    pub tool: &'a rmcp::model::Tool,
    pub connector_name: Option<&'a str>,
    pub connector_description: Option<&'a str>,
}

pub fn create_tool_search_tool(
    searchable_sources: &[ToolSearchSourceInfo],
    default_limit: usize,
) -> ToolSpec {
    let properties = BTreeMap::from([
        (
            "query".to_string(),
            JsonSchema::string(Some("Search query for deferred tools.".to_string())),
        ),
        (
            "limit".to_string(),
            JsonSchema::number(Some(format!(
                "Maximum number of tools to return (defaults to {default_limit})."
            ))),
        ),
    ]);

    let mut source_descriptions = BTreeMap::new();
    for source in searchable_sources {
        source_descriptions
            .entry(source.name.clone())
            .and_modify(|existing: &mut Option<String>| {
                if existing.is_none() {
                    *existing = source.description.clone();
                }
            })
            .or_insert(source.description.clone());
    }

    let source_descriptions = if source_descriptions.is_empty() {
        "None currently enabled.".to_string()
    } else {
        source_descriptions
            .into_iter()
            .map(|(name, description)| match description {
                Some(description) => format!("- {name}: {description}"),
                None => format!("- {name}"),
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let description = format!(
        "# Tool discovery\n\nSearches over deferred tool metadata with BM25 and exposes matching tools for the next model call.\n\nYou have access to tools from the following sources:\n{source_descriptions}\nSome of the tools may not have been provided to you upfront, and you should use this tool (`{TOOL_SEARCH_TOOL_NAME}`) to search for the required tools. For MCP tool discovery, always use `{TOOL_SEARCH_TOOL_NAME}` instead of `list_mcp_resources` or `list_mcp_resource_templates`."
    );

    ToolSpec::ToolSearch {
        execution: "client".to_string(),
        description,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["query".to_string()]),
            Some(false.into()),
        ),
    }
}

pub fn tool_search_result_source_to_loadable_tool_spec(
    source: ToolSearchResultSource<'_>,
) -> Result<LoadableToolSpec, serde_json::Error> {
    Ok(LoadableToolSpec::Namespace(ResponsesApiNamespace {
        name: source.tool_namespace.to_string(),
        description: tool_search_result_source_namespace_description(source),
        tools: vec![tool_search_result_source_to_namespace_tool(source)?],
    }))
}

fn tool_search_result_source_namespace_description(source: ToolSearchResultSource<'_>) -> String {
    source
        .connector_description
        .map(str::trim)
        .filter(|description| !description.is_empty())
        .map(str::to_string)
        .or_else(|| {
            source
                .connector_name
                .map(str::trim)
                .filter(|connector_name| !connector_name.is_empty())
                .map(|connector_name| format!("Tools for working with {connector_name}."))
        })
        .unwrap_or_else(|| default_namespace_description(source.tool_namespace))
}

fn tool_search_result_source_to_namespace_tool(
    source: ToolSearchResultSource<'_>,
) -> Result<ResponsesApiNamespaceTool, serde_json::Error> {
    let tool_name = ToolName::namespaced(source.tool_namespace, source.tool_name);
    mcp_tool_to_deferred_responses_api_tool(&tool_name, source.tool)
        .map(ResponsesApiNamespaceTool::Function)
}

pub fn collect_tool_search_source_infos<'a>(
    searchable_tools: impl IntoIterator<Item = ToolSearchSource<'a>>,
) -> Vec<ToolSearchSourceInfo> {
    searchable_tools
        .into_iter()
        .filter_map(|tool| {
            if let Some(name) = tool
                .connector_name
                .map(str::trim)
                .filter(|connector_name| !connector_name.is_empty())
            {
                return Some(ToolSearchSourceInfo {
                    name: name.to_string(),
                    description: tool
                        .connector_description
                        .map(str::trim)
                        .filter(|description| !description.is_empty())
                        .map(str::to_string),
                });
            }

            let name = tool.server_name.trim();
            if name.is_empty() {
                return None;
            }

            Some(ToolSearchSourceInfo {
                name: name.to_string(),
                description: None,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "tool_discovery_tests.rs"]
mod tests;
