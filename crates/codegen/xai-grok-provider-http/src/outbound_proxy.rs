//! Conservative outbound proxy selection for resolver-aware HTTP clients.
//!
//! When enabled, platform system discovery is tried first, explicit environment
//! proxies are the fallback, and the final fallback is a direct connection.
//! When disabled, callers retain the existing reqwest builder behavior.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use tokio::sync::Semaphore;

use sha2::Digest;
use sha2::Sha256;
use thiserror::Error;

const SYSTEM_PROXY_SUCCESS_CACHE_TTL: Duration = Duration::from_secs(60);
const SYSTEM_PROXY_UNAVAILABLE_CACHE_TTL: Duration = Duration::from_secs(5);
const SYSTEM_PROXY_CACHE_MAX_ENTRIES: usize = 256;
const MAX_CACHED_CODEX_ROUTES: usize = 16;
#[cfg(any(target_os = "windows", target_os = "macos"))]
static ASYNC_SYSTEM_PROXY_RESOLUTION_PERMIT: Semaphore = Semaphore::const_new(1);

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Coarse semantic bucket for the HTTP or WebSocket client being constructed.
///
/// This is not the selected proxy route or a concrete endpoint. It labels the
/// product path that owns the client so proxy-resolution diagnostics can
/// distinguish auth, API, WebSocket, and miscellaneous traffic without exposing
/// endpoint details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRouteClass {
    /// Login, token refresh/revoke, PAT, and agent identity auth traffic.
    Auth,
    /// First-party API traffic that is not part of the auth flow.
    Api,
    /// WebSocket traffic.
    WebSocket,
    /// Call sites without a more specific route class.
    Other,
}

impl fmt::Display for ClientRouteClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auth => "auth",
            Self::Api => "api",
            Self::WebSocket => "wss",
            Self::Other => "other",
        })
    }
}

/// Coarse failure class for route selection errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteFailureClass {
    ProxyResolutionUnavailable,
    ConnectTimeout,
    ProxyAuthenticationRequired,
    TlsError,
    InvalidProxyConfig,
    UnsupportedProxyScheme,
    ResolverError,
}

impl fmt::Display for RouteFailureClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ProxyResolutionUnavailable => "proxy_resolution_unavailable",
            Self::ConnectTimeout => "connect_timeout",
            Self::ProxyAuthenticationRequired => "proxy_407",
            Self::TlsError => "tls_error",
            Self::InvalidProxyConfig => "invalid_proxy_config",
            Self::UnsupportedProxyScheme => "unsupported_proxy_scheme",
            Self::ResolverError => "resolver_error",
        })
    }
}

/// Resolved outbound proxy behavior for HTTP clients.
///
/// Callers must choose a policy explicitly so omitting feature resolution cannot silently select
/// legacy behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboundProxyPolicy {
    /// Preserve reqwest's built-in proxy behavior.
    ReqwestDefault,
    /// Resolve system/PAC/WPAD settings, then environment settings, then direct routing.
    RespectSystemProxy,
}

/// Resolved proxy route for a concrete outbound destination.
///
/// `TransportDefault` preserves the underlying transport behavior only when system-proxy support
/// is disabled. When system resolution is enabled, environment and direct fallbacks are resolved
/// explicitly so the transport cannot repeat system discovery. Proxy URLs and no-proxy settings
/// are intentionally redacted from `Debug` output because they may contain credentials or private
/// hostnames.
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum OutboundProxyRoute {
    /// Preserve the underlying transport's existing proxy behavior.
    TransportDefault,
    /// Connect directly and bypass transport-level proxy discovery.
    Direct,
    /// Connect through the selected proxy URL.
    Proxy {
        url: String,
        no_proxy: Option<String>,
    },
}

impl fmt::Debug for OutboundProxyRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransportDefault => f.write_str("TransportDefault"),
            Self::Direct => f.write_str("Direct"),
            Self::Proxy { .. } => f
                .debug_struct("Proxy")
                .field("url", &"<redacted>")
                .field("no_proxy", &"<redacted>")
                .finish(),
        }
    }
}

/// Builds route-specific HTTP clients using one resolved outbound proxy policy.
///
/// Construct this once from the effective application configuration and carry it with the
/// session or component that owns outbound requests. Individual request paths should supply only
/// their destination and route class rather than resolving feature state themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientFactory {
    outbound_proxy_policy: OutboundProxyPolicy,
}

impl HttpClientFactory {
    /// Creates a factory from the outbound proxy policy resolved by the application.
    pub const fn new(outbound_proxy_policy: OutboundProxyPolicy) -> Self {
        Self {
            outbound_proxy_policy,
        }
    }

    /// Returns the outbound proxy policy used for clients built by this factory.
    pub const fn outbound_proxy_policy(&self) -> OutboundProxyPolicy {
        self.outbound_proxy_policy
    }

    /// Resolves the proxy route for a concrete destination.
    ///
    /// WebSocket schemes are resolved through their HTTP equivalents so platform PAC and system
    /// proxy APIs apply the same policy to `ws`/`wss` and `http`/`https` destinations. When system
    /// resolution is unavailable, explicit environment settings are resolved before falling back
    /// to a direct route.
    pub fn resolve_proxy_route(&self, request_url: &str) -> OutboundProxyRoute {
        resolve_proxy_route(
            &ProcessEnv,
            request_url,
            self.outbound_proxy_policy,
            resolve_system_proxy,
        )
    }

