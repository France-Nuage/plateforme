import { BaseSeeder } from '@adonisjs/lucid/seeders'
import { ServiceFactory } from '#database/factories/iam/service_factory'
import { VerbFactory } from '#database/factories/iam/verb_factory'
import { TypeFactory } from '#database/factories/iam/type_factory'
import { PermissionFactory } from '#database/factories/iam/permission_factory'
import { RoleFactory } from '#database/factories/iam/role_factory'

export default class extends BaseSeeder {
  public async run() {
    const verbs = [
      'abort',
      'access',
      'actAs',
      'add',
      'attach',
      'bind',
      'calculate',
      'call',
      'cancel',
      'check',
      'cloneRules',
      'close',
      'connect',
      'consume',
      'create',
      'delete',
      'deploy',
      'destroy',
      'detachSubscription',
      'disable',
      'download',
      'drop',
      'enable',
      'escalate',
      'execute',
      'explain',
      'export',
      'failover',
      'get',
      'group',
      'import',
      'ingest',
      'install',
      'instantiate',
      'instantiateInline',
      'invoke',
      'list',
      'listActive',
      'listAll',
      'login',
      'lookup',
      'manage',
      'mirror',
      'move',
      'patch',
      'pause',
      'publish',
      'purge',
      'quota',
      'read',
      'reopen',
      'report',
      'reportStatus',
      'reset',
      'resetpassword',
      'resize',
      'resolve',
      'restart',
      'restore',
      'resume',
      'review',
      'run',
      'runDiscovery',
      'runtime',
      'sampleRowKeys',
      'search',
      'seek',
      'select',
      'sendCommand',
      'sendVerificationCode',
      'set',
      'setMetadata',
      'setState',
      'setTags',
      'start',
      'stop',
      'subscribe',
      'truncateLog',
      'undelete',
      'undeploy',
      'uninstall',
      'update',
      'validate',
      'validateTrust',
      'verify',
      'view',
      'wait',
      'watch',
      'write',
      'searchPolicyBindings',
      'getIamPolicy',
      'setIamPolicy',
      'updatePolicyBinding',
      'listEffectiveTags',
      'listTagBindings',
      'pscGet',
      'setCommonInstanceMetadata',
      'getRoutePolicy',
      'listBgpRoutes',
      'listRoutePolicies',
      'listAvailableFeatures',
      'useExternalIp',
      'use',
      'useInternal',
      'copyLogEntries',
      'createInternal',
      'deleteInternal',
      'deleteTagBinding',
      'createTagBinding',
      'listIamRoles',
      'exportIamRoles',
      '*',
    ]
    const services = ['*', 'iam', 'logging', 'compute', 'resourcemanager', 'observability']
    const types = [
      { type__id: '*', service__id: 'iam' },
      { type__id: 'operations', service__id: 'iam' },
      { type__id: '*', service__id: 'compute' },
      { type__id: 'acceleratorTypes', service__id: 'compute' },
      { type__id: 'addresses', service__id: 'compute' },
      { type__id: 'autoscalers', service__id: 'compute' },
      { type__id: 'backendBuckets', service__id: 'compute' },
      { type__id: 'backendServices', service__id: 'compute' },
      { type__id: 'diskTypes', service__id: 'compute' },
      { type__id: 'disks', service__id: 'compute' },
      { type__id: 'externalVpnGateways', service__id: 'compute' },
      { type__id: 'firewalls', service__id: 'compute' },
      { type__id: 'forwardingRules', service__id: 'compute' },
      { type__id: 'globalAddresses', service__id: 'compute' },
      { type__id: 'globalForwardingRules', service__id: 'compute' },
      { type__id: 'globalNetworkEndpointGroups', service__id: 'compute' },
      { type__id: 'globalOperations', service__id: 'compute' },
      { type__id: 'healthChecks', service__id: 'compute' },
      { type__id: 'httpHealthChecks', service__id: 'compute' },
      { type__id: 'httpsHealthChecks', service__id: 'compute' },
      { type__id: 'images', service__id: 'compute' },
      { type__id: 'instanceGroupManagers', service__id: 'compute' },
      { type__id: 'instanceGroups', service__id: 'compute' },
      { type__id: 'instanceSettings', service__id: 'compute' },
      { type__id: 'instanceTemplates', service__id: 'compute' },
      { type__id: 'instances', service__id: 'compute' },
      { type__id: 'instantSnapshots', service__id: 'compute' },
      { type__id: 'interconnectAttachments', service__id: 'compute' },
      { type__id: 'interconnectLocations', service__id: 'compute' },
      { type__id: 'interconnectRemoteLocations', service__id: 'compute' },
      { type__id: 'interconnects', service__id: 'compute' },
      { type__id: 'licenseCodes', service__id: 'compute' },
      { type__id: 'licenses', service__id: 'compute' },
      { type__id: 'machineImages', service__id: 'compute' },
      { type__id: 'machineTypes', service__id: 'compute' },
      { type__id: 'multiMig', service__id: 'compute' },
      { type__id: 'networkAttachments', service__id: 'compute' },
      { type__id: 'networkEndpointGroups', service__id: 'compute' },
      { type__id: 'networkProfiles', service__id: 'compute' },
      { type__id: 'networks', service__id: 'compute' },
      { type__id: 'projects', service__id: 'compute' },
      { type__id: 'regionBackendServices', service__id: 'compute' },
      { type__id: 'regionHealthCheckServices', service__id: 'compute' },
      { type__id: 'regionHealthChecks', service__id: 'compute' },
      { type__id: 'regionNetworkEndpointGroups', service__id: 'compute' },
      { type__id: 'regionNotificationEndpoints', service__id: 'compute' },
      { type__id: 'regionOperations', service__id: 'compute' },
      { type__id: 'regionSslCertificates', service__id: 'compute' },
      { type__id: 'regionSslPolicies', service__id: 'compute' },
      { type__id: 'regionTargetHttpProxies', service__id: 'compute' },
      { type__id: 'regionTargetHttpsProxies', service__id: 'compute' },
      { type__id: 'regionTargetTcpProxies', service__id: 'compute' },
      { type__id: 'regionUrlMaps', service__id: 'compute' },
      { type__id: 'regions', service__id: 'compute' },
      { type__id: 'reservations', service__id: 'compute' },
      { type__id: 'resourcePolicies', service__id: 'compute' },
      { type__id: 'routers', service__id: 'compute' },
      { type__id: 'routes', service__id: 'compute' },
      { type__id: 'serviceAttachments', service__id: 'compute' },
      { type__id: 'snapshots', service__id: 'compute' },
      { type__id: 'spotAssistants', service__id: 'compute' },
      { type__id: 'sslCertificates', service__id: 'compute' },
      { type__id: 'sslPolicies', service__id: 'compute' },
      { type__id: 'storagePools', service__id: 'compute' },
      { type__id: 'subnetworks', service__id: 'compute' },
      { type__id: 'targetGrpcProxies', service__id: 'compute' },
      { type__id: 'targetHttpProxies', service__id: 'compute' },
      { type__id: 'targetHttpsProxies', service__id: 'compute' },
      { type__id: 'targetInstances', service__id: 'compute' },
      { type__id: 'targetPools', service__id: 'compute' },
      { type__id: 'targetSslProxies', service__id: 'compute' },
      { type__id: 'targetTcpProxies', service__id: 'compute' },
      { type__id: 'targetVpnGateways', service__id: 'compute' },
      { type__id: 'urlMaps', service__id: 'compute' },
      { type__id: 'vpnGateways', service__id: 'compute' },
      { type__id: 'vpnTunnels', service__id: 'compute' },
      { type__id: 'zoneOperations', service__id: 'compute' },
      { type__id: 'zones', service__id: 'compute' },
      { type__id: '*', service__id: 'resourcemanager' },
      { type__id: 'folders', service__id: 'resourcemanager' },
      { type__id: 'organizations', service__id: 'resourcemanager' },
      { type__id: 'projects', service__id: 'resourcemanager' },
      { type__id: '*', service__id: 'observability' },
      { type__id: 'scopes', service__id: 'observability' },
      { type__id: 'buckets', service__id: 'logging' },
      { type__id: 'exclusions', service__id: 'logging' },
      { type__id: 'fields', service__id: 'logging' },
      { type__id: 'links', service__id: 'logging' },
      { type__id: 'locations', service__id: 'logging' },
      { type__id: 'logEntries', service__id: 'logging' },
      { type__id: 'logMetrics', service__id: 'logging' },
      { type__id: 'logServiceIndexes', service__id: 'logging' },
      { type__id: 'logServices', service__id: 'logging' },
      { type__id: 'logs', service__id: 'logging' },
      { type__id: 'notificationRules', service__id: 'logging' },
      { type__id: 'operations', service__id: 'logging' },
      { type__id: 'privateLogEntries', service__id: 'logging' },
      { type__id: 'queries', service__id: 'logging' },
      { type__id: 'settings', service__id: 'logging' },
      { type__id: 'sinks', service__id: 'logging' },
      { type__id: 'sqlAlerts', service__id: 'logging' },
      { type__id: 'usage', service__id: 'logging' },
      { type__id: 'views', service__id: 'logging' },
      { type__id: 'assets', service__id: 'cloudassets' },
    ]

    await ServiceFactory.merge(services.map((service) => ({ id: service }))).createMany(
      services.length
    )

    await TypeFactory.merge(
      types.map((type) => ({ id: type.type__id, serviceId: type.service__id }))
    ).createMany(types.length)

    await VerbFactory.merge(verbs.map((verb) => ({ id: verb }))).createMany(verbs.length)

    const permissions = [
      { name: 'resourcemanager.organizations.get' },
      { name: 'resourcemanager.organizations.getIamPolicy' },
      { name: 'resourcemanager.organizations.searchPolicyBindings' },
      { name: 'resourcemanager.organizations.setIamPolicy' },
      { name: 'resourcemanager.organizations.updatePolicyBinding' },
      { name: 'resourcemanager.folders.get' },
      { name: 'resourcemanager.folders.list' },
      { name: 'resourcemanager.folders.create' },
      { name: 'resourcemanager.folders.delete' },
      { name: 'resourcemanager.projects.get' },
      { name: 'resourcemanager.projects.list' },
      { name: 'resourcemanager.projects.create' },
      { name: 'resourcemanager.projects.delete' },
      { name: 'resourcemanager.projects.getIamPolicy' },
      { name: 'resourcemanager.projects.getIamPolicy' },
      { name: 'resourcemanager.organizations.getIamPolicy' },
      { name: 'resourcemanager.folders.getIamPolicy' },
      { name: 'cloudasset.assets.listIamRoles' },
      { name: 'cloudasset.assets.exportIamRoles' },
      { name: 'iam.operations.get' },
      { name: 'logging.buckets.copyLogEntries' },
      { name: 'logging.buckets.create' },
      { name: 'logging.buckets.delete' },
      { name: 'logging.buckets.get' },
      { name: 'logging.buckets.listEffectiveTags' },
      { name: 'logging.buckets.list' },
      { name: 'logging.buckets.createTagBinding' },
      { name: 'logging.buckets.deleteTagBinding' },
      { name: 'logging.buckets.listTagBindings' },
      { name: 'logging.buckets.undelete' },
      { name: 'logging.buckets.update' },
      { name: 'logging.exclusions.*' },
      { name: 'logging.fields.access' },
      { name: 'logging.links.*' },
      { name: 'logging.locations.*' },
      { name: 'logging.logEntries.*' },
      { name: 'logging.logMetrics.*' },
      { name: 'logging.logServiceIndexes.list' },
      { name: 'logging.logServices.list' },
      { name: 'logging.logs.*' },
      { name: 'logging.notificationRules.*' },
      { name: 'logging.operations.*' },
      { name: 'logging.privateLogEntries.list' },
      { name: 'logging.queries.*' },
      { name: 'logging.settings.*' },
      { name: 'logging.sinks.*' },
      { name: 'logging.sqlAlerts.*' },
      { name: 'logging.usage.get' },
      { name: 'logging.views.*' },
      { name: 'observability.scopes.get' },
      { name: 'compute.acceleratorTypes.*' },
      { name: 'compute.addresses.createInternal' },
      { name: 'compute.addresses.deleteInternal' },
      { name: 'compute.addresses.get' },
      { name: 'compute.addresses.list' },
      { name: 'compute.addresses.listEffectiveTags' },
      { name: 'compute.addresses.listTagBindings' },
      { name: 'compute.addresses.use' },
      { name: 'compute.addresses.useInternal' },
      { name: 'compute.autoscalers.*' },
      { name: 'compute.backendBuckets.get' },
      { name: 'compute.backendBuckets.list' },
      { name: 'compute.backendBuckets.listEffectiveTags' },
      { name: 'compute.backendBuckets.listTagBindings' },
      { name: 'compute.backendServices.get' },
      { name: 'compute.backendServices.list' },
      { name: 'compute.backendServices.listEffectiveTags' },
      { name: 'compute.backendServices.listTagBindings' },
      { name: 'compute.diskTypes.*' },
      { name: 'compute.disks.*' },
      { name: 'compute.externalVpnGateways.get' },
      { name: 'compute.externalVpnGateways.list' },
      { name: 'compute.externalVpnGateways.listEffectiveTags' },
      { name: 'compute.externalVpnGateways.listTagBindings' },
      { name: 'compute.firewalls.get' },
      { name: 'compute.firewalls.list' },
      { name: 'compute.firewalls.listEffectiveTags' },
      { name: 'compute.firewalls.listTagBindings' },
      { name: 'compute.forwardingRules.get' },
      { name: 'compute.forwardingRules.list' },
      { name: 'compute.forwardingRules.listEffectiveTags' },
      { name: 'compute.forwardingRules.listTagBindings' },
      { name: 'compute.globalAddresses.get' },
      { name: 'compute.globalAddresses.list' },
      { name: 'compute.globalAddresses.listEffectiveTags' },
      { name: 'compute.globalAddresses.listTagBindings' },
      { name: 'compute.globalAddresses.use' },
      { name: 'compute.globalForwardingRules.get' },
      { name: 'compute.globalForwardingRules.list' },
      { name: 'compute.globalForwardingRules.listEffectiveTags' },
      { name: 'compute.globalForwardingRules.listTagBindings' },
      { name: 'compute.globalForwardingRules.pscGet' },
      { name: 'compute.globalNetworkEndpointGroups.*' },
      { name: 'compute.globalOperations.get' },
      { name: 'compute.globalOperations.list' },
      { name: 'compute.healthChecks.get' },
      { name: 'compute.healthChecks.list' },
      { name: 'compute.healthChecks.listEffectiveTags' },
      { name: 'compute.healthChecks.listTagBindings' },
      { name: 'compute.httpHealthChecks.get' },
      { name: 'compute.httpHealthChecks.list' },
      { name: 'compute.httpHealthChecks.listEffectiveTags' },
      { name: 'compute.httpHealthChecks.listTagBindings' },
      { name: 'compute.httpsHealthChecks.get' },
      { name: 'compute.httpsHealthChecks.list' },
      { name: 'compute.httpsHealthChecks.listEffectiveTags' },
      { name: 'compute.httpsHealthChecks.listTagBindings' },
      { name: 'compute.images.*' },
      { name: 'compute.instanceGroupManagers.*' },
      { name: 'compute.instanceGroups.*' },
      { name: 'compute.instanceSettings.*' },
      { name: 'compute.instanceTemplates.*' },
      { name: 'compute.instances.*' },
      { name: 'compute.instantSnapshots.*' },
      { name: 'compute.interconnectAttachments.get' },
      { name: 'compute.interconnectAttachments.list' },
      { name: 'compute.interconnectAttachments.listEffectiveTags' },
      { name: 'compute.interconnectAttachments.listTagBindings' },
      { name: 'compute.interconnectLocations.*' },
      { name: 'compute.interconnectRemoteLocations.*' },
      { name: 'compute.interconnects.get' },
      { name: 'compute.interconnects.list' },
      { name: 'compute.interconnects.listEffectiveTags' },
      { name: 'compute.interconnects.listTagBindings' },
      { name: 'compute.licenseCodes.*' },
      { name: 'compute.licenses.*' },
      { name: 'compute.machineImages.*' },
      { name: 'compute.machineTypes.*' },
      { name: 'compute.multiMig.*' },
      { name: 'compute.networkAttachments.get' },
      { name: 'compute.networkAttachments.list' },
      { name: 'compute.networkAttachments.listEffectiveTags' },
      { name: 'compute.networkAttachments.listTagBindings' },
      { name: 'compute.networkEndpointGroups.*' },
      { name: 'compute.networkProfiles.*' },
      { name: 'compute.networks.get' },
      { name: 'compute.networks.list' },
      { name: 'compute.networks.listEffectiveTags' },
      { name: 'compute.networks.listTagBindings' },
      { name: 'compute.networks.use' },
      { name: 'compute.networks.useExternalIp' },
      { name: 'compute.projects.get' },
      { name: 'compute.projects.setCommonInstanceMetadata' },
      { name: 'compute.regionBackendServices.get' },
      { name: 'compute.regionBackendServices.list' },
      { name: 'compute.regionBackendServices.listEffectiveTags' },
      { name: 'compute.regionBackendServices.listTagBindings' },
      { name: 'compute.regionHealthCheckServices.get' },
      { name: 'compute.regionHealthCheckServices.list' },
      { name: 'compute.regionHealthChecks.get' },
      { name: 'compute.regionHealthChecks.list' },
      { name: 'compute.regionHealthChecks.listEffectiveTags' },
      { name: 'compute.regionHealthChecks.listTagBindings' },
      { name: 'compute.regionNetworkEndpointGroups.*' },
      { name: 'compute.regionNotificationEndpoints.get' },
      { name: 'compute.regionNotificationEndpoints.list' },
      { name: 'compute.regionOperations.get' },
      { name: 'compute.regionOperations.list' },
      { name: 'compute.regionSslCertificates.get' },
      { name: 'compute.regionSslCertificates.list' },
      { name: 'compute.regionSslCertificates.listEffectiveTags' },
      { name: 'compute.regionSslCertificates.listTagBindings' },
      { name: 'compute.regionSslPolicies.get' },
      { name: 'compute.regionSslPolicies.list' },
      { name: 'compute.regionSslPolicies.listAvailableFeatures' },
      { name: 'compute.regionSslPolicies.listEffectiveTags' },
      { name: 'compute.regionSslPolicies.listTagBindings' },
      { name: 'compute.regionTargetHttpProxies.get' },
      { name: 'compute.regionTargetHttpProxies.list' },
      { name: 'compute.regionTargetHttpProxies.listEffectiveTags' },
      { name: 'compute.regionTargetHttpProxies.listTagBindings' },
      { name: 'compute.regionTargetHttpsProxies.get' },
      { name: 'compute.regionTargetHttpsProxies.list' },
      { name: 'compute.regionTargetHttpsProxies.listEffectiveTags' },
      { name: 'compute.regionTargetHttpsProxies.listTagBindings' },
      { name: 'compute.regionTargetTcpProxies.get' },
      { name: 'compute.regionTargetTcpProxies.list' },
      { name: 'compute.regionTargetTcpProxies.listEffectiveTags' },
      { name: 'compute.regionTargetTcpProxies.listTagBindings' },
      { name: 'compute.regionUrlMaps.get' },
      { name: 'compute.regionUrlMaps.list' },
      { name: 'compute.regionUrlMaps.listEffectiveTags' },
      { name: 'compute.regionUrlMaps.listTagBindings' },
      { name: 'compute.regions.*' },
      { name: 'compute.regions.list' },
      { name: 'compute.regions.get' },
      { name: 'compute.reservations.get' },
      { name: 'compute.reservations.list' },
      { name: 'compute.resourcePolicies.*' },
      { name: 'compute.routers.get' },
      { name: 'compute.routers.getRoutePolicy' },
      { name: 'compute.routers.list' },
      { name: 'compute.routers.listBgpRoutes' },
      { name: 'compute.routers.listEffectiveTags' },
      { name: 'compute.routers.listRoutePolicies' },
      { name: 'compute.routers.listTagBindings' },
      { name: 'compute.routes.get' },
      { name: 'compute.routes.list' },
      { name: 'compute.routes.listEffectiveTags' },
      { name: 'compute.routes.listTagBindings' },
      { name: 'compute.serviceAttachments.get' },
      { name: 'compute.serviceAttachments.list' },
      { name: 'compute.serviceAttachments.listEffectiveTags' },
      { name: 'compute.serviceAttachments.listTagBindings' },
      { name: 'compute.snapshots.*' },
      { name: 'compute.spotAssistants.get' },
      { name: 'compute.sslCertificates.get' },
      { name: 'compute.sslCertificates.list' },
      { name: 'compute.sslCertificates.listEffectiveTags' },
      { name: 'compute.sslCertificates.listTagBindings' },
      { name: 'compute.sslPolicies.get' },
      { name: 'compute.sslPolicies.list' },
      { name: 'compute.sslPolicies.listAvailableFeatures' },
      { name: 'compute.sslPolicies.listEffectiveTags' },
      { name: 'compute.sslPolicies.listTagBindings' },
      { name: 'compute.storagePools.get' },
      { name: 'compute.storagePools.list' },
      { name: 'compute.storagePools.use' },
      { name: 'compute.subnetworks.get' },
      { name: 'compute.subnetworks.list' },
      { name: 'compute.subnetworks.listEffectiveTags' },
      { name: 'compute.subnetworks.listTagBindings' },
      { name: 'compute.subnetworks.use' },
      { name: 'compute.subnetworks.useExternalIp' },
      { name: 'compute.targetGrpcProxies.get' },
      { name: 'compute.targetGrpcProxies.list' },
      { name: 'compute.targetGrpcProxies.listEffectiveTags' },
      { name: 'compute.targetGrpcProxies.listTagBindings' },
      { name: 'compute.targetHttpProxies.get' },
      { name: 'compute.targetHttpProxies.list' },
      { name: 'compute.targetHttpProxies.listEffectiveTags' },
      { name: 'compute.targetHttpProxies.listTagBindings' },
      { name: 'compute.targetHttpsProxies.get' },
      { name: 'compute.targetHttpsProxies.list' },
      { name: 'compute.targetHttpsProxies.listEffectiveTags' },
      { name: 'compute.targetHttpsProxies.listTagBindings' },
      { name: 'compute.targetInstances.get' },
      { name: 'compute.targetInstances.list' },
      { name: 'compute.targetInstances.listEffectiveTags' },
      { name: 'compute.targetInstances.listTagBindings' },
      { name: 'compute.targetPools.get' },
      { name: 'compute.targetPools.list' },
      { name: 'compute.targetPools.listEffectiveTags' },
      { name: 'compute.targetPools.listTagBindings' },
      { name: 'compute.targetSslProxies.get' },
      { name: 'compute.targetSslProxies.list' },
      { name: 'compute.targetSslProxies.listEffectiveTags' },
      { name: 'compute.targetSslProxies.listTagBindings' },
      { name: 'compute.targetTcpProxies.get' },
      { name: 'compute.targetTcpProxies.list' },
      { name: 'compute.targetTcpProxies.listEffectiveTags' },
      { name: 'compute.targetTcpProxies.listTagBindings' },
      { name: 'compute.targetVpnGateways.get' },
      { name: 'compute.targetVpnGateways.list' },
      { name: 'compute.targetVpnGateways.listEffectiveTags' },
      { name: 'compute.targetVpnGateways.listTagBindings' },
      { name: 'compute.urlMaps.get' },
      { name: 'compute.urlMaps.list' },
      { name: 'compute.urlMaps.listEffectiveTags' },
      { name: 'compute.urlMaps.listTagBindings' },
      { name: 'compute.vpnGateways.get' },
      { name: 'compute.vpnGateways.list' },
      { name: 'compute.vpnGateways.listEffectiveTags' },
      { name: 'compute.vpnGateways.listTagBindings' },
      { name: 'compute.vpnTunnels.get' },
      { name: 'compute.vpnTunnels.list' },
      { name: 'compute.vpnTunnels.listEffectiveTags' },
      { name: 'compute.vpnTunnels.listTagBindings' },
      { name: 'compute.zoneOperations.get' },
      { name: 'compute.zoneOperations.list' },
      { name: 'compute.zones.*' },
      { name: 'compute.zones.get' },
      { name: 'compute.zones.list' },
    ]

    await PermissionFactory.merge(
      permissions.map((permission) => {
        const permissionSplited = permission.name.split('.')
        return {
          serviceId: permissionSplited[0],
          typeId: permissionSplited[1],
          verbId: permissionSplited[2],
        }
      })
    ).createMany(permissions.length)

    const roles = [
      { id: 'roles/compute.admin', serviceId: 'compute' },
      { id: 'roles/iam.admin', serviceId: 'iam' },
      { id: 'roles/resourcemanager.organizationAdmin', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.organizationViewer', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.organizationCreator', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.projectAdmin', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.projectViewer', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.projectCreator', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.folderAdmin', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.folderViewer', serviceId: 'resourcemanager' },
      { id: 'roles/resourcemanager.folderCreator', serviceId: 'resourcemanager' },
    ]

    await RoleFactory.merge(roles).createMany(roles.length)
  }
}
