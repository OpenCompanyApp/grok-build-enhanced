use super::error::WebFetchError;
use super::hosted_kimi::HostedFetchResult;
use crate::implementations::zai_mcp::{READER_ENDPOINT, ZaiMcpClient, ZaiMcpError, text_content};
use crate::types::{SharedApiKeyProvider, ZAI_CODING_PLAN_PROVIDER_ID};

#[derive(Clone)]
pub(super) struct ZaiHostedReader {
    client: ZaiMcpClient,
    max_content_length: usize,
}

impl ZaiHostedReader {
    pub(super) fn new(
        auth_provider: SharedApiKeyProvider,
        max_content_length: usize,
    ) -> Result<Self, WebFetchError> {
        if auth_provider.request_auth_provider_id() != Some(ZAI_CODING_PLAN_PROVIDER_ID) {
            return Err(WebFetchError::ZaiReaderAuthentication);
        }
        let client = ZaiMcpClient::new(READER_ENDPOINT, auth_provider)
            .map_err(|_| WebFetchError::ZaiReaderAuthentication)?;
        Ok(Self {
            client,
            max_content_length,
        })
    }

    pub(super) async fn fetch(&self, url: &url::Url) -> Result<HostedFetchResult, WebFetchError> {
        let result = self
            .client
            .call_tool("webReader", serde_json::json!({"url": url.as_str()}))
            .await;
        project_reader_result(result, self.max_content_length)
    }
}

fn project_reader_result(
    result: Result<serde_json::Value, ZaiMcpError>,
    max_content_length: usize,
) -> Result<HostedFetchResult, WebFetchError> {
    let result = match result {
        Ok(result) => result,
        Err(ZaiMcpError::Authentication) => {
            return Err(WebFetchError::ZaiReaderAuthentication);
        }
        Err(ZaiMcpError::Quota) => return Err(WebFetchError::ZaiReaderQuota),
        Err(ZaiMcpError::Rejected) => return Err(WebFetchError::ZaiReaderRejected),
        // `auto` Reader policy retains the SSRF-safe local implementation
        // as a transport/protocol/service fallback. Credential and quota
        // failures above remain visible and never masquerade as success.
        Err(ZaiMcpError::Protocol | ZaiMcpError::InvalidResponse | ZaiMcpError::Unavailable) => {
            return Ok(HostedFetchResult::Fallback);
        }
    };
    let mut content = match text_content(&result) {
        Ok(content) => content,
        Err(_) => return Ok(HostedFetchResult::Fallback),
    };
    if content.len() > max_content_length {
        let mut end = max_content_length;
        while end > 0 && !content.is_char_boundary(end) {
            end -= 1;
        }
        content.truncate(end);
    }
    Ok(HostedFetchResult::Content(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_projects_text_and_truncates_at_utf8_boundary() {
        let result = serde_json::json!({
            "content": [{"type": "text", "text": "hello 😀 world"}]
        });
        let outcome = project_reader_result(Ok(result), 8).unwrap();
        assert!(matches!(outcome, HostedFetchResult::Content(content) if content == "hello "));
    }

    #[test]
    fn reader_falls_back_only_for_service_and_protocol_failures() {
        for error in [
            ZaiMcpError::Protocol,
            ZaiMcpError::InvalidResponse,
            ZaiMcpError::Unavailable,
        ] {
            assert!(matches!(
                project_reader_result(Err(error), 1024).unwrap(),
                HostedFetchResult::Fallback
            ));
        }
        assert!(matches!(
            project_reader_result(Err(ZaiMcpError::Authentication), 1024),
            Err(WebFetchError::ZaiReaderAuthentication)
        ));
        assert!(matches!(
            project_reader_result(Err(ZaiMcpError::Quota), 1024),
            Err(WebFetchError::ZaiReaderQuota)
        ));
        assert!(matches!(
            project_reader_result(Err(ZaiMcpError::Rejected), 1024),
            Err(WebFetchError::ZaiReaderRejected)
        ));
    }

    #[test]
    fn malformed_reader_content_uses_local_fallback() {
        assert!(matches!(
            project_reader_result(Ok(serde_json::json!({"content": []})), 1024).unwrap(),
            HostedFetchResult::Fallback
        ));
    }
}
