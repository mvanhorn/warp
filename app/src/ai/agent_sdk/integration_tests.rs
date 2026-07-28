use warp_cli::provider::ProviderType;
use warp_graphql::queries::get_simple_integrations::{
    SimpleIntegration, SimpleIntegrationConnectionStatus, SimpleIntegrationsOutput,
};

use super::reconnect_link;

fn integration(
    provider_slug: &str,
    connection_link: &str,
    connection_status: SimpleIntegrationConnectionStatus,
) -> SimpleIntegration {
    SimpleIntegration {
        provider_slug: provider_slug.to_string(),
        description: String::new(),
        connection_link: connection_link.to_string(),
        connection_status,
        integration_config: None,
        created_at: None,
        updated_at: None,
    }
}

fn output(integrations: Vec<SimpleIntegration>) -> anyhow::Result<SimpleIntegrationsOutput> {
    Ok(SimpleIntegrationsOutput {
        integrations,
        message: None,
    })
}

#[test]
fn reconnect_link_is_returned_even_when_cached_status_is_active() {
    let integrations = vec![integration(
        "linear",
        "https://example.com/linear/authorize",
        SimpleIntegrationConnectionStatus::Active,
    )];

    assert_eq!(
        reconnect_link(output(integrations), &ProviderType::Linear).unwrap(),
        "https://example.com/linear/authorize"
    );
}

#[test]
fn reconnect_link_uses_only_the_requested_provider() {
    let integrations = vec![
        integration(
            "linear",
            "https://example.com/linear/authorize",
            SimpleIntegrationConnectionStatus::ConnectionError,
        ),
        integration(
            "slack",
            "https://example.com/slack/authorize",
            SimpleIntegrationConnectionStatus::NotConnected,
        ),
    ];

    assert_eq!(
        reconnect_link(output(integrations), &ProviderType::Slack).unwrap(),
        "https://example.com/slack/authorize"
    );
}

#[test]
fn reconnect_link_errors_when_provider_is_missing() {
    let integrations = vec![integration(
        "slack",
        "https://example.com/slack/authorize",
        SimpleIntegrationConnectionStatus::Active,
    )];

    let err = reconnect_link(output(integrations), &ProviderType::Linear).unwrap_err();

    assert!(
        err.to_string().contains("was not returned by the server"),
        "{err}"
    );
}

#[test]
fn reconnect_link_errors_when_authorization_link_is_empty() {
    let integrations = vec![integration(
        "linear",
        "  ",
        SimpleIntegrationConnectionStatus::Active,
    )];

    let err = reconnect_link(output(integrations), &ProviderType::Linear).unwrap_err();

    assert!(
        err.to_string().contains("empty authorization link"),
        "{err}"
    );
}

#[test]
fn reconnect_link_propagates_list_request_failure() {
    let err = reconnect_link(
        Err(anyhow::anyhow!("request timed out")),
        &ProviderType::Linear,
    )
    .unwrap_err();

    assert!(
        err.to_string()
            .contains("failed to fetch integration details: request timed out"),
        "{err}"
    );
}
