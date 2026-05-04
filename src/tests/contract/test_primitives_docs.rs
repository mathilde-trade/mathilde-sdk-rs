use crate::generated::primitives::{ProcessorFamily, ProcessorGroup};
use crate::systems::primitives::DocsRegistryRequest;
use crate::systems::primitives::types::{selector_family_names, selector_group_names};

fn csv_param(values: Vec<String>) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        Some(values.join(","))
    }
}

#[test]
fn test_docs_registry_request_uses_canonical_csv_selectors() {
    let request = DocsRegistryRequest {
        family: Some(vec![ProcessorFamily::Metadata]),
        group: Some(vec![ProcessorGroup::Ema]),
    };

    let family = csv_param(selector_family_names(request.family.as_deref()));
    let group = csv_param(selector_group_names(request.group.as_deref()));

    assert_eq!(family.as_deref(), Some("metadata"));
    assert_eq!(group.as_deref(), Some("ema"));
}
