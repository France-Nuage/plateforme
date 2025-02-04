Ce que ca doit faire :

Supporter la notion de regopn :
Guide :
GET https://compute.googleapis.com/compute/v1/projects/{project}/regions/{region} (https://cloud.google.com/compute/docs/reference/rest/v1/regions/get)
GET https://compute.googleapis.com/compute/v1/projects/{project}/regions (https://cloud.google.com/compute/docs/reference/rest/v1/regions/list)

Supporter la notion de zone :
Guide : https://cloud.google.com/compute/docs/reference/rest/v1/zones
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones (https://cloud.google.com/compute/docs/reference/rest/v1/zones/list)
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone} (https://cloud.google.com/compute/docs/reference/rest/v1/zones/get)

Supporter la notion de zone par région :
Guide : https://cloud.google.com/compute/docs/reference/rest/v1/regionZones
GET https://compute.googleapis.com/compute/v1/projects/{project}/regions/{region}/zones (https://cloud.google.com/compute/docs/reference/rest/v1/regionZones/list)

Supporter la notion de disk par zone :  
Guide : https://cloud.google.com/compute/docs/reference/rest/v1/diskTypes
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/diskTypes/{diskType} (https://cloud.google.com/compute/docs/reference/rest/v1/diskTypes/get)
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/diskTypes (https://cloud.google.com/compute/docs/reference/rest/v1/diskTypes/list)

Supporter la notion de disk par région :  
Guide : https://cloud.google.com/compute/docs/reference/rest/v1/diskTypes
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/regions/{region}/diskTypes/{diskType} (https://cloud.google.com/compute/docs/reference/rest/v1/regionDiskTypes/get)
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/regions/{region}/diskTypes (https://cloud.google.com/compute/docs/reference/rest/v1/regionDiskTypes/list)

Supporter la notion de type de machine :
Guide : https://cloud.google.com/compute/docs/reference/rest/v1/diskTypes
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/machineTypes (https://cloud.google.com/compute/docs/reference/rest/v1/machineTypes/get)
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/machineTypes/{machineType} (https://cloud.google.com/compute/docs/reference/rest/v1/machineTypes/list)

Supporter la notion de nodeGroup :
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/nodeGroups (https://cloud.google.com/compute/docs/reference/rest/v1/nodeGroups/list)
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/nodeGroups/{nodeGroup} (https://cloud.google.com/compute/docs/reference/rest/v1/nodeGroups/get)

Supporter la notion de nodeTemplate :
GET https://compute.googleapis.com/compute/v1/projects/{project}/regions/{region}/nodeTemplates (https://cloud.google.com/compute/docs/reference/rest/v1/nodeTemplates/list)
GET https://compute.googleapis.com/compute/v1/projects/{project}/regions/{region}/nodeTemplates/{nodeTemplate} (https://cloud.google.com/compute/docs/reference/rest/v1/nodeTemplates/get)

Supporter la notion de nodeTypes :
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/nodeTypes (https://cloud.google.com/compute/docs/reference/rest/v1/nodeTypes/list)
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/nodeTypes/{nodeType} (https://cloud.google.com/compute/docs/reference/rest/v1/nodeTypes/get)

Supporter la notion de famille d'image :
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/imageFamilyViews (innexistante)
GET https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/imageFamilyViews/{family} (https://cloud.google.com/compute/docs/reference/rest/v1/imageFamilyViews/get)

Supporter la notion d'image :
GET https://compute.googleapis.com/compute/v1/projects/{project}/global/images (https://cloud.google.com/compute/docs/reference/rest/v1/images/list)
GET https://compute.googleapis.com/compute/v1/projects/{project}/global/images/{image} (https://cloud.google.com/compute/docs/reference/rest/v1/images/get)