    /// Resolves the proxy route for a concrete destination without blocking a Tokio worker.
    pub async fn resolve_proxy_route_async(
        &self,
        request_url: String,
    ) -> io::Result<OutboundProxyRoute> {
        if matches!(
            self.outbound_proxy_policy,
            OutboundProxyPolicy::ReqwestDefault
        ) {
            return Ok(OutboundProxyRoute::TransportDefault);
        }

        if let Some(route) = self.cached_proxy_route(&request_url) {
            return Ok(route);
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        return Ok(self.resolve_proxy_route(&request_url));

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            let permit = ASYNC_SYSTEM_PROXY_RESOLUTION_PERMIT
                .acquire()
                .await
                .map_err(io::Error::other)?;
            let factory = self.clone();
            tokio::task::spawn_blocking(move || {
                // Keep the permit with the blocking task: cancelling the caller must not allow a
                // second PAC/WinHTTP lookup to start while this one is still running.
                let _permit = permit;
                factory.resolve_proxy_route(&request_url)
            })
            .await
            .map_err(io::Error::other)
        }
    }

    fn cached_proxy_route(&self, request_url: &str) -> Option<OutboundProxyRoute> {
        let env_proxy_kind = EnvProxyKind::from_request_url(request_url);
        let request_url = proxy_resolution_url(request_url);
        if RequestOrigin::parse(&request_url).is_none() {
            return Some(OutboundProxyRoute::Direct);
        }
        cached_system_proxy_decision(&request_url)
            .map(|decision| route_from_system_decision(&ProcessEnv, env_proxy_kind, decision))
    }
}

/// Provider-scoped pool that selects a Codex transport from each complete request URL.
///
/// PAC rules may depend on the URL path or query, not only its origin. Callers therefore ask the
/// pool for a client only after constructing the final endpoint. Clients are reused by resolved
/// route, while each request is looked up by exact URL subject to the bounded platform decision
/// cache. Redirect policy remains owned by each caller; credential-bearing Enhanced clients
/// continue to refuse redirects.
#[derive(Clone)]
pub struct OpenAiCodexClientPool {
    factory: HttpClientFactory,
    route_class: ClientRouteClass,
    builder_factory: Arc<dyn Fn() -> reqwest::ClientBuilder + Send + Sync>,
    clients: Arc<Mutex<HashMap<OutboundProxyRoute, reqwest::Client>>>,
}

impl fmt::Debug for OpenAiCodexClientPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAiCodexClientPool")
            .field("route_class", &self.route_class)
            .finish_non_exhaustive()
    }
}

