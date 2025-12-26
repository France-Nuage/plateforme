//! Network gRPC service implementations.
//!
//! Provides VPCs, VNets, IPAM, and SecurityGroups gRPC services for the
//! France Nuage platform's VPC-based SDN architecture.

use std::time::SystemTime;

use crate::error::Error;
use frn_core::authorization::Authorize;
use frn_core::identity::IAM;
use frn_core::network::{
    self as network_core, Action as CoreAction, AllocateIPRequest,
    AllocationType as CoreAllocationType, Direction as CoreDirection, Protocol as CoreProtocol,
    ReserveIPRequest, SecurityGroupCreateRequest, SecurityGroupUpdateRequest,
    SecurityRuleCreateRequest, VNetCreateRequest, VNetUpdateRequest, VPCCreateRequest,
    VPCUpdateRequest,
};
use sqlx::types::Uuid;
use tonic::{Request, Response, Status};

tonic::include_proto!("francenuage.fr.v1.network");

// ============================================================================
// VPCs Service
// ============================================================================

/// Converts core VPC model to proto VPC message.
impl From<network_core::VPC> for Vpc {
    fn from(value: network_core::VPC) -> Self {
        Vpc {
            id: value.id.to_string(),
            name: value.name,
            slug: value.slug,
            organization_id: value.organization_id.to_string(),
            region: value.region,
            sdn_zone_id: value.sdn_zone_id,
            vxlan_tag: value.vxlan_tag,
            state: VpcState::from(value.state) as i32,
            mtu: value.mtu,
            created_at: Some(SystemTime::from(value.created_at).into()),
            updated_at: Some(SystemTime::from(value.updated_at).into()),
        }
    }
}

/// Converts core VPCState to proto VPCState.
impl From<network_core::VPCState> for VpcState {
    fn from(value: network_core::VPCState) -> Self {
        match value {
            network_core::VPCState::Pending => VpcState::Pending,
            network_core::VPCState::Creating => VpcState::Creating,
            network_core::VPCState::Active => VpcState::Active,
            network_core::VPCState::Error => VpcState::Error,
            network_core::VPCState::Deleting => VpcState::Deleting,
        }
    }
}

/// VPCs gRPC service implementation.
#[derive(Clone)]
pub struct VPCs<A: Authorize> {
    iam: IAM,
    service: network_core::VPCs<A>,
}

impl<A: Authorize> VPCs<A> {
    /// Creates a new VPCs gRPC service.
    pub fn new(iam: IAM, service: network_core::VPCs<A>) -> Self {
        Self { iam, service }
    }
}

#[tonic::async_trait]
impl<A: Authorize + 'static> vp_cs_server::VpCs for VPCs<A> {
    async fn create(&self, request: Request<CreateVpcRequest>) -> Result<Response<Vpc>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let vpc = self
            .service
            .clone()
            .create(
                &principal,
                VPCCreateRequest {
                    name: inner.name,
                    slug: inner.slug,
                    organization_id: Uuid::parse_str(&inner.organization_id)
                        .map_err(|_| Error::MalformedId(inner.organization_id))?,
                    region: if inner.region.is_empty() {
                        None
                    } else {
                        Some(inner.region)
                    },
                    mtu: if inner.mtu == 0 {
                        None
                    } else {
                        Some(inner.mtu)
                    },
                },
            )
            .await?;

        Ok(Response::new(vpc.into()))
    }

    async fn get(&self, request: Request<GetVpcRequest>) -> Result<Response<Vpc>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        let vpc = self.service.clone().get(&principal, id).await?;

        Ok(Response::new(vpc.into()))
    }

    async fn list(
        &self,
        request: Request<ListVpCsRequest>,
    ) -> Result<Response<ListVpCsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let organization_id = if inner.organization_id.is_empty() {
            None
        } else {
            Some(
                Uuid::parse_str(&inner.organization_id)
                    .map_err(|_| Error::MalformedId(inner.organization_id))?,
            )
        };

        let vpcs = self
            .service
            .clone()
            .list(&principal, organization_id)
            .await?;

        Ok(Response::new(ListVpCsResponse {
            vpcs: vpcs.into_iter().map(Into::into).collect(),
            next_page_token: String::new(),
        }))
    }

    async fn update(&self, request: Request<UpdateVpcRequest>) -> Result<Response<Vpc>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        let vpc = self
            .service
            .clone()
            .update(
                &principal,
                VPCUpdateRequest {
                    id,
                    name: inner.name,
                    mtu: inner.mtu,
                },
            )
            .await?;

        Ok(Response::new(vpc.into()))
    }

    async fn delete(
        &self,
        request: Request<DeleteVpcRequest>,
    ) -> Result<Response<DeleteVpcResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        self.service.clone().delete(&principal, id).await?;

        Ok(Response::new(DeleteVpcResponse {}))
    }
}

