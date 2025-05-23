// @generated by protobuf-ts 2.9.6 with parameter optimize_code_size
// @generated from protobuf file "instances.proto" (package "francenuage.fr.api.controlplane.v1.instances", syntax proto3)
// tslint:disable
import type { RpcTransport } from "@protobuf-ts/runtime-rpc";
import type { ServiceInfo } from "@protobuf-ts/runtime-rpc";
import { Instances } from "./instances";
import type { StopInstanceResponse } from "./instances";
import type { StopInstanceRequest } from "./instances";
import type { StartInstanceResponse } from "./instances";
import type { StartInstanceRequest } from "./instances";
import type { CreateInstanceResponse } from "./instances";
import type { CreateInstanceRequest } from "./instances";
import { stackIntercept } from "@protobuf-ts/runtime-rpc";
import type { ListInstancesResponse } from "./instances";
import type { ListInstancesRequest } from "./instances";
import type { UnaryCall } from "@protobuf-ts/runtime-rpc";
import type { RpcOptions } from "@protobuf-ts/runtime-rpc";
/**
 * Instances service provides operations to manage instances.
 *
 * @generated from protobuf service francenuage.fr.api.controlplane.v1.instances.Instances
 */
export interface IInstancesClient {
  /**
   * ListInstances retrieves information about all available instances.
   * Returns a collection of instance details including their current status and resource usage.
   *
   * @generated from protobuf rpc: ListInstances(francenuage.fr.api.controlplane.v1.instances.ListInstancesRequest) returns (francenuage.fr.api.controlplane.v1.instances.ListInstancesResponse);
   */
  listInstances(
    input: ListInstancesRequest,
    options?: RpcOptions,
  ): UnaryCall<ListInstancesRequest, ListInstancesResponse>;
  /**
   * CreateInstance provisions a new instance based on the specified configuration.
   * Returns details of the newly created instance or a ProblemDetails on failure.
   *
   * @generated from protobuf rpc: CreateInstance(francenuage.fr.api.controlplane.v1.instances.CreateInstanceRequest) returns (francenuage.fr.api.controlplane.v1.instances.CreateInstanceResponse);
   */
  createInstance(
    input: CreateInstanceRequest,
    options?: RpcOptions,
  ): UnaryCall<CreateInstanceRequest, CreateInstanceResponse>;
  /**
   * StartInstance initiates a specific instance identified by its unique ID.
   * Returns a response indicating success or a ProblemDetails on failure.
   *
   * @generated from protobuf rpc: StartInstance(francenuage.fr.api.controlplane.v1.instances.StartInstanceRequest) returns (francenuage.fr.api.controlplane.v1.instances.StartInstanceResponse);
   */
  startInstance(
    input: StartInstanceRequest,
    options?: RpcOptions,
  ): UnaryCall<StartInstanceRequest, StartInstanceResponse>;
  /**
   * StopInstance halts a specific instance identified by its unique ID.
   * Returns a response indicating success or a ProblemDetails on failure.
   *
   * @generated from protobuf rpc: StopInstance(francenuage.fr.api.controlplane.v1.instances.StopInstanceRequest) returns (francenuage.fr.api.controlplane.v1.instances.StopInstanceResponse);
   */
  stopInstance(
    input: StopInstanceRequest,
    options?: RpcOptions,
  ): UnaryCall<StopInstanceRequest, StopInstanceResponse>;
}
/**
 * Instances service provides operations to manage instances.
 *
 * @generated from protobuf service francenuage.fr.api.controlplane.v1.instances.Instances
 */
export class InstancesClient implements IInstancesClient, ServiceInfo {
  typeName = Instances.typeName;
  methods = Instances.methods;
  options = Instances.options;
  constructor(private readonly _transport: RpcTransport) {}
  /**
   * ListInstances retrieves information about all available instances.
   * Returns a collection of instance details including their current status and resource usage.
   *
   * @generated from protobuf rpc: ListInstances(francenuage.fr.api.controlplane.v1.instances.ListInstancesRequest) returns (francenuage.fr.api.controlplane.v1.instances.ListInstancesResponse);
   */
  listInstances(
    input: ListInstancesRequest,
    options?: RpcOptions,
  ): UnaryCall<ListInstancesRequest, ListInstancesResponse> {
    const method = this.methods[0],
      opt = this._transport.mergeOptions(options);
    return stackIntercept<ListInstancesRequest, ListInstancesResponse>(
      "unary",
      this._transport,
      method,
      opt,
      input,
    );
  }
  /**
   * CreateInstance provisions a new instance based on the specified configuration.
   * Returns details of the newly created instance or a ProblemDetails on failure.
   *
   * @generated from protobuf rpc: CreateInstance(francenuage.fr.api.controlplane.v1.instances.CreateInstanceRequest) returns (francenuage.fr.api.controlplane.v1.instances.CreateInstanceResponse);
   */
  createInstance(
    input: CreateInstanceRequest,
    options?: RpcOptions,
  ): UnaryCall<CreateInstanceRequest, CreateInstanceResponse> {
    const method = this.methods[1],
      opt = this._transport.mergeOptions(options);
    return stackIntercept<CreateInstanceRequest, CreateInstanceResponse>(
      "unary",
      this._transport,
      method,
      opt,
      input,
    );
  }
  /**
   * StartInstance initiates a specific instance identified by its unique ID.
   * Returns a response indicating success or a ProblemDetails on failure.
   *
   * @generated from protobuf rpc: StartInstance(francenuage.fr.api.controlplane.v1.instances.StartInstanceRequest) returns (francenuage.fr.api.controlplane.v1.instances.StartInstanceResponse);
   */
  startInstance(
    input: StartInstanceRequest,
    options?: RpcOptions,
  ): UnaryCall<StartInstanceRequest, StartInstanceResponse> {
    const method = this.methods[2],
      opt = this._transport.mergeOptions(options);
    return stackIntercept<StartInstanceRequest, StartInstanceResponse>(
      "unary",
      this._transport,
      method,
      opt,
      input,
    );
  }
  /**
   * StopInstance halts a specific instance identified by its unique ID.
   * Returns a response indicating success or a ProblemDetails on failure.
   *
   * @generated from protobuf rpc: StopInstance(francenuage.fr.api.controlplane.v1.instances.StopInstanceRequest) returns (francenuage.fr.api.controlplane.v1.instances.StopInstanceResponse);
   */
  stopInstance(
    input: StopInstanceRequest,
    options?: RpcOptions,
  ): UnaryCall<StopInstanceRequest, StopInstanceResponse> {
    const method = this.methods[3],
      opt = this._transport.mergeOptions(options);
    return stackIntercept<StopInstanceRequest, StopInstanceResponse>(
      "unary",
      this._transport,
      method,
      opt,
      input,
    );
  }
}