impl OpenAiCodexClientPool {
    /// Create a pool whose builder factory carries the owning component's timeout, redirect,
    /// protocol, header, and cookie policy.
    pub fn new(
        route_class: ClientRouteClass,
        builder_factory: impl Fn() -> reqwest::ClientBuilder + Send + Sync + 'static,
    ) -> Self {
        Self {
            factory: HttpClientFactory::new(OutboundProxyPolicy::RespectSystemProxy),
            route_class,
            builder_factory: Arc::new(builder_factory),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Resolve the complete outbound URL and return a reusable client for its selected route.
    pub async fn client_for_url(
        &self,
        request_url: &str,
    ) -> Result<reqwest::Client, BuildRouteAwareHttpClientError> {
        let factory = self.factory.clone();
        self.client_for_url_with_resolver(request_url, move |request_url| async move {
            factory.resolve_proxy_route_async(request_url).await
        })
        .await
    }

    async fn client_for_url_with_resolver<F, Fut>(
        &self,
        request_url: &str,
        resolve_route: F,
    ) -> Result<reqwest::Client, BuildRouteAwareHttpClientError>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = io::Result<OutboundProxyRoute>>,
    {
        let route = resolve_route(request_url.to_owned()).await.map_err(|_| {
            BuildRouteAwareHttpClientError::ProxyResolution {
                route_class: self.route_class,
            }
        })?;

        if let Some(client) = self
            .clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&route)
            .cloned()
        {
            return Ok(client);
        }

        let builder = configure_builder_for_resolved_route(
            (self.builder_factory)(),
            self.route_class,
            &route,
        )?;
        let client = builder
            .build()
            .map_err(|_| BuildRouteAwareHttpClientError::ClientBuild {
                route_class: self.route_class,
            })?;

        let mut clients = self
            .clients
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(existing) = clients.get(&route) {
            return Ok(existing.clone());
        }
        if clients.len() >= MAX_CACHED_CODEX_ROUTES
            && let Some(route_to_evict) = clients.keys().next().cloned()
        {
            clients.remove(&route_to_evict);
        }
        clients.insert(route, client.clone());
        Ok(client)
    }
}

/// Build one OpenAI Codex HTTP client with an explicit destination route.
///
/// The caller supplies all ordinary transport policy (timeouts, redirect
/// refusal, pooling, and protocol version). This function only resolves and
/// applies the provider-scoped proxy route, then builds the client. Errors are
/// fixed-shape so proxy credentials, bypass lists, and destination hostnames
/// cannot enter diagnostics.
pub async fn build_openai_codex_client(
    builder: reqwest::ClientBuilder,
    request_url: &str,
    route_class: ClientRouteClass,
) -> Result<reqwest::Client, BuildRouteAwareHttpClientError> {
    let factory = HttpClientFactory::new(OutboundProxyPolicy::RespectSystemProxy);
    let route = factory
        .resolve_proxy_route_async(request_url.to_owned())
        .await
        .map_err(|_| BuildRouteAwareHttpClientError::ProxyResolution { route_class })?;
    let builder = configure_builder_for_resolved_route(builder, route_class, &route)?;
    builder
        .build()
        .map_err(|_| BuildRouteAwareHttpClientError::ClientBuild { route_class })
}

fn resolve_proxy_route(
    env: &dyn EnvSource,
    request_url: &str,
    outbound_proxy_policy: OutboundProxyPolicy,
    resolve_system_proxy: impl FnOnce(&str, &RequestOrigin) -> SystemProxyDecision,
) -> OutboundProxyRoute {
    if matches!(outbound_proxy_policy, OutboundProxyPolicy::ReqwestDefault) {
        return OutboundProxyRoute::TransportDefault;
    }

    let env_proxy_kind = EnvProxyKind::from_request_url(request_url);
    let request_url = proxy_resolution_url(request_url);
    let Some(origin) = RequestOrigin::parse(&request_url) else {
        return OutboundProxyRoute::Direct;
    };

    route_from_system_decision(
        env,
        env_proxy_kind,
        resolve_system_proxy(&request_url, &origin),
    )
}

fn route_from_system_decision(
    env: &dyn EnvSource,
    env_proxy_kind: EnvProxyKind,
    decision: SystemProxyDecision,
) -> OutboundProxyRoute {
    match decision {
        SystemProxyDecision::Direct => OutboundProxyRoute::Direct,
        SystemProxyDecision::Proxy { url } => OutboundProxyRoute::Proxy {
            url,
            no_proxy: None,
        },
        SystemProxyDecision::Unavailable { .. } => resolve_env_proxy_route(env, env_proxy_kind),
    }
}

fn resolve_env_proxy_route(
    env: &dyn EnvSource,
    env_proxy_kind: EnvProxyKind,
) -> OutboundProxyRoute {
    let proxy_url = match env_proxy_kind {
        EnvProxyKind::Https => {
            proxy_env_value(env, "HTTPS_PROXY").or_else(|| proxy_env_value(env, "ALL_PROXY"))
        }
        EnvProxyKind::SecureWebSocket => proxy_env_value(env, "HTTPS_PROXY")
            .or_else(|| proxy_env_value(env, "HTTP_PROXY"))
            .or_else(|| proxy_env_value(env, "ALL_PROXY")),
        EnvProxyKind::Http => {
            proxy_env_value(env, "HTTP_PROXY").or_else(|| proxy_env_value(env, "ALL_PROXY"))
        }
        EnvProxyKind::Other => proxy_env_value(env, "ALL_PROXY"),
    };
    match proxy_url {
        Some(url) => OutboundProxyRoute::Proxy {
            url,
            no_proxy: proxy_env_value(env, "NO_PROXY"),
        },
        None => OutboundProxyRoute::Direct,
    }
}

#[derive(Clone, Copy)]
enum EnvProxyKind {
    Http,
    Https,
    SecureWebSocket,
    Other,
}

impl EnvProxyKind {
    fn from_request_url(request_url: &str) -> Self {
        let scheme = request_url
            .parse::<http::Uri>()
            .ok()
            .and_then(|uri| uri.scheme_str().map(str::to_ascii_lowercase));
        match scheme.as_deref() {
            Some("http" | "ws") => Self::Http,
            Some("https") => Self::Https,
            Some("wss") => Self::SecureWebSocket,
            Some(_) | None => Self::Other,
        }
    }
}

fn proxy_resolution_url(request_url: &str) -> Cow<'_, str> {
    if let Some(suffix) = request_url.strip_prefix("wss://") {
        Cow::Owned(format!("https://{suffix}"))
    } else if let Some(suffix) = request_url.strip_prefix("ws://") {
        Cow::Owned(format!("http://{suffix}"))
    } else {
        Cow::Borrowed(request_url)
    }
}

/// Fixed-shape failure while resolving or building a provider-scoped client.
/// Neither the source error nor any route material is retained.
#[derive(Debug, Error)]
pub enum BuildRouteAwareHttpClientError {
    #[error("Failed to resolve outbound proxy for {route_class}")]
    ProxyResolution { route_class: ClientRouteClass },

    #[error("Failed to configure outbound proxy selected for {route_class}")]
    InvalidProxyConfig { route_class: ClientRouteClass },

    #[error("Failed to build route-aware HTTP client for {route_class}")]
    ClientBuild { route_class: ClientRouteClass },
}

