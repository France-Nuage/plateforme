#!/bin/bash

# Variables
TEMPLATE_ID=1000
TEMPLATE_NAME="debian12-docker-default-template"
CLOUD_IMAGE_URL="https://cloud.debian.org/images/cloud/bookworm/latest/debian-12-genericcloud-amd64.qcow2"
CLOUD_IMAGE_PATH="/var/lib/vz/images/0/debian-12-genericcloud-amd64.qcow2"
CI_CUSTOM_USER_SNIPPET_PATH="/var/lib/vz/snippets/ci-custom-user-default-snippet.yaml"
STORAGE_POOL=$(pvesm status | grep -i ceph | awk '{print $1}')
SNIPPETS_STORAGE="local"
BRIDGE="vmbr0"
TEMPLATE_DEFAULT_GATEWAY=$(ip route show default | awk '/default/ {print $3}') # Passerelle actuelle de l'hôte Proxmox
TEMPLATE_DEFAULT_IP=$(echo $TEMPLATE_DEFAULT_GATEWAY | sed 's/\.[0-9]\+$/\.10/') # IP = Passerelle mais avec 10 sur le dernier octet
TEMPLATE_DEFAULT_CIDR="${TEMPLATE_DEFAULT_IP}$(ip addr show | awk '/inet / && !/127.0.0.1/ {print $2}' | grep -o '/[0-9]*' | head -n 1)" # Concaténation de l'ip et du masque
VM_USER="france-nuage"
SSH_PUBLIC_KEY="ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDngnLZX5rwOc9OHB0dHrOU2Is97O/O76SmKhfiFCiqNcWm+Gs0ngD2ZUiE7wi5sN9CwKX7b/13NFE6gRClIc6iiMEyBhezQQiTp7yVT25x1W4DJCOb6gM64kZ3CSuxztcgN2uHYeAhs6L4ItHI2NuTpBdyKsHc9UkgiRh6F13ZYbHNp5HITz3g05IY+dWpFEiXotNSLaD891gvoQEZjBsFcqXAHrDfb71hEhVlT9AtoLOhfS1wZN1J4wYE0C5Tvf/woPLVHl2FtCcBWxWO/OIg/RV1bvqfQ0cK1je9oAWJpvqn8pglXyNzPSBufhEMaYCPY2kGxZ71BlrHtnIkw9nS8KND3EzJO5uyUjnSbEl7aJmmEjLlOllRHDQ34Jiaki96Rlyvcvpq4xEt/AIPQp/sK+Dd4z9cKuEk22vSVIXpCmbuwzAaORRS2m/gIrux+OZxH68qAncBvYwnZv+l4IfRbGHLRmCcPi48uNPFd49goO2P+LXlAeSp/RQ/iLqc/2B6U4tZyWiluCY/2FzS/9rV79ShkgQ/dCnhjALXWPEh806tu45gb+owHCcNQ/6k2AhgjVEBpUJZsjce2s7JlvhD6c0CnPRJXTdzKAIy3k0AHNzX4kZNA1jBX1oQKe8gdGPbMs6OV9/5SGpBAELfhR+gxtXVQUpvjuZk5K2rxS3pZQ== ssh@france-nuage.fr"

# Étape 0 : Vérifier le présence d'un stockage CEPH
if [[ -z "$STORAGE_POOL" ]]; then # Si la var est nulle
  echo "Erreur : Aucun stockage Ceph trouvé. Veuillez vérifier votre configuration de stockage."
  exit 1
fi

# Étape 1 : Vérifier si le dossier pour l'image cloud-init existe, sinon le créer
CLOUD_IMAGE_DIR=$(dirname "$CLOUD_IMAGE_PATH")
if [[ ! -d "$CLOUD_IMAGE_DIR" ]]; then # Vérifie que le chemin de la var pointe vers un répertoire
  echo "Le dossier $CLOUD_IMAGE_DIR n'existe pas. Création du dossier..."
  mkdir -p "$CLOUD_IMAGE_DIR" || { echo "Erreur : Échec de la création du dossier."; exit 1; }
