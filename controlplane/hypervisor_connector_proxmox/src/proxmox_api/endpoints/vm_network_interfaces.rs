use serde::Deserialize;

use crate::proxmox_api::api_response::{ApiResponse, ApiResponseExt};

/// Retrieves network interface information for a Proxmox VM via the QEMU guest agent.
///
/// # Arguments
/// * `api_url` - Base URL of the Proxmox API
/// * `client` - HTTP client for making requests
/// * `authorization` - Authorization header value
/// * `node_id` - Proxmox node identifier
/// * `vm_id` - VM identifier
///
/// # Returns
/// Returns the network interfaces data on success, or a Problem on failure.
///
/// # Errors
/// May return `Problem::MissingAgent` if QEMU guest agent is not configured,
/// or `Problem::VMNotRunning` if the VM is not currently running.
pub async fn vm_network_interfaces(
    api_url: &str,
    client: &reqwest::Client,
    authorization: &str,
    node_id: &str,
    vm_id: u32,
) -> Result<ApiResponse<NetworkInterfaces>, crate::proxmox_api::Problem> {
    client
        .get(format!(
            "{}/api2/json/nodes/{}/qemu/{}/agent/network-get-interfaces",
            api_url, node_id, vm_id
        ))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .to_api_response()
        .await
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct NetworkInterfaces {
    pub result: Vec<NetworkInterface>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    #[serde(rename = "ip-addresses")]
    pub ip_addresses: Option<Vec<IPAddress>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct IPAddress {
    #[serde(rename = "ip-address")]
    pub ip_address: String,
    #[serde(rename = "ip-address-type")]
    pub ip_address_type: IpAddressType,
    pub prefix: u8,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum IpAddressType {
    #[serde(rename = "ipv4")]
    Ipv4,
    #[serde(rename = "ipv6")]
    Ipv6,
}

#[cfg(feature = "mock")]
pub mod mock {
    use mock_server::MockServer;

    pub trait WithVMNetworkInterfaces {
        fn test_vm_network_interfaces(self) -> Self;
    }

    impl WithVMNetworkInterfaces for MockServer {
        fn test_vm_network_interfaces(mut self) -> Self {
            let mock = self
                .server
                .mock(
                    "GET",
                    mockito::Matcher::Regex(
                        r"^/api2/json/nodes/.*/qemu/\d+/agent/network-get-interfaces$".to_string(),
                    ),
                )
                .with_body(r#"{"data":{"result":[{"statistics":{"rx-errs":0,"tx-packets":521,"rx-bytes":49442,"tx-dropped":0,"rx-packets":521,"tx-bytes":49442,"rx-dropped":0,"tx-errs":0},"ip-addresses":[{"prefix":8,"ip-address-type":"ipv4","ip-address":"127.0.0.1"},{"prefix":128,"ip-address":"::1","ip-address-type":"ipv6"}],"hardware-address":"00:00:00:00:00:00","name":"lo"},{"hardware-address":"bc:24:11:a7:85:c8","name":"enp6s18"},{"ip-addresses":[{"ip-address-type":"ipv4","ip-address":"10.2.16.69","prefix":21},{"ip-address":"fe80::be24:11ff:fea7:85c8","ip-address-type":"ipv6","prefix":64}],"statistics":{"tx-errs":0,"tx-bytes":127022,"rx-dropped":0,"rx-packets":625,"tx-dropped":0,"rx-bytes":255388,"tx-packets":667,"rx-errs":0},"name":"vmbr0","hardware-address":"bc:24:11:a7:85:c8"},{"statistics":{"tx-errs":0,"tx-bytes":2552,"rx-dropped":0,"rx-packets":16,"tx-dropped":0,"rx-bytes":9612,"tx-packets":25,"rx-errs":0},"ip-addresses":[{"ip-address":"100.96.0.72","ip-address-type":"ipv4","prefix":32},{"prefix":128,"ip-address":"2606:4700:cf1:1000::1a","ip-address-type":"ipv6"},{"ip-address-type":"ipv6","ip-address":"fe80::5853:18a3:9927:7fcc","prefix":64}],"hardware-address":"00:00:00:00:00:00","name":"CloudflareWARP"}]}}"#)
                .create();
            self.mocks.push(mock);
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::WithVMNetworkInterfaces;
    use super::*;
    use mock_server::MockServer;

    #[tokio::test]
    async fn test_vm_network_interfaces() {
        // Arrange a client and the mock server
        let client = reqwest::Client::new();
        let server = MockServer::new().await.test_vm_network_interfaces();

        // Act the call to the function
        let result = vm_network_interfaces(&server.url(), &client, "", "pve-node1", 100).await;

        // Assert the result
        println!("result: {:#?}", &result);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            ApiResponse {
                data: NetworkInterfaces {
                    result: vec![
                        NetworkInterface {
                            name: String::from("lo"),
                            ip_addresses: Some(vec![
                                IPAddress {
                                    ip_address: String::from("127.0.0.1"),
                                    ip_address_type: IpAddressType::Ipv4,
                                    prefix: 8,
                                },
                                IPAddress {
                                    ip_address: String::from("::1"),
                                    ip_address_type: IpAddressType::Ipv6,
                                    prefix: 128,
                                },
                            ],),
                        },
                        NetworkInterface {
                            name: String::from("enp6s18"),
                            ip_addresses: None,
                        },
                        NetworkInterface {
                            name: String::from("vmbr0"),
                            ip_addresses: Some(vec![
                                IPAddress {
                                    ip_address: String::from("10.2.16.69"),
                                    ip_address_type: IpAddressType::Ipv4,
                                    prefix: 21,
                                },
                                IPAddress {
                                    ip_address: String::from("fe80::be24:11ff:fea7:85c8"),
                                    ip_address_type: IpAddressType::Ipv6,
                                    prefix: 64,
                                },
                            ],),
                        },
                        NetworkInterface {
                            name: String::from("CloudflareWARP"),
                            ip_addresses: Some(vec![
                                IPAddress {
                                    ip_address: String::from("100.96.0.72"),
                                    ip_address_type: IpAddressType::Ipv4,
                                    prefix: 32,
                                },
                                IPAddress {
                                    ip_address: String::from("2606:4700:cf1:1000::1a"),
                                    ip_address_type: IpAddressType::Ipv6,
                                    prefix: 128,
                                },
                                IPAddress {
                                    ip_address: String::from("fe80::5853:18a3:9927:7fcc"),
                                    ip_address_type: IpAddressType::Ipv6,
                                    prefix: 64,
                                },
                            ],),
                        },
                    ],
                },
            },
        );
    }
}
