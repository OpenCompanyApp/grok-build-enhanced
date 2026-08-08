use super::{SystemPromptProvenance, infer_system_prompt_provenance};

#[test]
fn provenance_serialization_round_trips_and_legacy_absence_stays_optional() {
    for provenance in [
        SystemPromptProvenance::Custom,
        SystemPromptProvenance::Model {
            model_id: "openai-codex/gpt-5.4".to_string(),
        },
    ] {
        let encoded = serde_json::to_vec(&provenance).unwrap();
        assert_eq!(
            serde_json::from_slice::<SystemPromptProvenance>(&encoded).unwrap(),
            provenance
        );
    }
}

#[test]
fn legacy_prompt_inference_only_treats_the_model_template_as_generated() {
    assert_eq!(
        infer_system_prompt_provenance("model template\n", "model template", "grok-4"),
        SystemPromptProvenance::Model {
            model_id: "grok-4".to_string()
        }
    );
    assert_eq!(
        infer_system_prompt_provenance("client override", "model template", "grok-4"),
        SystemPromptProvenance::Custom
    );
}
