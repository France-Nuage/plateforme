// @generated by protobuf-ts 2.9.6 with parameter optimize_code_size
// @generated from protobuf file "controlplane.proto" (package "francenuage.fr.api.controlplane.v1", syntax proto3)
// tslint:disable
import { ServiceType } from "@protobuf-ts/runtime-rpc";
import { MessageType } from "@protobuf-ts/runtime";
import { Empty } from "./google/protobuf/empty";
import { ProblemDetails } from "./problem";
/**
 * InstanceInfo contains detailed information about a virtual machine instance.
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.InstanceInfo
 */
export interface InstanceInfo {
  /**
   * Unique identifier for the instance
   *
   * @generated from protobuf field: string id = 1;
   */
  id: string;
  /**
   * Current operational status of the instance
   *
   * @generated from protobuf field: francenuage.fr.api.controlplane.v1.InstanceStatus status = 2;
   */
  status: InstanceStatus;
  /**
   * Maximum CPU cores available to the instance (max 99)
   *
   * @generated from protobuf field: uint32 max_cpu_cores = 3;
   */
  maxCpuCores: number;
  /**
   * Current CPU utilization as a percentage (0.0-100.0)
   *
   * @generated from protobuf field: float cpu_usage_percent = 4;
   */
  cpuUsagePercent: number;
  /**
   * Maximum memory available to the instance (in bytes, max 64GB)
   *
   * @generated from protobuf field: uint64 max_memory_bytes = 5;
   */
  maxMemoryBytes: bigint;
  /**
   * Current memory utilization (in bytes, cannot exceed max_memory_bytes)
   *
   * @generated from protobuf field: uint64 memory_usage_bytes = 6;
   */
  memoryUsageBytes: bigint;
}
/**
 * ListInstancesRequest is an empty message for listing instances.
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.ListInstancesRequest
 */
export interface ListInstancesRequest {}
/**
 * ListInstancesResponse contains a collection of instance information.
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.ListInstancesResponse
 */
export interface ListInstancesResponse {
  /**
   * @generated from protobuf oneof: result
   */
  result:
    | {
        oneofKind: "success";
        /**
         * List of instance details on success
         *
         * @generated from protobuf field: francenuage.fr.api.controlplane.v1.InstanceList success = 1;
         */
        success: InstanceList;
      }
    | {
        oneofKind: "problem";
        /**
         * Problem details on failure
         *
         * @generated from protobuf field: francenuage.fr.api.controlplane.v1.ProblemDetails problem = 2;
         */
        problem: ProblemDetails;
      }
    | {
        oneofKind: undefined;
      };
}
/**
 * Container for instance list
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.InstanceList
 */
export interface InstanceList {
  /**
   * List of instance details
   *
   * @generated from protobuf field: repeated francenuage.fr.api.controlplane.v1.InstanceInfo instances = 1;
   */
  instances: InstanceInfo[];
}
/**
 * StartInstanceRequest identifies which instance to start.
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.StartInstanceRequest
 */
export interface StartInstanceRequest {
  /**
   * Unique identifier of the instance to start
   *
   * @generated from protobuf field: string id = 1;
   */
  id: string;
}
/**
 * StartInstanceResponse contains the result of a start instance operation.
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.StartInstanceResponse
 */
export interface StartInstanceResponse {
  /**
   * @generated from protobuf oneof: result
   */
  result:
    | {
        oneofKind: "success";
        /**
         * Empty success response
         *
         * @generated from protobuf field: google.protobuf.Empty success = 1;
         */
        success: Empty;
      }
    | {
        oneofKind: "problem";
        /**
         * Problem details on failure
         *
         * @generated from protobuf field: francenuage.fr.api.controlplane.v1.ProblemDetails problem = 2;
         */
        problem: ProblemDetails;
      }
    | {
        oneofKind: undefined;
      };
}
/**
 * StopInstanceRequest identifies which instance to stop.
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.StopInstanceRequest
 */
export interface StopInstanceRequest {
  /**
   * Unique identifier of the instance to stop
   *
   * @generated from protobuf field: string id = 1;
   */
  id: string;
}
/**
 * StopInstanceResponse contains the result of a stop instance operation.
 *
 * @generated from protobuf message francenuage.fr.api.controlplane.v1.StopInstanceResponse
 */
export interface StopInstanceResponse {
  /**
   * @generated from protobuf oneof: result
   */
  result:
    | {
        oneofKind: "success";
        /**
         * Empty success response
         *
         * @generated from protobuf field: google.protobuf.Empty success = 1;
         */
        success: Empty;
      }
    | {
        oneofKind: "problem";
        /**
         * Problem details on failure
         *
         * @generated from protobuf field: francenuage.fr.api.controlplane.v1.ProblemDetails problem = 2;
         */
        problem: ProblemDetails;
      }
    | {
        oneofKind: undefined;
      };
}
/**
 * InstanceStatus represents the possible states of a virtual machine instance.
 *
 * @generated from protobuf enum francenuage.fr.api.controlplane.v1.InstanceStatus
 */
