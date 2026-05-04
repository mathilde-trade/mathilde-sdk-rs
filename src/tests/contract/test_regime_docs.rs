use crate::generated::regime::{ProcessorFamily, ProcessorGroup};
use crate::systems::regime::DocsRegistryRequest;
use crate::systems::regime::types::{selector_family_names, selector_group_names};

fn csv_param(values: Vec<String>) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        Some(values.join(","))
    }
}

#[test]
fn test_regime_docs_registry_request_uses_canonical_csv_selectors() {
    let request = DocsRegistryRequest {
        family: Some(vec![ProcessorFamily::Metadata]),
        group: Some(vec![ProcessorGroup::TrendQ1]),
    };

    let family = csv_param(selector_family_names(request.family.as_deref()));
    let group = csv_param(selector_group_names(request.group.as_deref()));

    assert_eq!(family.as_deref(), Some("metadata"));
    assert_eq!(group.as_deref(), Some("trend.q1"));
}
