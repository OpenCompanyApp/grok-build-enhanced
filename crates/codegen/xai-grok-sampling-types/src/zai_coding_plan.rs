//! Public Z.AI GLM Coding Plan endpoint and provider constants.
//!
//! These values contain no credentials. Keeping the subscription product,
//! region, and endpoints explicit prevents a Coding Plan key from falling
//! through to Z.AI Open Platform pay-as-you-go or BigModel China services.

pub const ZAI_CODING_PLAN_AUTH_SCOPE: &str = "zai::coding-plan::global";
pub const ZAI_CODING_PLAN_API_KEY_ENV: &str = "Z_AI_API_KEY";

pub const ZAI_CODING_PLAN_BASE_URL: &str = "https://api.z.ai/api/coding/paas/v4";
pub const ZAI_CODING_PLAN_MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
pub const ZAI_CODING_PLAN_MAX_FUNCTION_TOOLS: usize = 128;
