import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { Project as RpcProject } from '../../generated/rpc/resourcemanager';
import { ProjectsClient } from '../../generated/rpc/resourcemanager.client';
import { Project, ProjectFormValue } from '../../models';
import { ProjectService } from '../api';

export class ProjectRpcService implements ProjectService {
  /**
   * The gRPC resources client
   */
  private client: ProjectsClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ProjectsClient(transport);
  }

  /** @inheritdoc */
  public create(data: ProjectFormValue): Promise<Project> {
    return this.client
      .create({
        name: data.name,
        organizationId: data.organizationId,
      })
      .response.then(({ project }) => fromRpcProject(project!));
  }

  /** @inheritdoc */
  public list(): Promise<Project[]> {
    return this.client
      .list({})
      .response.then(({ projects }) => projects.map(fromRpcProject));
  }
}

/**
 * Convert a protocol project into a concrete Project.
 */
function fromRpcProject(project: RpcProject): Project {
  return {
    id: project.id,
    name: project.name,
    organizationId: project.organizationId,
  };
}
