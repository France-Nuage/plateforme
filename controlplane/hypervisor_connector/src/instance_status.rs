//! Module providing status representations for virtualized instances.
//!
//! This module contains enumerations and types that represent the current state
//! of instances across different virtualization platforms. These abstractions allow
//! for consistent handling of instance states regardless of the underlying hypervisor.

/// Represents the operational state of an instance across different hypervisors.
///
/// This enum provides a generic abstraction over the various status values that might
/// be used by different virtualization platforms, mapping them to a consistent set
/// of states that can be used throughout the application.
pub enum InstanceStatus {
    /// Instance is active and operational.
    ///
    /// The instance is running, consuming resources, and able to process workloads.
    /// This corresponds to the RUNNING (1) state in the protocol specification.
    Running,

    /// Instance is inactive.
    ///
    /// The instance exists but is not currently executing. Its configuration is preserved,
    /// but it is not consuming compute resources or able to process workloads.
    /// This corresponds to the STOPPED (2) state in the protocol specification.
    Stopped,
}
