import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  async up() {
    this.schema.withSchema('iam').table('role__permission', (table) => {
      table.dropForeign('permission__id')
      table.dropColumn('permission__id')
    })

    this.schema.withSchema('iam').table('permissions', (table) => {
      table.dropPrimary('permissions_pkey')
      table.dropColumn('permission__id')
    })

    this.db.from('iam.role__permission').delete()
    this.db.from('iam.permissions').delete()
    this.db.from('iam.verbs').delete()
    this.db.from('catalog.services').delete()
    this.db.from('iam.types').delete()

    this.schema.withSchema('iam').table('permissions', (table) => {
      table.string('permission__id', 255)

      table.primary(['permission__id'])
    })

    this.schema.withSchema('iam').alterTable('role__permission', (table) => {
      table.string('permission__id', 255)
      table
        .foreign('permission__id')
        .references('permission__id')
        .inTable('iam.permissions')
        .onDelete('restrict')
        .onUpdate('cascade')

      table.unique(['role__id', 'permission__id'])
    })

    this.db
      .table('iam.verbs')
      .multiInsert([
        { verb__id: 'abort' },
        { verb__id: 'access' },
        { verb__id: 'actAs' },
        { verb__id: 'add' },
        { verb__id: 'attach' },
        { verb__id: 'bind' },
        { verb__id: 'calculate' },
        { verb__id: 'call' },
        { verb__id: 'cancel' },
        { verb__id: 'check' },
        { verb__id: 'cloneRules' },
        { verb__id: 'close' },
        { verb__id: 'connect' },
        { verb__id: 'consume' },
        { verb__id: 'create' },
        { verb__id: 'copyLogEntries' },
        { verb__id: 'createInternal' },
        { verb__id: 'createTagBinding' },
        { verb__id: 'deleteInternal' },
        { verb__id: 'deleteTagBinding' },
        { verb__id: 'delete' },
        { verb__id: 'deploy' },
        { verb__id: 'destroy' },
        { verb__id: 'detachSubscription' },
        { verb__id: 'disable' },
        { verb__id: 'download' },
        { verb__id: 'drop' },
        { verb__id: 'exportIamRoles' },
        { verb__id: 'enable' },
        { verb__id: 'escalate' },
        { verb__id: 'execute' },
        { verb__id: 'explain' },
        { verb__id: 'export' },
        { verb__id: 'failover' },
        { verb__id: 'get' },
        { verb__id: 'group' },
        { verb__id: 'getIamPolicy' },
        { verb__id: 'getRoutePolicy' },
        { verb__id: 'import' },
        { verb__id: 'ingest' },
        { verb__id: 'install' },
        { verb__id: 'instantiate' },
        { verb__id: 'instantiateInline' },
        { verb__id: 'invoke' },
        { verb__id: 'list' },
        { verb__id: 'listActive' },
        { verb__id: 'listAll' },
        { verb__id: 'listIamRoles' },
        { verb__id: 'login' },
        { verb__id: 'lookup' },
        { verb__id: 'listEffectiveTags' },
        { verb__id: 'listTagBindings' },
        { verb__id: 'listBgpRoutes' },
        { verb__id: 'listRoutePolicies' },
        { verb__id: 'listAvailableFeatures' },
        { verb__id: 'manage' },
        { verb__id: 'mirror' },
        { verb__id: 'move' },
        { verb__id: 'patch' },
        { verb__id: 'pause' },
        { verb__id: 'publish' },
        { verb__id: 'purge' },
        { verb__id: 'quota' },
        { verb__id: 'pscGet' },
        { verb__id: 'read' },
        { verb__id: 'reopen' },
        { verb__id: 'report' },
        { verb__id: 'reportStatus' },
        { verb__id: 'reset' },
        { verb__id: 'resetpassword' },
        { verb__id: 'resize' },
        { verb__id: 'resolve' },
        { verb__id: 'restart' },
        { verb__id: 'restore' },
        { verb__id: 'resume' },
        { verb__id: 'review' },
        { verb__id: 'run' },
        { verb__id: 'runDiscovery' },
        { verb__id: 'runtime' },
        { verb__id: 'sampleRowKeys' },
        { verb__id: 'search' },
        { verb__id: 'seek' },
        { verb__id: 'select' },
        { verb__id: 'sendCommand' },
        { verb__id: 'sendVerificationCode' },
        { verb__id: 'set' },
        { verb__id: 'setMetadata' },
        { verb__id: 'setState' },
        { verb__id: 'setTags' },
        { verb__id: 'searchPolicyBindings' },
        { verb__id: 'setIamPolicy' },
        { verb__id: 'setCommonInstanceMetadata' },
        { verb__id: 'start' },
        { verb__id: 'stop' },
        { verb__id: 'subscribe' },
        { verb__id: 'truncateLog' },
        { verb__id: 'undelete' },
        { verb__id: 'undeploy' },
        { verb__id: 'uninstall' },
        { verb__id: 'updatePolicyBinding' },
        { verb__id: 'update' },
        { verb__id: 'useExternalIp' },
        { verb__id: 'use' },
        { verb__id: 'useInternal' },
        { verb__id: 'validate' },
        { verb__id: 'validateTrust' },
        { verb__id: 'verify' },
        { verb__id: 'view' },
        { verb__id: 'wait' },
        { verb__id: 'watch' },
        { verb__id: 'write' },
        { verb__id: '*' },
      ])
    this.db
      .table('catalog.services')
      .multiInsert([
        { service__id: '*' },
        { service__id: 'iam' },
        { service__id: 'logging' },
        { service__id: 'compute' },
        { service__id: 'resourcemanager' },
        { service__id: 'observability' },
        { service__id: 'cloudassets' },
      ])
    this.db.table('iam.types').multiInsert([
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
    ])

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
      'compute.zones.list',
    ]

    this.db.table('iam.permissions').multiInsert(
      permissions.map((permission) => {
        const permissionSplit = permission.split('.')
        return {
          permission__id: permission,
          service__id: permissionSplit[0],
          type__id: permissionSplit[1],
          verb__id: permissionSplit[2],
        }
      })
    )

    this.db.table('iam.roles').multiInsert([
      { role__id: 'roles/compute.admin', service__id: 'compute' },
      { role__id: 'roles/iam.admin', service__id: 'iam' },
      { role__id: 'roles/resourcemanager.organizationAdmin', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.organizationViewer', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.organizationCreator', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.projectAdmin', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.projectViewer', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.projectCreator', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.folderAdmin', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.folderViewer', service__id: 'resourcemanager' },
      { role__id: 'roles/resourcemanager.folderCreator', service__id: 'resourcemanager' },
    ])

    this.db.table('iam.role__permission').multiInsert([
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.organizations.get',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.organizations.get',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.organizations.getIamPolicy',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.folders.list',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.projects.get',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.projects.getIamPolicy',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.projects.list',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'compute.projects.get',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'compute.projects.setCommonInstanceMetadata',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.folders.create',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.projects.create',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.projects.delete',
      },
      {
        role__id: 'roles/resourcemanager.organizationAdmin',
        permission__id: 'resourcemanager.folders.getIamPolicy',
      },
    ])
  }
}
