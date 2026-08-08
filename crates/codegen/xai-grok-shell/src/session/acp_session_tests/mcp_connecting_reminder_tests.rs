use super::mcp::format_mcp_connecting_reminder;

#[test]
fn ordinary_client_keeps_plain_text_connecting_guidance() {
    let text = format_mcp_connecting_reminder(&["alpha".to_string()], &[]);
    assert!(text.contains("- alpha\n"));
    assert!(text.contains("proceed with what you can do in the meantime"));
    assert!(!text.contains("Do NOT end the turn"));
}

#[test]
fn delivery_only_client_must_use_declared_tools() {
    let text = format_mcp_connecting_reminder(
        &["alpha".to_string(), "beta".to_string()],
        &["alpha__post".to_string(), "alpha__ask".to_string()],
    );
    assert!(text.contains("- alpha\n- beta\n"));
    assert!(text.contains("delivered ONLY through: alpha__post, alpha__ask"));
    assert!(text.contains("Do NOT end the turn"));
    assert!(!text.contains("Do not attempt to use tools from these servers yet"));
}