fi

# Étape 2 : Télécharger l'image cloud-init si elle n'existe pas
if [[ ! -f "$CLOUD_IMAGE_PATH" ]]; then # Vérifie que le chemin de la var pointe vers un fichier
  echo "Téléchargement de l'image Debian 12 Cloud-Init..."
  wget -O "$CLOUD_IMAGE_PATH" "$CLOUD_IMAGE_URL" || { echo "Erreur : Échec du téléchargement."; exit 1; }
fi

# Étape 3 : Activation du stockage des snippets
if ! grep -q "snippets" /etc/pve/storage.cfg; then # Si la string n'est pas trouvé dnas le fichier
    pvesm set "$SNIPPETS_STORAGE" --content rootdir,images,iso,vztmpl,backup,snippets
    echo "$SNIPPETS_STORAGE is now configured to accept snippets."
fi
grep -q "snippets" /etc/pve/storage.cfg || { echo "Erreur : Échec de la configuration des snippets."; exit 1; }

# Étape 4 : Vérifier si le template existe déjà
if qm list | grep -q "$TEMPLATE_ID"; then
  echo "Le modèle avec l'ID $TEMPLATE_ID existe déjà."
  exit 0
fi

# Étape 5 : Créer le modèle cloud-init
echo "Création et configuration de la VM à templater..."
qm create "$TEMPLATE_ID" --name "debian-12-cloudinit-template" --memory 1024 --net0 virtio,bridge="$BRIDGE" --cores 1 --sockets 1
qm importdisk "$TEMPLATE_ID" "$CLOUD_IMAGE_PATH" "$STORAGE_POOL"
qm set "$TEMPLATE_ID" --scsihw virtio-scsi-pci --scsi0 "$STORAGE_POOL:vm-$TEMPLATE_ID-disk-0"
qm set "$TEMPLATE_ID" --ide2 "$STORAGE_POOL:cloudinit"
qm set "$TEMPLATE_ID" --boot c --bootdisk scsi0
qm set "$TEMPLATE_ID" --serial0 socket --vga serial0
qm set "$TEMPLATE_ID" --ipconfig0 ip=$TEMPLATE_DEFAULT_CIDR,gw=$TEMPLATE_DEFAULT_GATEWAY
qm set "$TEMPLATE_ID" --name "$TEMPLATE_NAME"
qm set "$TEMPLATE_ID" --cpu x86-64-v2-AES

# Créer un fichier de script cloud-init pour installer Docker
echo "Création d'un fichier de script cloud-init pour installer Docker..."
cat << EOF > $CI_CUSTOM_USER_SNIPPET_PATH
#cloud-config
users:
  - name: ${VM_USER}
    gecos: "${VM_USER}"
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    ssh-authorized-keys:
      - ${SSH_PUBLIC_KEY}

runcmd:
  - apt-get update
  - apt-get install -y ca-certificates curl gnupg lsb-release
  - mkdir -p /etc/apt/keyrings
  - curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  - echo "deb [arch=\$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \$(lsb_release -cs) stable" > /etc/apt/sources.list.d/docker.list
  - apt-get update
  - apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
  - usermod -aG docker ${VM_USER}
  - systemctl enable --now docker
EOF

# Configurer les bon droits sur le snippet
chmod 644 "$CI_CUSTOM_USER_SNIPPET_PATH"

# Configurer cloud-init pour inclure le script
qm set "$TEMPLATE_ID" --cicustom "user=$SNIPPETS_STORAGE:snippets/ci-custom-user-snippet.yaml"

# Création réelle du template
echo "Création du template à partir de la VM..."
qm template "$TEMPLATE_ID"

echo "Modèle Cloud-Init Debian 12 créé avec succès avec l'ID $TEMPLATE_ID."

