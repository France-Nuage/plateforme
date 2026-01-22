#!/usr/bin/env tsx

/**
 * List all instances from the production France Nuage cloud.
 * Useful for finding PROXMOX_DIRTY_ID to reuse an existing instance
 * instead of cloning a template.
 */

import { configureResolver, transport, ServiceMode, InstanceStatus } from '@france-nuage/sdk';

const removeBearer = (token: string): string => token.startsWith('Bearer ') ? token.slice(7) : token;

async function main() {
  const token = process.env.PRODUCTION_CONTROLPLANE_TOKEN;
  if (!token) {
    console.error('❌ PRODUCTION_CONTROLPLANE_TOKEN not set');
    process.exit(1);
  }

  const controlplaneUrl = process.env.PRODUCTION_CONTROLPLANE_URL || 'https://controlplane.france-nuage.fr';

  console.log(`\n🔗 Connexion à ${controlplaneUrl}...\n`);

  const services = configureResolver(transport(controlplaneUrl, removeBearer(token)))[ServiceMode.Rpc];

  console.log('📋 Récupération des instances...\n');

  const instances = await services.instance.list();

  // Sort by status and name
  const running = instances.filter(i => i.status === InstanceStatus.Running).sort((a, b) => a.name.localeCompare(b.name));
  const stopped = instances.filter(i => i.status === InstanceStatus.Stopped).sort((a, b) => a.name.localeCompare(b.name));
  const other = instances.filter(i => i.status !== InstanceStatus.Running && i.status !== InstanceStatus.Stopped);

  console.log(`✅ ${instances.length} instance(s) trouvée(s)\n`);

  // Print running instances
  if (running.length > 0) {
    console.log('🟢 INSTANCES EN COURS D\'EXÉCUTION (candidates pour PROXMOX_DIRTY_ID)');
    console.log('═'.repeat(80));
    running.forEach((inst, idx) => {
      console.log(`ID:         ${inst.id}`);
      console.log(`  Nom:        ${inst.name}`);
      console.log(`  Status:     ${inst.status}`);
      console.log(`  IP:         ${inst.ipV4 || 'N/A'}`);
      console.log(`  Hypervisor: ${inst.hypervisorId}`);
      console.log(`  Projet:     ${inst.projectId}`);
      console.log(`  CPU:        ${inst.cpuUsagePercent.toFixed(1)}% (${inst.maxCpuCores} cores)`);
      console.log(`  RAM:        ${(inst.memoryUsageBytes / (1024 ** 3)).toFixed(2)} GB / ${(inst.maxMemoryBytes / (1024 ** 3)).toFixed(2)} GB`);
      console.log(`  Disk:       ${(inst.diskUsageBytes / (1024 ** 3)).toFixed(0)} B / ${(inst.maxDiskBytes / (1024 ** 3)).toFixed(2)} GB`);
      console.log(`  Créée:      ${new Date(inst.createdAt).toLocaleString('fr-FR')}`);
      if (idx < running.length - 1) console.log('  ──────────────────────────────────────────────────────────────────────────────');
    });
    console.log('\n💡 Pour utiliser une de ces instances, ajoutez dans votre .env :');
    console.log(`   PROXMOX_DIRTY_ID=${running[0].id}\n`);
  }

  // Print other status
  if (other.length > 0 || stopped.length > 0) {
    console.log('📊 AUTRES INSTANCES');
    console.log('═'.repeat(80));

    const statusGroups = new Map<string, typeof other>();
    [...other, ...stopped].forEach(inst => {
      const status = inst.status;
      if (!statusGroups.has(status)) statusGroups.set(status, []);
      statusGroups.get(status)!.push(inst);
    });

    statusGroups.forEach((insts, status) => {
      console.log(`\n🔴 ${status} (${insts.length})`);
      insts.slice(0, 10).forEach(inst => {
        console.log(`  - ${inst.name} (${inst.id})`);
      });
      if (insts.length > 10) {
        console.log(`  ... et ${insts.length - 10} autres`);
      }
    });
    console.log();
  }

  // Print templates
  const templates = instances.filter(i => i.name.match(/^pve\d+-test\d+-template|^Copy-of-VM-pve/i));
  if (templates.length > 0) {
    console.log('📦 TEMPLATES PROXMOX');
    console.log('═'.repeat(80));
    templates.forEach((inst, idx) => {
      console.log(`ID:      ${inst.id}`);
      console.log(`  Nom:     ${inst.name}`);
      console.log(`  Status:  ${inst.status}`);
      if (idx < templates.length - 1) console.log('  ──────────────────────────────────────────────────────────────────────────────');
    });
    console.log(`\n⚠️  Ces templates sont utilisés pour cloner de nouveaux hypervisors.`);
    console.log(`   Si le clone échoue, utilisez une instance Running ci-dessus.\n`);
  }
}

main().catch(err => {
  console.error('❌ Erreur:', err.message);
  process.exit(1);
});
