use super::error::WebFetchError;
use super::hosted_kimi::HostedFetchResult;
use crate::implementations::zai_mcp::{
    READER_ENDPOINT, ZaiMcpClient, ZaiMcpError, text_content,
};
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

    pub(super) async fn fetch(
        &self,
        url: &url::Url,
    ) -> Result<HostedFetchResult, WebFetchError> {
        let result = match self
            .client
            .call_tool("webReader", serde_json::json!({"url": url.as_str()}))
            .await
        {
            Ok(result) => result,
            Err(ZaiMcpError::Authentication) => {
                return Err(WebFetchError::ZaiReaderAuthentication);
            }
            Err(ZaiMcpError::Quota) => return Err(WebFetchError::ZaiReaderQuota),
            Err(ZaiMcpError::Rejected) => return Err(WebFetchError::ZaiReaderRejected),
            // `auto` Reader policy retains the SSRF-safe local implementation
            // as a transport/protocol/service fallback. Credential and quota
            // failures above remain visible and never masquerade as success.
            Err(
                ZaiMcpError::Protocol
                | ZaiMcpError::InvalidResponse
                | ZaiMcpError::Unavailable,
            ) => return Ok(HostedFetchResult::Fallback),
        };
        let mut content = match text_content(&result) {
            Ok(content) => content,
            Err(_) => return Ok(HostedFetchResult::Fallback),
        };
        if content.len() > self.max_content_length {
            content.truncate(self.max_content_length);
            while !content.is_char_boundary(content.len()) {
                content.pop();
            }
        }
        Ok(HostedFetchResult::Content(content))
    }
}