// ============================================================================
// VNets Service
// ============================================================================

/// Converts core VNet model to proto VNet message.
impl From<network_core::VNet> for VNet {
    fn from(value: network_core::VNet) -> Self {
        VNet {
            id: value.id.to_string(),
            vpc_id: value.vpc_id.to_string(),
            name: value.name,
            vnet_bridge_id: value.vnet_bridge_id,
            subnet: value.subnet,
            gateway: value.gateway,
            dhcp_enabled: value.dhcp_enabled,
            dns_servers: value.dns_servers,
            state: VNetState::from(value.state) as i32,
            created_at: Some(SystemTime::from(value.created_at).into()),
            updated_at: Some(SystemTime::from(value.updated_at).into()),
        }
    }
}

/// Converts core VNetState to proto VNetState.
impl From<network_core::VNetState> for VNetState {
    fn from(value: network_core::VNetState) -> Self {
        match value {
            network_core::VNetState::Pending => VNetState::VnetStatePending,
            network_core::VNetState::Active => VNetState::VnetStateActive,
            network_core::VNetState::Error => VNetState::VnetStateError,
        }
    }
}

/// VNets gRPC service implementation.
#[derive(Clone)]
pub struct VNets<A: Authorize> {
    iam: IAM,
    service: network_core::VNets<A>,
}

impl<A: Authorize> VNets<A> {
    /// Creates a new VNets gRPC service.
    pub fn new(iam: IAM, service: network_core::VNets<A>) -> Self {
        Self { iam, service }
    }
}

#[tonic::async_trait]
impl<A: Authorize + 'static> v_nets_server::VNets for VNets<A> {
    async fn create(&self, request: Request<CreateVNetRequest>) -> Result<Response<VNet>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let vnet = self
            .service
            .clone()
            .create(
                &principal,
                VNetCreateRequest {
                    vpc_id: Uuid::parse_str(&inner.vpc_id)
                        .map_err(|_| Error::MalformedId(inner.vpc_id))?,
                    name: inner.name,
                    subnet: inner.subnet,
                    gateway: if inner.gateway.is_empty() {
                        None
                    } else {
                        Some(inner.gateway)
                    },
                    dns_servers: if inner.dns_servers.is_empty() {
                        None
                    } else {
                        Some(inner.dns_servers)
                    },
                },
            )
            .await?;

        Ok(Response::new(vnet.into()))
    }

    async fn get(&self, request: Request<GetVNetRequest>) -> Result<Response<VNet>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        let vnet = self.service.clone().get(&principal, id).await?;

        Ok(Response::new(vnet.into()))
    }

    async fn list(
        &self,
        request: Request<ListVNetsRequest>,
    ) -> Result<Response<ListVNetsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let vpc_id =
            Uuid::parse_str(&inner.vpc_id).map_err(|_| Error::MalformedId(inner.vpc_id))?;

        let vnets = self.service.clone().list(&principal, vpc_id).await?;

        Ok(Response::new(ListVNetsResponse {
            vnets: vnets.into_iter().map(Into::into).collect(),
            next_page_token: String::new(),
        }))
    }

    async fn update(&self, request: Request<UpdateVNetRequest>) -> Result<Response<VNet>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        let vnet = self
            .service
            .clone()
            .update(
                &principal,
                VNetUpdateRequest {
                    id,
                    name: inner.name,
                    dns_servers: inner.dns_servers,
                },
            )
            .await?;

        Ok(Response::new(vnet.into()))
    }

    async fn delete(
        &self,
        request: Request<DeleteVNetRequest>,
    ) -> Result<Response<DeleteVNetResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        self.service.clone().delete(&principal, id).await?;

        Ok(Response::new(DeleteVNetResponse {}))
    }
}