fn configure_builder_for_resolved_route(
    builder: reqwest::ClientBuilder,
    route_class: ClientRouteClass,
    route: &OutboundProxyRoute,
) -> Result<reqwest::ClientBuilder, BuildRouteAwareHttpClientError> {
    match route {
        OutboundProxyRoute::TransportDefault => Ok(builder),
        OutboundProxyRoute::Direct => Ok(builder.no_proxy()),
        OutboundProxyRoute::Proxy { url, no_proxy } => {
            let no_proxy = no_proxy.as_deref().and_then(reqwest::NoProxy::from_string);
            configure_concrete_proxy(builder, route_class, url, no_proxy)
        }
    }
}

fn configure_concrete_proxy(
    builder: reqwest::ClientBuilder,
    route_class: ClientRouteClass,
    proxy_url: &str,
    no_proxy: Option<reqwest::NoProxy>,
) -> Result<reqwest::ClientBuilder, BuildRouteAwareHttpClientError> {
    let proxy = match reqwest::Proxy::all(proxy_url) {
        Ok(proxy) => proxy,
        Err(_source) => {
            return Err(BuildRouteAwareHttpClientError::InvalidProxyConfig { route_class });
        }
    };
    Ok(builder.proxy(proxy.no_proxy(no_proxy)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
struct RequestOrigin {
    scheme: String,
    host: String,
    port: u16,
}

impl RequestOrigin {
    fn parse(request_url: &str) -> Option<Self> {
        let uri = request_url.parse::<http::Uri>().ok()?;
        let scheme = uri.scheme_str()?.to_ascii_lowercase();
        let host = uri.host()?.trim_matches(['[', ']']).to_ascii_lowercase();
        let port = uri.port_u16().or(match scheme.as_str() {
            "http" | "ws" => Some(80),
            "https" | "wss" => Some(443),
            _ => None,
        })?;
        Some(Self { scheme, host, port })
    }
}

#[cfg_attr(
    not(any(target_os = "windows", target_os = "macos")),
    allow(
        dead_code,
        reason = "Direct and Proxy are constructed only by platform-specific resolvers"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum SystemProxyDecision {
    Direct,
    Proxy { url: String },
    Unavailable { failure: RouteFailureClass },
}

fn resolve_system_proxy(request_url: &str, origin: &RequestOrigin) -> SystemProxyDecision {
    let cache = SYSTEM_PROXY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    resolve_system_proxy_with(cache, request_url, origin, resolve_platform_system_proxy)
}

fn resolve_system_proxy_with(
    cache: &Mutex<HashMap<String, CachedSystemProxyDecision>>,
    request_url: &str,
    origin: &RequestOrigin,
    resolve_platform_system_proxy: impl FnOnce(&str, &RequestOrigin) -> SystemProxyDecision,
) -> SystemProxyDecision {
    let mut cache = match cache.lock() {
        Ok(cache) => cache,
        Err(error) => panic!("system proxy cache lock should not be poisoned: {error}"),
    };
    let cache_key = system_proxy_cache_key(request_url);
    if let Some(decision) =
        cached_system_proxy_decision_from_cache(&mut cache, &cache_key, Instant::now())
    {
        return decision;
    }

    // Keep cache misses single-flight. Platform PAC/WPAD APIs are synchronous, so async callers
    // run this work on the blocking pool; serializing misses prevents concurrent requests from
    // consuming an unbounded number of blocking workers while system lookup is pending.
    let decision = resolve_platform_system_proxy(request_url, origin);
    insert_system_proxy_cache_entry(&mut cache, &cache_key, decision.clone(), Instant::now());
    decision
}

#[cfg(target_os = "macos")]
fn resolve_platform_system_proxy(request_url: &str, origin: &RequestOrigin) -> SystemProxyDecision {
    macos::resolve(request_url, origin)
}

#[cfg(target_os = "windows")]
fn resolve_platform_system_proxy(request_url: &str, origin: &RequestOrigin) -> SystemProxyDecision {
    windows::resolve(request_url, origin)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn resolve_platform_system_proxy(
    _request_url: &str,
    _origin: &RequestOrigin,
) -> SystemProxyDecision {
    SystemProxyDecision::Unavailable {
        failure: RouteFailureClass::ProxyResolutionUnavailable,
    }
}

#[derive(Debug, Clone)]
struct CachedSystemProxyDecision {
    decision: SystemProxyDecision,
    expires_at: Instant,
}

static SYSTEM_PROXY_CACHE: OnceLock<Mutex<HashMap<String, CachedSystemProxyDecision>>> =
    OnceLock::new();

fn cached_system_proxy_decision(request_url: &str) -> Option<SystemProxyDecision> {
    let cache = SYSTEM_PROXY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().ok()?;
    let key = system_proxy_cache_key(request_url);
    cached_system_proxy_decision_from_cache(&mut cache, &key, Instant::now())
}

fn cached_system_proxy_decision_from_cache(
    cache: &mut HashMap<String, CachedSystemProxyDecision>,
    cache_key: &str,
    now: Instant,
) -> Option<SystemProxyDecision> {
    let cached = cache.get(cache_key)?;
    if cached.expires_at > now {
        return Some(cached.decision.clone());
    }
    cache.remove(cache_key);
    None
}

#[cfg(test)]
fn cache_system_proxy_decision(request_url: &str, decision: SystemProxyDecision) {
    let cache = SYSTEM_PROXY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        let cache_key = system_proxy_cache_key(request_url);
        insert_system_proxy_cache_entry(&mut cache, &cache_key, decision, Instant::now());
    }
}

fn insert_system_proxy_cache_entry(
    cache: &mut HashMap<String, CachedSystemProxyDecision>,
    cache_key: &str,
    decision: SystemProxyDecision,
    now: Instant,
) {
    let ttl = match &decision {
        SystemProxyDecision::Direct | SystemProxyDecision::Proxy { .. } => {
            SYSTEM_PROXY_SUCCESS_CACHE_TTL
        }
        SystemProxyDecision::Unavailable { .. } => SYSTEM_PROXY_UNAVAILABLE_CACHE_TTL,
    };

    cache.retain(|_, cached| cached.expires_at > now);
    if cache.len() >= SYSTEM_PROXY_CACHE_MAX_ENTRIES
        && !cache.contains_key(cache_key)
        && let Some(cache_key_to_evict) = cache
            .iter()
            .min_by_key(|(_, cached)| cached.expires_at)
            .map(|(cache_key, _)| cache_key.clone())
    {
        cache.remove(&cache_key_to_evict);
    }
    cache.insert(
        cache_key.to_string(),
        CachedSystemProxyDecision {
            decision,
            expires_at: now + ttl,
        },
    );
}

fn system_proxy_cache_key(request_url: &str) -> String {
    // Keep URL-specific PAC decisions without retaining the raw routed URL.
    let mut hasher = Sha256::new();
    hasher.update(b"system-proxy-cache-v1\0");
    hasher.update(request_url.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(any(test, target_os = "windows"))]
fn no_proxy_matches_origin(no_proxy: &str, origin: &RequestOrigin) -> bool {
    no_proxy
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .any(|entry| no_proxy_entry_matches_origin(entry, origin))
}

#[cfg(any(test, target_os = "windows"))]
fn no_proxy_entry_matches_origin(entry: &str, origin: &RequestOrigin) -> bool {
    if entry == "*" {
        return true;
    }

    let mut entry = entry
        .strip_prefix("http://")
        .or_else(|| entry.strip_prefix("https://"))
        .unwrap_or(entry)
        .trim_matches(['[', ']'])
        .to_ascii_lowercase();
    let mut port = None;
    let parsed_host_port = entry.rsplit_once(':').and_then(|(host, candidate_port)| {
        if host.contains(':') {
            return None;
        }
        candidate_port
            .parse::<u16>()
            .ok()
            .map(|parsed_port| (host.to_string(), parsed_port))
    });
    if let Some((host, parsed_port)) = parsed_host_port {
        entry = host;
        port = Some(parsed_port);
    }
    if port.is_some_and(|port| port != origin.port) {
        return false;
    }

    if let Some(suffix) = entry.strip_prefix('.') {
        return origin.host == suffix || origin.host.ends_with(&format!(".{suffix}"));
    }

    if entry.contains('*') {
        return wildcard_host_match(&entry, &origin.host);
    }

    origin.host == entry
}

#[cfg(any(test, target_os = "windows"))]
fn wildcard_host_match(pattern: &str, host: &str) -> bool {
    let mut remaining = host;
    let mut first = true;
    for part in pattern.split('*') {
        if part.is_empty() {
            continue;
        }
        if first && !pattern.starts_with('*') {
            let Some(stripped) = remaining.strip_prefix(part) else {
                return false;
            };
            remaining = stripped;
        } else {
            let Some(index) = remaining.find(part) else {
                return false;
            };
            remaining = &remaining[index + part.len()..];
        }
        first = false;
    }
    pattern.ends_with('*') || remaining.is_empty()
}

#[cfg(any(test, target_os = "windows"))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedProxyListDecision {
    Direct,
    Proxy(String),
    UnsupportedScheme,
    Unavailable,
}

#[cfg(any(test, target_os = "windows"))]
fn parse_proxy_list(input: &str, target_scheme: &str) -> ParsedProxyListDecision {
    let mut saw_unsupported = false;

    {
        let mut process_token = |token: &str| {
            let decision = parse_proxy_token(token, target_scheme);
            match decision {
                ParsedProxyListDecision::Direct => Some(ParsedProxyListDecision::Direct),
                ParsedProxyListDecision::Proxy(url) => Some(ParsedProxyListDecision::Proxy(url)),
                ParsedProxyListDecision::UnsupportedScheme => {
                    saw_unsupported = true;
                    None
                }
                ParsedProxyListDecision::Unavailable => None,
            }
        };

        for segment in input
            .split(';')
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
        {
            let mut parts = segment.split_whitespace();
            let directive = parts.next();
            let hostport = parts.next();
            let extra = parts.next();
            let is_proxy_directive = matches!(
                directive.map(str::to_ascii_lowercase).as_deref(),
                Some("proxy" | "http" | "https" | "socks" | "socks4" | "socks5")
            ) && hostport.is_some()
                && extra.is_none();

            if is_proxy_directive {
                if let Some(decision) = process_token(segment) {
                    return decision;
                }
            } else {
                for token in segment.split_whitespace() {
                    if let Some(decision) = process_token(token) {
                        return decision;
                    }
                }
            }
        }
    }

    if saw_unsupported {
        ParsedProxyListDecision::UnsupportedScheme
    } else {
        ParsedProxyListDecision::Unavailable
    }
}

#[cfg(any(test, target_os = "windows"))]
fn parse_proxy_token(token: &str, target_scheme: &str) -> ParsedProxyListDecision {
    if token.eq_ignore_ascii_case("DIRECT") {
        return ParsedProxyListDecision::Direct;
    }

    if let Some(decision) = parse_proxy_key_token(token, target_scheme) {
        return decision;
    }
    if token.contains('=') {
        return ParsedProxyListDecision::Unavailable;
    }

    let mut parts = token.split_whitespace();
    let directive = parts.next();
    let hostport = parts.next();
    if let (Some(directive), Some(hostport), None) = (directive, hostport, parts.next()) {
        return match directive.to_ascii_lowercase().as_str() {
            "proxy" | "http" => proxy_url_from_hostport("http", hostport),
            "https" => proxy_url_from_hostport("https", hostport),
            "socks" | "socks4" | "socks5" => ParsedProxyListDecision::UnsupportedScheme,
            _ => ParsedProxyListDecision::Unavailable,
        };
    }

    proxy_url_from_hostport("http", token)
}

#[cfg(any(test, target_os = "windows"))]
fn parse_proxy_key_token(token: &str, target_scheme: &str) -> Option<ParsedProxyListDecision> {
    let (key, value) = token.split_once('=')?;
    if key.trim().eq_ignore_ascii_case(target_scheme) {
        Some(proxy_url_from_hostport("http", value.trim()))
    } else {
        Some(ParsedProxyListDecision::Unavailable)
    }
}

#[cfg(any(test, target_os = "windows"))]
fn proxy_url_from_hostport(proxy_scheme: &str, hostport: &str) -> ParsedProxyListDecision {
    if hostport.is_empty() {
        return ParsedProxyListDecision::Unavailable;
    }
    if hostport.contains("://") {
        return ParsedProxyListDecision::Proxy(hostport.to_string());
    }
    ParsedProxyListDecision::Proxy(format!("{proxy_scheme}://{hostport}"))
}

trait EnvSource {
    fn var(&self, key: &str) -> Option<String>;
}

struct ProcessEnv;

impl EnvSource for ProcessEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

fn proxy_env_value(env: &dyn EnvSource, upper: &str) -> Option<String> {
    let lower = upper.to_ascii_lowercase();
    env.var(upper)
        .or_else(|| env.var(&lower))
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestEnv(std::collections::HashMap<String, String>);

    impl EnvSource for TestEnv {
        fn var(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
    }

    fn unavailable_system_proxy(_: &str, _: &RequestOrigin) -> SystemProxyDecision {
        SystemProxyDecision::Unavailable {
            failure: RouteFailureClass::ProxyResolutionUnavailable,
        }
    }

    #[tokio::test]
    async fn codex_pool_resolves_the_complete_request_url() {
        let seen = Arc::new(Mutex::new(None));
        let seen_by_resolver = Arc::clone(&seen);
        let pool = OpenAiCodexClientPool::new(ClientRouteClass::Api, reqwest::Client::builder);

        pool.client_for_url_with_resolver(
            "https://chatgpt.com/backend-api/codex/models?client_version=0.2.6",
            move |request_url| {
                *seen_by_resolver
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(request_url);
                async { Ok(OutboundProxyRoute::Direct) }
            },
        )
        .await
        .expect("exact request route should build");

        assert_eq!(
            seen.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_deref(),
            Some("https://chatgpt.com/backend-api/codex/models?client_version=0.2.6")
        );
    }

    #[tokio::test]
    async fn codex_pool_reuses_a_client_when_distinct_urls_select_the_same_route() {
        let builds = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let builds_by_factory = Arc::clone(&builds);
        let pool = OpenAiCodexClientPool::new(ClientRouteClass::Api, move || {
            builds_by_factory.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            reqwest::Client::builder()
        });

        for request_url in [
            "https://chatgpt.com/backend-api/codex/responses",
            "https://chatgpt.com/backend-api/codex/models",
        ] {
            pool.client_for_url_with_resolver(request_url, |_| async {
                Ok(OutboundProxyRoute::Direct)
            })
            .await
            .expect("shared direct route should build");
        }

        assert_eq!(
            builds.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "one transport should serve every URL selecting the same route"
        );
    }

    #[test]
    fn https_environment_fallback_prefers_https_proxy_over_all_proxy() {
        let env = TestEnv(std::collections::HashMap::from([
            (
                "HTTPS_PROXY".to_owned(),
                "http://https-proxy.test".to_owned(),
            ),
            ("ALL_PROXY".to_owned(), "http://all-proxy.test".to_owned()),
            ("NO_PROXY".to_owned(), ".internal.test".to_owned()),
        ]));

        let route = resolve_proxy_route(
            &env,
            "https://chatgpt.com/backend-api/codex/responses",
            OutboundProxyPolicy::RespectSystemProxy,
            unavailable_system_proxy,
        );

        assert_eq!(
            route,
            OutboundProxyRoute::Proxy {
                url: "http://https-proxy.test".to_owned(),
                no_proxy: Some(".internal.test".to_owned()),
            }
        );
    }

    #[test]
    fn lowercase_environment_proxy_is_used_when_uppercase_is_absent() {
        let env = TestEnv(std::collections::HashMap::from([(
            "https_proxy".to_owned(),
            "http://lowercase-proxy.test".to_owned(),
        )]));

        let route = resolve_proxy_route(
            &env,
            "https://chatgpt.com/backend-api/codex/responses",
            OutboundProxyPolicy::RespectSystemProxy,
            unavailable_system_proxy,
        );

        assert_eq!(
            route,
            OutboundProxyRoute::Proxy {
                url: "http://lowercase-proxy.test".to_owned(),
                no_proxy: None,
            }
        );
    }

    #[test]
    fn unavailable_system_and_environment_routes_resolve_directly() {
        let route = resolve_proxy_route(
            &TestEnv::default(),
            "https://chatgpt.com/backend-api/codex/responses",
            OutboundProxyPolicy::RespectSystemProxy,
            unavailable_system_proxy,
        );

        assert_eq!(route, OutboundProxyRoute::Direct);
    }

    #[test]
    fn route_debug_redacts_proxy_credentials_and_bypass_hosts() {
        let route = OutboundProxyRoute::Proxy {
            url: "http://user:password@private-proxy.test".to_owned(),
            no_proxy: Some("secret.internal.test".to_owned()),
        };

        let debug = format!("{route:?}");

        assert_eq!(
            debug,
            "Proxy { url: \"<redacted>\", no_proxy: \"<redacted>\" }"
        );
        assert!(!debug.contains("password"));
        assert!(!debug.contains("secret.internal.test"));
    }

    #[test]
    fn system_proxy_cache_keys_do_not_retain_destination_urls() {
        let url = "https://private.internal.test/secret/path";

        let key = system_proxy_cache_key(url);

        assert_eq!(key.len(), 64);
        assert!(!key.contains("private.internal.test"));
        assert!(!key.contains("secret/path"));
    }

    #[test]
    fn invalid_proxy_errors_do_not_retain_proxy_values() {
        let route = OutboundProxyRoute::Proxy {
            url: "not a valid proxy value with secret-password".to_owned(),
            no_proxy: Some("secret.internal.test".to_owned()),
        };

        let error = configure_builder_for_resolved_route(
            reqwest::Client::builder(),
            ClientRouteClass::Api,
            &route,
        )
        .expect_err("invalid proxy must fail before client build");
        let rendered = format!("{error:?} {error}");

        assert!(!rendered.contains("secret-password"));
        assert!(!rendered.contains("secret.internal.test"));
        assert!(rendered.contains("InvalidProxyConfig"));
    }

    #[test]
    fn cache_entries_use_shorter_ttl_for_unavailable_decisions() {
        let now = Instant::now();
        let mut cache = HashMap::new();
        insert_system_proxy_cache_entry(&mut cache, "success", SystemProxyDecision::Direct, now);
        insert_system_proxy_cache_entry(
            &mut cache,
            "unavailable",
            SystemProxyDecision::Unavailable {
                failure: RouteFailureClass::ProxyResolutionUnavailable,
            },
            now,
        );

        assert_eq!(cache["success"].expires_at, now + Duration::from_secs(60));
        assert_eq!(
            cache["unavailable"].expires_at,
            now + Duration::from_secs(5)
        );
    }

    #[test]
    fn cached_system_decisions_skip_platform_resolution() {
        let cache = Mutex::new(HashMap::new());
        let origin = RequestOrigin::parse("https://chatgpt.com").expect("valid origin");
        let now = Instant::now();
        let key = system_proxy_cache_key("https://chatgpt.com");
        cache.lock().expect("cache lock").insert(
            key,
            CachedSystemProxyDecision {
                decision: SystemProxyDecision::Direct,
                expires_at: now + Duration::from_secs(60),
            },
        );

        let decision = resolve_system_proxy_with(&cache, "https://chatgpt.com", &origin, |_, _| {
            panic!("cached resolution must not call the platform resolver")
        });

        assert_eq!(decision, SystemProxyDecision::Direct);
    }

    #[test]
    fn websocket_route_uses_the_http_equivalent_for_system_resolution() {
        let route = resolve_proxy_route(
            &TestEnv::default(),
            "wss://chatgpt.com/backend-api/codex/responses",
            OutboundProxyPolicy::RespectSystemProxy,
            |request_url, origin| {
                assert_eq!(
                    request_url,
                    "https://chatgpt.com/backend-api/codex/responses"
                );
                assert_eq!(origin.scheme, "https");
                assert_eq!(origin.host, "chatgpt.com");
                assert_eq!(origin.port, 443);
                SystemProxyDecision::Proxy {
                    url: "http://proxy.example:8080".to_owned(),
                }
            },
        );

        assert_eq!(
            route,
            OutboundProxyRoute::Proxy {
                url: "http://proxy.example:8080".to_owned(),
                no_proxy: None,
            }
        );
    }

    #[test]
    fn reqwest_default_policy_skips_explicit_proxy_resolution() {
        let route = resolve_proxy_route(
            &TestEnv::default(),
            "https://chatgpt.com/backend-api/codex/responses",
            OutboundProxyPolicy::ReqwestDefault,
            |_, _| panic!("default policy must not resolve system proxy settings"),
        );

        assert_eq!(route, OutboundProxyRoute::TransportDefault);
    }

    #[test]
    fn environment_proxy_casing_matches_reqwest_precedence() {
        let env = TestEnv(HashMap::from([
            ("HTTPS_PROXY".to_owned(), "upper".to_owned()),
            ("https_proxy".to_owned(), "lower".to_owned()),
            ("http_proxy".to_owned(), "lower-only".to_owned()),
            ("ALL_PROXY".to_owned(), String::new()),
            ("all_proxy".to_owned(), "masked".to_owned()),
        ]));

        assert_eq!(
            proxy_env_value(&env, "HTTPS_PROXY").as_deref(),
            Some("upper")
        );
        assert_eq!(
            proxy_env_value(&env, "HTTP_PROXY").as_deref(),
            Some("lower-only")
        );
        assert_eq!(proxy_env_value(&env, "ALL_PROXY"), None);
    }

    #[test]
    fn pac_proxy_tokens_select_the_first_supported_route() {
        assert_eq!(
            parse_proxy_list("PROXY proxy.internal:8080; DIRECT", "https"),
            ParsedProxyListDecision::Proxy("http://proxy.internal:8080".to_owned())
        );
        assert_eq!(
            parse_proxy_list("HTTPS proxy.internal:8443", "https"),
            ParsedProxyListDecision::Proxy("https://proxy.internal:8443".to_owned())
        );
    }

    #[test]
    fn winhttp_static_proxy_entries_select_the_target_scheme() {
        assert_eq!(
            parse_proxy_list("http=web-proxy:8080;https=secure-proxy:8443", "https"),
            ParsedProxyListDecision::Proxy("http://secure-proxy:8443".to_owned())
        );
        assert_eq!(
            parse_proxy_list("http=web-proxy:8080", "https"),
            ParsedProxyListDecision::Unavailable
        );
        assert_eq!(
            parse_proxy_list("proxy.internal:8080", "https"),
            ParsedProxyListDecision::Proxy("http://proxy.internal:8080".to_owned())
        );
    }

    #[test]
    fn direct_and_unsupported_proxy_tokens_remain_distinct() {
        assert_eq!(
            parse_proxy_list("DIRECT; PROXY proxy.internal:8080", "https"),
            ParsedProxyListDecision::Direct
        );
        assert_eq!(
            parse_proxy_list("SOCKS proxy.internal:1080", "https"),
            ParsedProxyListDecision::UnsupportedScheme
        );
    }

    #[test]
    fn no_proxy_matching_supports_exact_suffix_wildcard_and_port_entries() {
        let origin = RequestOrigin {
            scheme: "https".to_owned(),
            host: "auth.openai.com".to_owned(),
            port: 443,
        };

        assert!(no_proxy_matches_origin("auth.openai.com", &origin));
        assert!(!no_proxy_matches_origin("openai.com", &origin));
        assert!(no_proxy_matches_origin(".openai.com", &origin));
        assert!(no_proxy_matches_origin("*.openai.com", &origin));
        assert!(no_proxy_matches_origin("auth.openai.com:443", &origin));
        assert!(!no_proxy_matches_origin("auth.openai.com:8443", &origin));
    }

    #[test]
    fn unavailable_system_proxy_decisions_are_cached_briefly() {
        let request_url = "https://unavailable-cache.test/oauth/token";
        let decision = SystemProxyDecision::Unavailable {
            failure: RouteFailureClass::ProxyResolutionUnavailable,
        };

        cache_system_proxy_decision(request_url, decision.clone());

        assert_eq!(cached_system_proxy_decision(request_url), Some(decision));
    }

    #[test]
    fn system_proxy_resolution_serializes_cache_misses() {
        let cache = std::sync::Arc::new(Mutex::new(HashMap::new()));
        let request_url = "https://single-flight.test/models";
        let origin = RequestOrigin::parse(request_url).expect("valid request URL");
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let worker_cache = std::sync::Arc::clone(&cache);
        let worker_origin = origin.clone();

        let worker = std::thread::spawn(move || {
            resolve_system_proxy_with(&worker_cache, request_url, &worker_origin, |_, _| {
                started_tx.send(()).expect("test should still be running");
                release_rx.recv().expect("test should release resolver");
                SystemProxyDecision::Direct
            })
        });

        started_rx.recv().expect("resolver should start");
        assert!(matches!(
            cache.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        ));
        release_tx
            .send(())
            .expect("resolver should still be running");
        assert_eq!(
            worker.join().expect("resolver should finish"),
            SystemProxyDecision::Direct
        );
        assert_eq!(
            resolve_system_proxy_with(&cache, request_url, &origin, |_, _| {
                panic!("cached waiter must not resolve the platform proxy again")
            }),
            SystemProxyDecision::Direct
        );
    }

    #[test]
    fn system_proxy_cache_evicts_entries_at_its_fixed_bound() {
        let mut cache = HashMap::new();
        let now = Instant::now();

        for index in 0..=SYSTEM_PROXY_CACHE_MAX_ENTRIES {
            insert_system_proxy_cache_entry(
                &mut cache,
                &format!("cache-key-{index}"),
                SystemProxyDecision::Direct,
                now,
            );
        }

        assert_eq!(cache.len(), SYSTEM_PROXY_CACHE_MAX_ENTRIES);
    }
}
