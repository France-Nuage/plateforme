# Soft delete instances removed from hypervisors

## Summary

The synchronizer doesn't handle instances that get deleted on Proxmox. They remain in the Control Plane forever, even though they don't exist on the hypervisor anymore.

Add soft delete support using fabrique's `#[fabrique(soft_delete)]` mechanism to mark these instances as deleted.

## Motivation

Right now the synchronizer does two things:
- Creates instances that exist on Proxmox but not in our DB
- Updates instances that exist in both places

What it doesn't do: mark instances as deleted when they disappear from Proxmox. This means deleted VMs stick around in the Control Plane indefinitely.

We need to handle this case, but be careful not to soft delete everything when a hypervisor temporarily goes offline.

## Proposed changes

Add `deleted_at` column to instances and use it to track deleted instances. After each successful sync with a hypervisor, compare the list of instances we got from Proxmox with what's in the database for that hypervisor, and soft delete anything that's missing.