// ============================================================================
// IPAM Service
// ============================================================================

/// Converts core IPAllocation model to proto IPAllocation message.
impl From<network_core::IPAllocation> for IpAllocation {
    fn from(value: network_core::IPAllocation) -> Self {
        IpAllocation {
            id: value.id.to_string(),
            vnet_id: value.vnet_id.to_string(),
            address: value.address,
            mac_address: value.mac_address.unwrap_or_default(),
            instance_interface_id: value
                .instance_interface_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            allocation_type: allocation_type_to_proto(CoreAllocationType::from(
                value.allocation_type,
            )) as i32,
            hostname: value.hostname.unwrap_or_default(),
            allocated_at: value.allocated_at.map(|dt| SystemTime::from(dt).into()),
            created_at: Some(SystemTime::from(value.created_at).into()),
        }
    }
}

/// Converts core AllocationType to proto AllocationType.
fn allocation_type_to_proto(value: CoreAllocationType) -> AllocationType {
    match value {
        CoreAllocationType::Static => AllocationType::Static,
        CoreAllocationType::Dynamic => AllocationType::Dynamic,
        CoreAllocationType::Reserved => AllocationType::Reserved,
        CoreAllocationType::Gateway => AllocationType::Gateway,
    }
}

/// Converts proto AllocationType to core AllocationType.
fn allocation_type_from_proto(value: i32) -> Option<CoreAllocationType> {
    match AllocationType::try_from(value) {
        Ok(AllocationType::Static) => Some(CoreAllocationType::Static),
        Ok(AllocationType::Dynamic) => Some(CoreAllocationType::Dynamic),
        Ok(AllocationType::Reserved) => Some(CoreAllocationType::Reserved),
        Ok(AllocationType::Gateway) => Some(CoreAllocationType::Gateway),
        _ => None,
    }
}

/// IPAM gRPC service implementation.
#[derive(Clone)]
pub struct Ipam<A: Authorize> {
    iam: IAM,
    service: network_core::IPAM<A>,
}

impl<A: Authorize> Ipam<A> {
    /// Creates a new IPAM gRPC service.
    pub fn new(iam: IAM, service: network_core::IPAM<A>) -> Self {
        Self { iam, service }
    }
}

