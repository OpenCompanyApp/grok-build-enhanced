//! Wire-level tests for the process-wide shared sampling client: connection
//! reuse across `SamplingClient`s, per-config header isolation, and the
//! pool-less HTTP/1.1 fallback. These live in their own integration binary
//! (one process under cargo test, nextest, and Bazel alike) so the
//! environment they pin cannot leak into, or be poisoned by, other tests.

mod support;

use std::sync::Once;
use std::sync::atomic::Ordering;
use std::time::Duration;

use support::{send_one, test_config};
use xai_grok_sampler::SamplingClient;
use xai_grok_test_support::spawn_counting_server;

const CREDENTIAL_A: &str = "opaque-credential-a";
const CREDENTIAL_B: &str = "opaque-credential-b";
const AUTHORIZATION_A: &str = "Bearer opaque-credential-a";
const AUTHORIZATION_B: &str = "Bearer opaque-credential-b";
const EXTRA_A: &str = "isolated-a";
const EXTRA_B: &str = "isolated-b";

/// Pin the env these assertions depend on before any client is built, so
/// ambient shell exports (`GROK_SAMPLER_SHARED_CLIENT=0`,
/// `GROK_POOL_MAX_IDLE=0`) cannot flip the expected pooling behavior.
fn pin_env() {
    static PIN: Once = Once::new();
    PIN.call_once(|| {
        // Safety: runs before any test builds a client or reads these vars;
        // racing tests block on the Once, and the crate latches the kill
        // switch and pool knobs only at first client construction.
        unsafe {
            std::env::remove_var("GROK_SAMPLER_SHARED_CLIENT");
            std::env::set_var("GROK_POOL_MAX_IDLE", "2");
            std::env::set_var("GROK_POOL_IDLE_TIMEOUT_SECS", "90");
        }
    });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_sampling_clients_share_one_connection() {
    pin_env();
    let (base_url, accepts, _heads) = spawn_counting_server().await;
    let a = SamplingClient::new(test_config(&base_url, CREDENTIAL_A)).unwrap();
    let b = SamplingClient::new(test_config(&base_url, CREDENTIAL_B)).unwrap();
    send_one(&a).await;
    // Brief pause so the idle connection is checked back into the pool.
    tokio::time::sleep(Duration::from_millis(50)).await;
    send_one(&b).await;
    assert_eq!(accepts.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shared_client_keeps_per_config_headers_isolated() {
    pin_env();
    let (base_url, _accepts, heads) = spawn_counting_server().await;
    let mut cfg_a = test_config(&base_url, CREDENTIAL_A);
    cfg_a
        .extra_headers
        .insert("x-test-extra".to_string(), EXTRA_A.to_string());
    let mut cfg_b = test_config(&base_url, CREDENTIAL_B);
    cfg_b
        .extra_headers
        .insert("x-test-extra".to_string(), EXTRA_B.to_string());
    let a = SamplingClient::new(cfg_a).unwrap();
    let b = SamplingClient::new(cfg_b).unwrap();
    send_one(&a).await;
    send_one(&b).await;

    let heads = heads.lock().unwrap();
    assert_eq!(heads.len(), 2);
    assert!(heads[0].contains(AUTHORIZATION_A) && heads[0].contains(EXTRA_A));
    assert!(!heads[0].contains(CREDENTIAL_B) && !heads[0].contains(EXTRA_B));
    assert!(heads[1].contains(AUTHORIZATION_B) && heads[1].contains(EXTRA_B));
    assert!(!heads[1].contains(CREDENTIAL_A) && !heads[1].contains(EXTRA_A));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shared_http1_fallback_never_pools() {
    pin_env();
    let (base_url, accepts, _heads) = spawn_counting_server().await;
    let mut cfg = test_config(&base_url, CREDENTIAL_A);
    cfg.force_http1 = true;
    let client = SamplingClient::new(cfg).unwrap();
    send_one(&client).await;
    send_one(&client).await;
    assert_eq!(accepts.load(Ordering::SeqCst), 2);
}
