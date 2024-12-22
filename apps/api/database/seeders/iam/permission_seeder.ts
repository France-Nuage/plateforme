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
      'resourcemanager.organizations.get',
      'resourcemanager.organizations.getIamPolicy',
      'resourcemanager.organizations.searchPolicyBindings',
      'resourcemanager.organizations.setIamPolicy',
      'resourcemanager.organizations.updatePolicyBinding',
      'resourcemanager.folders.get',
      'resourcemanager.folders.list',
      'resourcemanager.folders.create',
      'resourcemanager.folders.delete',
      'resourcemanager.projects.get',
      'resourcemanager.projects.list',
      'resourcemanager.projects.create',
      'resourcemanager.projects.delete',
      'resourcemanager.projects.getIamPolicy',
      'resourcemanager.projects.getIamPolicy',
      'resourcemanager.organizations.getIamPolicy',
      'resourcemanager.folders.getIamPolicy',
      'cloudasset.assets.listIamRoles',
      'cloudasset.assets.exportIamRoles',
      'iam.operations.get',
      'logging.buckets.copyLogEntries',
      'logging.buckets.create',
      'logging.buckets.delete',
      'logging.buckets.get',
      'logging.buckets.listEffectiveTags',
      'logging.buckets.list',
      'logging.buckets.createTagBinding',
      'logging.buckets.deleteTagBinding',
      'logging.buckets.listTagBindings',
      'logging.buckets.undelete',
      'logging.buckets.update',
      'logging.exclusions.*',
      'logging.fields.access',
      'logging.links.*',
      'logging.locations.*',
      'logging.logEntries.*',
      'logging.logMetrics.*',
      'logging.logServiceIndexes.list',
      'logging.logServices.list',
      'logging.logs.*',
      'logging.notificationRules.*',
      'logging.operations.*',
      'logging.privateLogEntries.list',
      'logging.queries.*',
      'logging.settings.*',
      'logging.sinks.*',
      'logging.sqlAlerts.*',
      'logging.usage.get',
      'logging.views.*',
      'observability.scopes.get',
      'compute.acceleratorTypes.*',
      'compute.addresses.createInternal',
      'compute.addresses.deleteInternal',
      'compute.addresses.get',
      'compute.addresses.list',
      'compute.addresses.listEffectiveTags',
      'compute.addresses.listTagBindings',
      'compute.addresses.use',
      'compute.addresses.useInternal',
      'compute.autoscalers.*',
      'compute.backendBuckets.get',
      'compute.backendBuckets.list',
      'compute.backendBuckets.listEffectiveTags',
      'compute.backendBuckets.listTagBindings',
      'compute.backendServices.get',
      'compute.backendServices.list',
      'compute.backendServices.listEffectiveTags',
      'compute.backendServices.listTagBindings',
      'compute.diskTypes.*',
      'compute.disks.*',
      'compute.externalVpnGateways.get',
      'compute.externalVpnGateways.list',
      'compute.externalVpnGateways.listEffectiveTags',
      'compute.externalVpnGateways.listTagBindings',
      'compute.firewalls.get',
      'compute.firewalls.list',
      'compute.firewalls.listEffectiveTags',
      'compute.firewalls.listTagBindings',
      'compute.forwardingRules.get',
      'compute.forwardingRules.list',
      'compute.forwardingRules.listEffectiveTags',
      'compute.forwardingRules.listTagBindings',
      'compute.globalAddresses.get',
      'compute.globalAddresses.list',
      'compute.globalAddresses.listEffectiveTags',
      'compute.globalAddresses.listTagBindings',
      'compute.globalAddresses.use',
      'compute.globalForwardingRules.get',
      'compute.globalForwardingRules.list',
      'compute.globalForwardingRules.listEffectiveTags',
      'compute.globalForwardingRules.listTagBindings',
      'compute.globalForwardingRules.pscGet',
      'compute.globalNetworkEndpointGroups.*',
      'compute.globalOperations.get',
      'compute.globalOperations.list',
      'compute.healthChecks.get',
      'compute.healthChecks.list',
      'compute.healthChecks.listEffectiveTags',
      'compute.healthChecks.listTagBindings',
      'compute.httpHealthChecks.get',
      'compute.httpHealthChecks.list',
      'compute.httpHealthChecks.listEffectiveTags',
      'compute.httpHealthChecks.listTagBindings',
      'compute.httpsHealthChecks.get',
      'compute.httpsHealthChecks.list',
      'compute.httpsHealthChecks.listEffectiveTags',
      'compute.httpsHealthChecks.listTagBindings',
      'compute.images.*',
      'compute.instanceGroupManagers.*',
      'compute.instanceGroups.*',
      'compute.instanceSettings.*',
      'compute.instanceTemplates.*',
      'compute.instances.*',
      'compute.instantSnapshots.*',
      'compute.interconnectAttachments.get',
      'compute.interconnectAttachments.list',
      'compute.interconnectAttachments.listEffectiveTags',
      'compute.interconnectAttachments.listTagBindings',
      'compute.interconnectLocations.*',
      'compute.interconnectRemoteLocations.*',
      'compute.interconnects.get',
      'compute.interconnects.list',
      'compute.interconnects.listEffectiveTags',
      'compute.interconnects.listTagBindings',
      'compute.licenseCodes.*',
      'compute.licenses.*',
      'compute.machineImages.*',
      'compute.machineTypes.*',
      'compute.multiMig.*',
      'compute.networkAttachments.get',
      'compute.networkAttachments.list',
      'compute.networkAttachments.listEffectiveTags',
      'compute.networkAttachments.listTagBindings',
      'compute.networkEndpointGroups.*',
      'compute.networkProfiles.*',
      'compute.networks.get',
      'compute.networks.list',
      'compute.networks.listEffectiveTags',
      'compute.networks.listTagBindings',
      'compute.networks.use',
      'compute.networks.useExternalIp',
      'compute.projects.get',
      'compute.projects.setCommonInstanceMetadata',
      'compute.regionBackendServices.get',
      'compute.regionBackendServices.list',
      'compute.regionBackendServices.listEffectiveTags',
      'compute.regionBackendServices.listTagBindings',
      'compute.regionHealthCheckServices.get',
      'compute.regionHealthCheckServices.list',
      'compute.regionHealthChecks.get',
      'compute.regionHealthChecks.list',
      'compute.regionHealthChecks.listEffectiveTags',
      'compute.regionHealthChecks.listTagBindings',
      'compute.regionNetworkEndpointGroups.*',
      'compute.regionNotificationEndpoints.get',
      'compute.regionNotificationEndpoints.list',
      'compute.regionOperations.get',
      'compute.regionOperations.list',
      'compute.regionSslCertificates.get',
      'compute.regionSslCertificates.list',
      'compute.regionSslCertificates.listEffectiveTags',
      'compute.regionSslCertificates.listTagBindings',
      'compute.regionSslPolicies.get',
      'compute.regionSslPolicies.list',
      'compute.regionSslPolicies.listAvailableFeatures',
      'compute.regionSslPolicies.listEffectiveTags',
      'compute.regionSslPolicies.listTagBindings',
      'compute.regionTargetHttpProxies.get',
      'compute.regionTargetHttpProxies.list',
      'compute.regionTargetHttpProxies.listEffectiveTags',
      'compute.regionTargetHttpProxies.listTagBindings',
      'compute.regionTargetHttpsProxies.get',
      'compute.regionTargetHttpsProxies.list',
      'compute.regionTargetHttpsProxies.listEffectiveTags',
      'compute.regionTargetHttpsProxies.listTagBindings',
      'compute.regionTargetTcpProxies.get',
      'compute.regionTargetTcpProxies.list',
      'compute.regionTargetTcpProxies.listEffectiveTags',
      'compute.regionTargetTcpProxies.listTagBindings',
      'compute.regionUrlMaps.get',
      'compute.regionUrlMaps.list',
      'compute.regionUrlMaps.listEffectiveTags',
      'compute.regionUrlMaps.listTagBindings',
      'compute.regions.*',
      'compute.regions.list',
      'compute.regions.get',
      'compute.reservations.get',
      'compute.reservations.list',
      'compute.resourcePolicies.*',
      'compute.routers.get',
      'compute.routers.getRoutePolicy',
      'compute.routers.list',
      'compute.routers.listBgpRoutes',
      'compute.routers.listEffectiveTags',
      'compute.routers.listRoutePolicies',
      'compute.routers.listTagBindings',
      'compute.routes.get',
      'compute.routes.list',
      'compute.routes.listEffectiveTags',
      'compute.routes.listTagBindings',
      'compute.serviceAttachments.get',
      'compute.serviceAttachments.list',
      'compute.serviceAttachments.listEffectiveTags',
      'compute.serviceAttachments.listTagBindings',
      'compute.snapshots.*',
      'compute.spotAssistants.get',
      'compute.sslCertificates.get',
      'compute.sslCertificates.list',
      'compute.sslCertificates.listEffectiveTags',
      'compute.sslCertificates.listTagBindings',
      'compute.sslPolicies.get',
      'compute.sslPolicies.list',
      'compute.sslPolicies.listAvailableFeatures',
      'compute.sslPolicies.listEffectiveTags',
      'compute.sslPolicies.listTagBindings',
      'compute.storagePools.get',
      'compute.storagePools.list',
      'compute.storagePools.use',
      'compute.subnetworks.get',
      'compute.subnetworks.list',
      'compute.subnetworks.listEffectiveTags',
      'compute.subnetworks.listTagBindings',
      'compute.subnetworks.use',
      'compute.subnetworks.useExternalIp',
      'compute.targetGrpcProxies.get',
      'compute.targetGrpcProxies.list',
      'compute.targetGrpcProxies.listEffectiveTags',
      'compute.targetGrpcProxies.listTagBindings',
      'compute.targetHttpProxies.get',
      'compute.targetHttpProxies.list',
      'compute.targetHttpProxies.listEffectiveTags',
      'compute.targetHttpProxies.listTagBindings',
      'compute.targetHttpsProxies.get',
      'compute.targetHttpsProxies.list',
      'compute.targetHttpsProxies.listEffectiveTags',
      'compute.targetHttpsProxies.listTagBindings',
      'compute.targetInstances.get',
      'compute.targetInstances.list',
      'compute.targetInstances.listEffectiveTags',
      'compute.targetInstances.listTagBindings',
      'compute.targetPools.get',
      'compute.targetPools.list',
      'compute.targetPools.listEffectiveTags',
      'compute.targetPools.listTagBindings',
      'compute.targetSslProxies.get',
      'compute.targetSslProxies.list',
      'compute.targetSslProxies.listEffectiveTags',
      'compute.targetSslProxies.listTagBindings',
      'compute.targetTcpProxies.get',
      'compute.targetTcpProxies.list',
      'compute.targetTcpProxies.listEffectiveTags',
      'compute.targetTcpProxies.listTagBindings',
      'compute.targetVpnGateways.get',
      'compute.targetVpnGateways.list',
      'compute.targetVpnGateways.listEffectiveTags',
      'compute.targetVpnGateways.listTagBindings',
      'compute.urlMaps.get',
      'compute.urlMaps.list',
      'compute.urlMaps.listEffectiveTags',
      'compute.urlMaps.listTagBindings',
      'compute.vpnGateways.get',
      'compute.vpnGateways.list',
      'compute.vpnGateways.listEffectiveTags',
      'compute.vpnGateways.listTagBindings',
      'compute.vpnTunnels.get',
      'compute.vpnTunnels.list',
      'compute.vpnTunnels.listEffectiveTags',
      'compute.vpnTunnels.listTagBindings',
      'compute.zoneOperations.get',
      'compute.zoneOperations.list',
      'compute.zones.*',
      'compute.zones.get',
      'compute.zones.list'
    ]

    await PermissionFactory.merge(
      permissions.map((permission) => {
        const permissionSplited = permission.split('.')
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