#[tonic::async_trait]
impl<A: Authorize + 'static> ipam_server::Ipam for Ipam<A> {
    async fn allocate_ip(
        &self,
        request: Request<AllocateIpRequest>,
    ) -> Result<Response<IpAllocation>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let allocation = self
            .service
            .clone()
            .allocate(
                &principal,
                AllocateIPRequest {
                    vnet_id: Uuid::parse_str(&inner.vnet_id)
                        .map_err(|_| Error::MalformedId(inner.vnet_id))?,
                    requested_ip: inner.requested_ip,
                    hostname: if inner.hostname.is_empty() {
                        None
                    } else {
                        Some(inner.hostname)
                    },
                },
            )
            .await?;

        Ok(Response::new(allocation.into()))
    }

    async fn release_ip(
        &self,
        request: Request<ReleaseIpRequest>,
    ) -> Result<Response<ReleaseIpResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let allocation_id = Uuid::parse_str(&inner.allocation_id)
            .map_err(|_| Error::MalformedId(inner.allocation_id))?;

        let released_address = self
            .service
            .clone()
            .release(&principal, allocation_id)
            .await?;

        Ok(Response::new(ReleaseIpResponse { released_address }))
    }

    async fn reserve_ip(
        &self,
        request: Request<ReserveIpRequest>,
    ) -> Result<Response<IpAllocation>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let allocation = self
            .service
            .clone()
            .reserve(
                &principal,
                ReserveIPRequest {
                    vnet_id: Uuid::parse_str(&inner.vnet_id)
                        .map_err(|_| Error::MalformedId(inner.vnet_id))?,
                    address: inner.address,
                    reason: if inner.reason.is_empty() {
                        None
                    } else {
                        Some(inner.reason)
                    },
                },
            )
            .await?;

        Ok(Response::new(allocation.into()))
    }

    async fn list_ip_allocations(
        &self,
        request: Request<ListIpAllocationsRequest>,
    ) -> Result<Response<ListIpAllocationsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let vnet_id =
            Uuid::parse_str(&inner.vnet_id).map_err(|_| Error::MalformedId(inner.vnet_id))?;

        let filter_type = inner.filter_type.and_then(allocation_type_from_proto);

        let (allocations, stats) = self
            .service
            .clone()
            .list_allocations(&principal, vnet_id, filter_type)
            .await?;

        Ok(Response::new(ListIpAllocationsResponse {
            allocations: allocations.into_iter().map(Into::into).collect(),
            next_page_token: String::new(),
            total_count: stats.total_count,
            available_count: stats.available_count,
        }))
    }

    async fn generate_mac(
        &self,
        request: Request<GenerateMacRequest>,
    ) -> Result<Response<GenerateMacResponse>, Status> {
        let principal = self.iam.principal(&request).await?;

        let mac_address = self.service.clone().generate_mac(&principal).await?;

        Ok(Response::new(GenerateMacResponse { mac_address }))
    }
}

// ============================================================================
// SecurityGroups Service
// ============================================================================

/// Converts core SecurityGroup model to proto SecurityGroup message.
impl From<(network_core::SecurityGroup, Vec<network_core::SecurityRule>)> for SecurityGroup {
    fn from((sg, rules): (network_core::SecurityGroup, Vec<network_core::SecurityRule>)) -> Self {
        SecurityGroup {
            id: sg.id.to_string(),
            vpc_id: sg.vpc_id.to_string(),
            name: sg.name,
            description: sg.description.unwrap_or_default(),
            is_default: sg.is_default,
            rules: rules.into_iter().map(Into::into).collect(),
            created_at: Some(SystemTime::from(sg.created_at).into()),
            updated_at: Some(SystemTime::from(sg.updated_at).into()),
        }
    }
}

/// Converts core SecurityGroup model to proto SecurityGroup message (without rules).
fn security_group_without_rules(sg: network_core::SecurityGroup) -> SecurityGroup {
    SecurityGroup {
        id: sg.id.to_string(),
        vpc_id: sg.vpc_id.to_string(),
        name: sg.name,
        description: sg.description.unwrap_or_default(),
        is_default: sg.is_default,
        rules: Vec::new(),
        created_at: Some(SystemTime::from(sg.created_at).into()),
        updated_at: Some(SystemTime::from(sg.updated_at).into()),
    }
}

/// Converts core SecurityRule model to proto SecurityRule message.
impl From<network_core::SecurityRule> for SecurityRule {
    fn from(value: network_core::SecurityRule) -> Self {
        SecurityRule {
            id: value.id.to_string(),
            direction: direction_to_proto(value.direction) as i32,
            protocol: protocol_to_proto(value.protocol) as i32,
            port_from: value.port_from,
            port_to: value.port_to,
            source_cidr: value.source_cidr,
            action: action_to_proto(value.action) as i32,
            priority: value.priority,
            description: value.description.unwrap_or_default(),
        }
    }
}

