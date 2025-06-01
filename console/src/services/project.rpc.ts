import { Project, ProjectFormValue } from "@/types";
import { ProjectService } from "./project.interface";
import { ResourcesClient } from "@/generated/rpc/resources.client";
import { Project as RpcProject } from "@/generated/rpc/resources";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { transport } from "./transport.rpc";

export class ProjectRpcService implements ProjectService {
  /**
   * The gRPC resources client
   */
  private client: ResourcesClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ResourcesClient(transport);
  }

  /** @inheritdoc */
  public create(data: ProjectFormValue): Promise<Project> {
    return this.client.createProject({
      name: data.name,
      organizationId: data.organizationId,
    }).response.then(({ project }) => fromRpcProject(project!));
  }

  /** @inheritdoc */
  public list(): Promise<Project[]> {
    return this.client.listProjects({}).response.then(({ projects }) => projects.map(fromRpcProject));
  }
}

export const projectRpcService = new ProjectRpcService(transport);

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
