//! VM Firewall rules endpoint for Proxmox VE.
//!
//! Sets firewall rules for a VM.

use crate::proxmox::api::Error;
use crate::proxmox::api::api_response::{ApiResponse, ApiResponseExt};
use serde::Serialize;
use serde_with::skip_serializing_none;

/// Adds a firewall rule to a VM.
///
/// API: POST /nodes/{node}/qemu/{vmid}/firewall/rules
pub async fn firewall_rule_create(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node: &str,
    vmid: u32,
    rule: &FirewallRule,
) -> Result<ApiResponse<Option<String>>, Error> {
    client
        .post(format!(
            "{}/api2/json/nodes/{}/qemu/{}/firewall/rules",
            api_url, node, vmid
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(rule)
        .send()
        .await
        .to_api_response()
        .await
}

/// Deletes a firewall rule from a VM.
///
/// API: DELETE /nodes/{node}/qemu/{vmid}/firewall/rules/{pos}
pub async fn firewall_rule_delete(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node: &str,
    vmid: u32,
    position: u32,
) -> Result<ApiResponse<Option<String>>, Error> {
    client
        .delete(format!(
            "{}/api2/json/nodes/{}/qemu/{}/firewall/rules/{}",
            api_url, node, vmid, position
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

/// Enables or disables the firewall for a VM.
///
/// API: PUT /nodes/{node}/qemu/{vmid}/firewall/options
pub async fn firewall_enable(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node: &str,
    vmid: u32,
    enable: bool,
) -> Result<ApiResponse<Option<String>>, Error> {
    #[derive(Serialize)]
    struct FirewallOptions {
        enable: u8,
    }

    client
        .put(format!(
            "{}/api2/json/nodes/{}/qemu/{}/firewall/options",
            api_url, node, vmid
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .json(&FirewallOptions {
            enable: if enable { 1 } else { 0 },
        })
        .send()
        .await
        .to_api_response()
        .await
}

/// A firewall rule configuration.
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct FirewallRule {
    /// Rule type: "in", "out", or "group"
    #[serde(rename = "type")]
    pub rule_type: String,

    /// Action: "ACCEPT", "DROP", or "REJECT"
    pub action: String,

    /// Protocol: tcp, udp, icmp, or empty for all
    pub proto: Option<String>,

    /// Destination port or port range (e.g., "80" or "8000:8080")
    pub dport: Option<String>,

    /// Source port or port range
    pub sport: Option<String>,

    /// Source address in CIDR notation
    pub source: Option<String>,

    /// Destination address in CIDR notation
    pub dest: Option<String>,

    /// Enable/disable rule (1 = enabled)
    pub enable: Option<u8>,

    /// Comment/description
    pub comment: Option<String>,

    /// Position in rule list (lower = higher priority)
    pub pos: Option<u32>,

    /// Log level: "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug", "nolog"
    pub log: Option<String>,

    /// Interface name (e.g., "net0")
    pub iface: Option<String>,

    /// ICMP type (for ICMP protocol)
    #[serde(rename = "icmp-type")]
    pub icmp_type: Option<String>,

    /// Macro name (e.g., "SSH", "HTTP", "HTTPS")
    #[serde(rename = "macro")]
    pub macro_name: Option<String>,
}

impl FirewallRule {
    /// Creates an ACCEPT rule for inbound traffic.
    pub fn accept_in(proto: Option<&str>, dport: Option<&str>, source: Option<&str>) -> Self {
        Self {
            rule_type: "in".to_string(),
            action: "ACCEPT".to_string(),
            proto: proto.map(String::from),
            dport: dport.map(String::from),
            sport: None,
            source: source.map(String::from),
            dest: None,
            enable: Some(1),
            comment: None,
            pos: None,
            log: None,
            iface: None,
            icmp_type: None,
            macro_name: None,
        }
    }

    /// Creates a DROP rule for inbound traffic.
    pub fn drop_in(source: Option<&str>) -> Self {
        Self {
            rule_type: "in".to_string(),
            action: "DROP".to_string(),
            proto: None,
            dport: None,
            sport: None,
            source: source.map(String::from),
            dest: None,
            enable: Some(1),
            comment: None,
            pos: None,
            log: None,
            iface: None,
            icmp_type: None,
            macro_name: None,
        }
    }

    /// Creates an ACCEPT rule for outbound traffic.
    pub fn accept_out(proto: Option<&str>, dport: Option<&str>, dest: Option<&str>) -> Self {
        Self {
            rule_type: "out".to_string(),
            action: "ACCEPT".to_string(),
            proto: proto.map(String::from),
            dport: dport.map(String::from),
            sport: None,
            source: None,
            dest: dest.map(String::from),
            enable: Some(1),
            comment: None,
            pos: None,
            log: None,
            iface: None,
            icmp_type: None,
            macro_name: None,
        }
    }

    /// Creates a DROP rule for outbound traffic.
    pub fn drop_out(dest: Option<&str>) -> Self {
        Self {
            rule_type: "out".to_string(),
            action: "DROP".to_string(),
            proto: None,
            dport: None,
            sport: None,
            source: None,
            dest: dest.map(String::from),
            enable: Some(1),
            comment: None,
            pos: None,
            log: None,
            iface: None,
            icmp_type: None,
            macro_name: None,
        }
    }

    /// Sets a comment for the rule.
    pub fn with_comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }

    /// Sets the position of the rule.
    pub fn at_position(mut self, pos: u32) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Sets the interface for the rule.
    pub fn on_interface(mut self, iface: &str) -> Self {
        self.iface = Some(iface.to_string());
        self
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithFirewallRulesMock {
        fn with_firewall_rule_create(self) -> Self;
        fn with_firewall_rule_delete(self) -> Self;
        fn with_firewall_enable(self) -> Self;
    }

    impl WithFirewallRulesMock for MockServer {
        fn with_firewall_rule_create(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "POST",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.+/qemu/\d+/firewall/rules$".to_string(),
                    ),
                )
                .with_body(r#"{"data":null}"#)
                .create();
            self.mocks.push(mock);
            self
        }

        fn with_firewall_rule_delete(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "DELETE",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.+/qemu/\d+/firewall/rules/\d+$".to_string(),
                    ),
                )
                .with_body(r#"{"data":null}"#)
                .create();
            self.mocks.push(mock);
            self
        }

        fn with_firewall_enable(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "PUT",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.+/qemu/\d+/firewall/options$".to_string(),
                    ),
                )
                .with_body(r#"{"data":null}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithFirewallRulesMock;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_firewall_rule_create() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_firewall_rule_create();
        let rule = FirewallRule::accept_in(Some("tcp"), Some("22"), None).with_comment("Allow SSH");
        let result =
            firewall_rule_create(&server.url(), &client, "", "pve-node1", 100, &rule).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_firewall_rule_delete() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_firewall_rule_delete();
        let result = firewall_rule_delete(&server.url(), &client, "", "pve-node1", 100, 0).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_firewall_enable() {
        let client = reqwest::Client::new();
        let server = MockServer::new().await.with_firewall_enable();
        let result = firewall_enable(&server.url(), &client, "", "pve-node1", 100, true).await;

        assert!(result.is_ok());
    }
}