/// Converts core Direction to proto Direction.
fn direction_to_proto(value: CoreDirection) -> Direction {
    match value {
        CoreDirection::Inbound => Direction::Inbound,
        CoreDirection::Outbound => Direction::Outbound,
    }
}

/// Converts proto Direction to core Direction.
fn direction_from_proto(value: i32) -> CoreDirection {
    match Direction::try_from(value) {
        Ok(Direction::Inbound) => CoreDirection::Inbound,
        Ok(Direction::Outbound) => CoreDirection::Outbound,
        _ => CoreDirection::Inbound,
    }
}

/// Converts core Protocol to proto Protocol.
fn protocol_to_proto(value: CoreProtocol) -> Protocol {
    match value {
        CoreProtocol::Tcp => Protocol::Tcp,
        CoreProtocol::Udp => Protocol::Udp,
        CoreProtocol::Icmp => Protocol::Icmp,
        CoreProtocol::All => Protocol::All,
    }
}

/// Converts proto Protocol to core Protocol.
fn protocol_from_proto(value: i32) -> CoreProtocol {
    match Protocol::try_from(value) {
        Ok(Protocol::Tcp) => CoreProtocol::Tcp,
        Ok(Protocol::Udp) => CoreProtocol::Udp,
        Ok(Protocol::Icmp) => CoreProtocol::Icmp,
        Ok(Protocol::All) => CoreProtocol::All,
        _ => CoreProtocol::All,
    }
}

/// Converts core Action to proto Action.
fn action_to_proto(value: CoreAction) -> Action {
    match value {
        CoreAction::Allow => Action::Allow,
        CoreAction::Deny => Action::Deny,
    }
}

/// Converts proto Action to core Action.
fn action_from_proto(value: i32) -> CoreAction {
    match Action::try_from(value) {
        Ok(Action::Allow) => CoreAction::Allow,
        Ok(Action::Deny) => CoreAction::Deny,
        _ => CoreAction::Deny,
    }
}

/// SecurityGroups gRPC service implementation.
#[derive(Clone)]
pub struct SecurityGroups<A: Authorize> {
    iam: IAM,
    service: network_core::SecurityGroups<A>,
}

impl<A: Authorize> SecurityGroups<A> {
    /// Creates a new SecurityGroups gRPC service.
    pub fn new(iam: IAM, service: network_core::SecurityGroups<A>) -> Self {
        Self { iam, service }
    }
}