export enum InstanceStatus {
  /**
   * Instance is active and operational
   *
   * @generated from protobuf enum value: RUNNING = 0;
   */
  RUNNING = 0,
  /**
   * Instance is inactive
   *
   * @generated from protobuf enum value: STOPPED = 1;
   */
  STOPPED = 1,
}
// @generated message type with reflection information, may provide speed optimized methods
class InstanceInfo$Type extends MessageType<InstanceInfo> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.InstanceInfo", [
      {
        no: 1,
        name: "id",
        kind: "scalar",
        T: 9 /*ScalarType.STRING*/,
        options: {
          "validate.rules": {
            string: { minLen: "1", maxLen: "36", pattern: "^[a-zA-Z0-9_-]+$" },
          },
        },
      },
      {
        no: 2,
        name: "status",
        kind: "enum",
        T: () => [
          "francenuage.fr.api.controlplane.v1.InstanceStatus",
          InstanceStatus,
        ],
        options: { "validate.rules": { enum: { definedOnly: true } } },
      },
      {
        no: 3,
        name: "max_cpu_cores",
        kind: "scalar",
        T: 13 /*ScalarType.UINT32*/,
        options: { "validate.rules": { uint32: { lte: 99, gt: 0 } } },
      },
      {
        no: 4,
        name: "cpu_usage_percent",
        kind: "scalar",
        T: 2 /*ScalarType.FLOAT*/,
        options: { "validate.rules": { float: { lte: 100, gte: 0 } } },
      },
      {
        no: 5,
        name: "max_memory_bytes",
        kind: "scalar",
        T: 4 /*ScalarType.UINT64*/,
        L: 0 /*LongType.BIGINT*/,
        options: {
          "validate.rules": { uint64: { lte: "68719476736", gt: "0" } },
        },
      },
      {
        no: 6,
        name: "memory_usage_bytes",
        kind: "scalar",
        T: 4 /*ScalarType.UINT64*/,
        L: 0 /*LongType.BIGINT*/,
        options: {
          "validate.rules": { uint64: { lte: "68719476736", gte: "0" } },
        },
      },
    ]);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.InstanceInfo
 */
export const InstanceInfo = new InstanceInfo$Type();
// @generated message type with reflection information, may provide speed optimized methods
class ListInstancesRequest$Type extends MessageType<ListInstancesRequest> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.ListInstancesRequest", []);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.ListInstancesRequest
 */
export const ListInstancesRequest = new ListInstancesRequest$Type();
// @generated message type with reflection information, may provide speed optimized methods
class ListInstancesResponse$Type extends MessageType<ListInstancesResponse> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.ListInstancesResponse", [
      {
        no: 1,
        name: "success",
        kind: "message",
        oneof: "result",
        T: () => InstanceList,
      },
      {
        no: 2,
        name: "problem",
        kind: "message",
        oneof: "result",
        T: () => ProblemDetails,
      },
    ]);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.ListInstancesResponse
 */
export const ListInstancesResponse = new ListInstancesResponse$Type();
// @generated message type with reflection information, may provide speed optimized methods
class InstanceList$Type extends MessageType<InstanceList> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.InstanceList", [
      {
        no: 1,
        name: "instances",
        kind: "message",
        repeat: 1 /*RepeatType.PACKED*/,
        T: () => InstanceInfo,
      },
    ]);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.InstanceList
 */
export const InstanceList = new InstanceList$Type();
// @generated message type with reflection information, may provide speed optimized methods
class StartInstanceRequest$Type extends MessageType<StartInstanceRequest> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.StartInstanceRequest", [
      { no: 1, name: "id", kind: "scalar", T: 9 /*ScalarType.STRING*/ },
    ]);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.StartInstanceRequest
 */
export const StartInstanceRequest = new StartInstanceRequest$Type();
// @generated message type with reflection information, may provide speed optimized methods
class StartInstanceResponse$Type extends MessageType<StartInstanceResponse> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.StartInstanceResponse", [
      {
        no: 1,
        name: "success",
        kind: "message",
        oneof: "result",
        T: () => Empty,
      },
      {
        no: 2,
        name: "problem",
        kind: "message",
        oneof: "result",
        T: () => ProblemDetails,
      },
    ]);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.StartInstanceResponse
 */
export const StartInstanceResponse = new StartInstanceResponse$Type();
// @generated message type with reflection information, may provide speed optimized methods
class StopInstanceRequest$Type extends MessageType<StopInstanceRequest> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.StopInstanceRequest", [
      { no: 1, name: "id", kind: "scalar", T: 9 /*ScalarType.STRING*/ },
    ]);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.StopInstanceRequest
 */
export const StopInstanceRequest = new StopInstanceRequest$Type();
// @generated message type with reflection information, may provide speed optimized methods
class StopInstanceResponse$Type extends MessageType<StopInstanceResponse> {
  constructor() {
    super("francenuage.fr.api.controlplane.v1.StopInstanceResponse", [
      {
        no: 1,
        name: "success",
        kind: "message",
        oneof: "result",
        T: () => Empty,
      },
      {
        no: 2,
        name: "problem",
        kind: "message",
        oneof: "result",
        T: () => ProblemDetails,
      },
    ]);
  }
}
/**
 * @generated MessageType for protobuf message francenuage.fr.api.controlplane.v1.StopInstanceResponse
 */
export const StopInstanceResponse = new StopInstanceResponse$Type();
/**
 * @generated ServiceType for protobuf service francenuage.fr.api.controlplane.v1.Hypervisor
 */
export const Hypervisor = new ServiceType(
  "francenuage.fr.api.controlplane.v1.Hypervisor",
  [
    {
      name: "ListInstances",
      options: {},
      I: ListInstancesRequest,
      O: ListInstancesResponse,
    },
    {
      name: "StartInstance",
      options: {},
      I: StartInstanceRequest,
      O: StartInstanceResponse,
    },
    {
      name: "StopInstance",
      options: {},
      I: StopInstanceRequest,
      O: StopInstanceResponse,
    },
  ],
);
