//! Drain images captured by tools before hook serialization and split them by
//! harness capability for the session vision pipeline.

use super::*;
use xai_grok_tools::util::base64_images::ExtractedImage;

pub(super) fn drain_tool_layer_extracted_images(
    output: &mut ToolsToolOutput,
) -> Vec<ExtractedImage> {
    match output {
        ToolsToolOutput::ReadFile(ReadFileOutput::FileContent(file)) => {
            std::mem::take(&mut file.extracted_images)
        }
        ToolsToolOutput::MCP(mcp) => std::mem::take(&mut mcp.extracted_images),
        _ => Vec::new(),
    }
}

/// Successful tool output with large image captures removed. Constructing
/// this before PostToolUse serialization prevents base64 payloads from leaking
/// into hook data while retaining them for multimodal follow-ups.
pub(super) struct DrainedToolSuccess {
    result: ToolRunResult,
    tool_layer_images: Vec<ExtractedImage>,
}

impl DrainedToolSuccess {
    #[must_use]
    pub(super) fn new(mut result: ToolRunResult) -> Self {
        let tool_layer_images = drain_tool_layer_extracted_images(&mut result.output);
        Self {
            result,
            tool_layer_images,
        }
    }

    pub(super) fn output(&self) -> &ToolsToolOutput {
        &self.result.output
    }

    pub(super) fn into_parts(self) -> (ToolRunResult, Vec<ExtractedImage>) {
        (self.result, self.tool_layer_images)
    }
}

pub(super) fn split_tool_layer_for_harness(
    text_only_harness: bool,
    vision: &mut Vec<ExtractedImage>,
    tool_layer: Vec<ExtractedImage>,
) {
    if !text_only_harness {
        vision.extend(tool_layer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_tools::types::output::{MCPOutput, ToolOutput};
    use xai_grok_tools::util::base64_images::{ExtractedImage, IMAGE_CONTENT_PLACEHOLDER};

    fn image(data: &str, mime_type: &str) -> ExtractedImage {
        ExtractedImage {
            data: data.to_owned(),
            mime_type: mime_type.to_owned(),
        }
    }

    fn run_result(output: ToolOutput) -> ToolRunResult {
        ToolRunResult {
            output,
            prompt_text: "prompt".into(),
            effective_tool_name: None,
            external_content: None,
        }
    }

    #[test]
    fn drained_success_removes_mcp_images_before_serialization() {
        let payload = "A".repeat(8_000);
        let mut mcp = MCPOutput::okay_output(
            "browser_screenshot".into(),
            "browser-use".into(),
            IMAGE_CONTENT_PLACEHOLDER.into(),
        );
        mcp.extracted_images = vec![image(&payload, "image/png")];

        let drained = DrainedToolSuccess::new(run_result(ToolOutput::MCP(mcp)));
        let serialized = serde_json::to_string(drained.output()).unwrap();
        assert!(!serialized.contains(&payload));
        let (result, images) = drained.into_parts();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].data, payload);
        let ToolOutput::MCP(mcp) = result.output else {
            panic!("expected MCP output");
        };
        assert!(mcp.extracted_images.is_empty());
    }

    #[test]
    fn harness_split_preserves_order_and_discards_for_text_only() {
        let mut vision = vec![image("existing", "image/webp")];
        split_tool_layer_for_harness(
            false,
            &mut vision,
            vec![image("first", "image/png"), image("second", "image/jpeg")],
        );
        assert_eq!(
            vision.iter().map(|i| i.data.as_str()).collect::<Vec<_>>(),
            ["existing", "first", "second"]
        );

        split_tool_layer_for_harness(true, &mut vision, vec![image("discarded", "image/png")]);
        assert_eq!(vision.len(), 3);
    }
}