#[tonic::async_trait]
impl<A: Authorize + 'static> security_groups_server::SecurityGroups for SecurityGroups<A> {
    async fn create(
        &self,
        request: Request<CreateSecurityGroupRequest>,
    ) -> Result<Response<SecurityGroup>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let sg = self
            .service
            .clone()
            .create(
                &principal,
                SecurityGroupCreateRequest {
                    vpc_id: Uuid::parse_str(&inner.vpc_id)
                        .map_err(|_| Error::MalformedId(inner.vpc_id))?,
                    name: inner.name,
                    description: if inner.description.is_empty() {
                        None
                    } else {
                        Some(inner.description)
                    },
                },
            )
            .await?;

        Ok(Response::new(security_group_without_rules(sg)))
    }

    async fn get(
        &self,
        request: Request<GetSecurityGroupRequest>,
    ) -> Result<Response<SecurityGroup>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        let (sg, rules) = self.service.clone().get(&principal, id).await?;

        Ok(Response::new((sg, rules).into()))
    }

    async fn list(
        &self,
        request: Request<ListSecurityGroupsRequest>,
    ) -> Result<Response<ListSecurityGroupsResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let vpc_id =
            Uuid::parse_str(&inner.vpc_id).map_err(|_| Error::MalformedId(inner.vpc_id))?;

        let sgs = self.service.clone().list(&principal, vpc_id).await?;

        Ok(Response::new(ListSecurityGroupsResponse {
            security_groups: sgs.into_iter().map(security_group_without_rules).collect(),
            next_page_token: String::new(),
        }))
    }

    async fn update(
        &self,
        request: Request<UpdateSecurityGroupRequest>,
    ) -> Result<Response<SecurityGroup>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        let sg = self
            .service
            .clone()
            .update(
                &principal,
                SecurityGroupUpdateRequest {
                    id,
                    name: inner.name,
                    description: inner.description,
                },
            )
            .await?;

        Ok(Response::new(security_group_without_rules(sg)))
    }

    async fn delete(
        &self,
        request: Request<DeleteSecurityGroupRequest>,
    ) -> Result<Response<DeleteSecurityGroupResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let id = Uuid::parse_str(&inner.id).map_err(|_| Error::MalformedId(inner.id))?;

        self.service.clone().delete(&principal, id).await?;

        Ok(Response::new(DeleteSecurityGroupResponse {}))
    }

    async fn add_rule(
        &self,
        request: Request<AddRuleRequest>,
    ) -> Result<Response<SecurityRule>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let rule = self
            .service
            .clone()
            .add_rule(
                &principal,
                SecurityRuleCreateRequest {
                    security_group_id: Uuid::parse_str(&inner.security_group_id)
                        .map_err(|_| Error::MalformedId(inner.security_group_id))?,
                    direction: direction_from_proto(inner.direction),
                    protocol: protocol_from_proto(inner.protocol),
                    port_from: inner.port_from,
                    port_to: inner.port_to,
                    source_cidr: inner.source_cidr,
                    action: action_from_proto(inner.action),
                    priority: inner.priority,
                    description: if inner.description.is_empty() {
                        None
                    } else {
                        Some(inner.description)
                    },
                },
            )
            .await?;

        Ok(Response::new(rule.into()))
    }

    async fn remove_rule(
        &self,
        request: Request<RemoveRuleRequest>,
    ) -> Result<Response<RemoveRuleResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();
        let rule_id =
            Uuid::parse_str(&inner.rule_id).map_err(|_| Error::MalformedId(inner.rule_id))?;

        self.service
            .clone()
            .remove_rule(&principal, rule_id)
            .await?;

        Ok(Response::new(RemoveRuleResponse {}))
    }

    async fn attach_to_interface(
        &self,
        request: Request<AttachToInterfaceRequest>,
    ) -> Result<Response<AttachToInterfaceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let security_group_id = Uuid::parse_str(&inner.security_group_id)
            .map_err(|_| Error::MalformedId(inner.security_group_id.clone()))?;
        let interface_id = Uuid::parse_str(&inner.instance_interface_id)
            .map_err(|_| Error::MalformedId(inner.instance_interface_id))?;

        self.service
            .clone()
            .attach_to_interface(&principal, security_group_id, interface_id)
            .await?;

        Ok(Response::new(AttachToInterfaceResponse {}))
    }

    async fn detach_from_interface(
        &self,
        request: Request<DetachFromInterfaceRequest>,
    ) -> Result<Response<DetachFromInterfaceResponse>, Status> {
        let principal = self.iam.principal(&request).await?;
        let inner = request.into_inner();

        let security_group_id = Uuid::parse_str(&inner.security_group_id)
            .map_err(|_| Error::MalformedId(inner.security_group_id.clone()))?;
        let interface_id = Uuid::parse_str(&inner.instance_interface_id)
            .map_err(|_| Error::MalformedId(inner.instance_interface_id))?;

        self.service
            .clone()
            .detach_from_interface(&principal, security_group_id, interface_id)
            .await?;

        Ok(Response::new(DetachFromInterfaceResponse {}))
    }
}
