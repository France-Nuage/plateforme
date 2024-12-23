#!/bin/bash

# Variables template
TEMPLATE_ID=1001
TEMPLATE_NAME="debian12-docker-growthbook-template"
CLOUD_IMAGE_URL="https://cloud.debian.org/images/cloud/bookworm/latest/debian-12-genericcloud-amd64.qcow2"
CLOUD_IMAGE_PATH="/var/lib/vz/images/0/debian-12-genericcloud-amd64.qcow2"
CI_CUSTOM_USER_SNIPPET_PATH="/var/lib/vz/snippets/ci-custom-user-snippet.yaml"
STORAGE_POOL=$(pvesm status | grep -i ceph | awk '{print $1}')
SNIPPETS_STORAGE="local"
BRIDGE="vmbr0"
TEMPLATE_DEFAULT_GATEWAY=$(ip route show default | awk '/default/ {print $3}')
TEMPLATE_DEFAULT_IP=$(echo $TEMPLATE_DEFAULT_GATEWAY | sed 's/\.[0-9]\+$/\.10/')
TEMPLATE_DEFAULT_CIDR="${TEMPLATE_DEFAULT_IP}$(ip addr show | awk '/inet / && !/127.0.0.1/ {print $2}' | grep -o '/[0-9]*' | head -n 1)"
VM_USER="france-nuage"
SSH_PUBLIC_KEY="ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDngnLZX5rwOc9OHB0dHrOU2Is97O/O76SmKhfiFCiqNcWm+Gs0ngD2ZUiE7wi5sN9CwKX7b/13NFE6gRClIc6iiMEyBhezQQiTp7yVT25x1W4DJCOb6gM64kZ3CSuxztcgN2uHYeAhs6L4ItHI2NuTpBdyKsHc9UkgiRh6F13ZYbHNp5HITz3g05IY+dWpFEiXotNSLaD891gvoQEZjBsFcqXAHrDfb71hEhVlT9AtoLOhfS1wZN1J4wYE0C5Tvf/woPLVHl2FtCcBWxWO/OIg/RV1bvqfQ0cK1je9oAWJpvqn8pglXyNzPSBufhEMaYCPY2kGxZ71BlrHtnIkw9nS8KND3EzJO5uyUjnSbEl7aJmmEjLlOllRHDQ34Jiaki96Rlyvcvpq4xEt/AIPQp/sK+Dd4z9cKuEk22vSVIXpCmbuwzAaORRS2m/gIrux+OZxH68qAncBvYwnZv+l4IfRbGHLRmCcPi48uNPFd49goO2P+LXlAeSp/RQ/iLqc/2B6U4tZyWiluCY/2FzS/9rV79ShkgQ/dCnhjALXWPEh806tu45gb+owHCcNQ/6k2AhgjVEBpUJZsjce2s7JlvhD6c0CnPRJXTdzKAIy3k0AHNzX4kZNA1jBX1oQKe8gdGPbMs6OV9/5SGpBAELfhR+gxtXVQUpvjuZk5K2rxS3pZQ== ssh@france-nuage.fr"

# Variables docker compose
DOCKER_APP_PATH="${DOCKER_APP_PATH:-/home/${VM_USER}/docker}"
DOCKER_COMPOSE_PATH="${APP_DOCKER_COMPOSE_PATH:-${DOCKER_APP_PATH}/docker-compose.yaml}"
MONGO_USER="${MONGO_USER:-france-nuage}"
MONGO_PASSWORD="${MONGO_PASSWORD:$(openssl rand -base64 32)}"
JWT_SECRET="${JWT_SECRET:$(openssl rand -base64 32)}"
APP_DOMAIN="${APP_DOMAIN:-france-nuage.fr}"
APP_UI_PORT="${APP_UI_PORT:-3000}"
APP_API_PORT="${APP_API_PORT:-3100}"
GROWTHBOOK_UPLOADS="/home/${VM_USER}/docker/volume/growthbook/uploads"
MONGODB_DATA="/home/${VM_USER}/docker/volume/mongodb/mongodata"

# Étape 0 : Vérifier la présence d'un stockage CEPH
if [[ -z "$STORAGE_POOL" ]]; then
    echo "Erreur : Aucun stockage Ceph trouvé. Veuillez vérifier votre configuration de stockage."
    exit 1
fi

# Étape 1 : Vérifier si le dossier pour l'image cloud-init existe, sinon le créer
CLOUD_IMAGE_DIR=$(dirname "$CLOUD_IMAGE_PATH")
if [[ ! -d "$CLOUD_IMAGE_DIR" ]]; then
    echo "Le dossier $CLOUD_IMAGE_DIR n'existe pas. Création du dossier..."
    mkdir -p "$CLOUD_IMAGE_DIR" || { echo "Erreur : Échec de la création du dossier."; exit 1; }
fi

# Étape 2 : Télécharger l'image cloud-init si elle n'existe pas
if [[ ! -f "$CLOUD_IMAGE_PATH" ]]; then
    echo "Téléchargement de l'image Debian 12 Cloud-Init..."
    wget -O "$CLOUD_IMAGE_PATH" "$CLOUD_IMAGE_URL" || { echo "Erreur : Échec du téléchargement."; exit 1; }
fi

# Étape 3 : Activation du stockage des snippets
if ! grep -q "snippets" /etc/pve/storage.cfg; then
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
qm resize "$TEMPLATE_ID" scsi0 +5G # Espace insuffisant pour growthbook

# Créer le fichier de script cloud-init incluant docker-compose
echo "Création du fichier cloud-init avec Docker et docker-compose..."
cat << EOF > $CI_CUSTOM_USER_SNIPPET_PATH
#cloud-config
users:
  - name: ${VM_USER}
    gecos: "${VM_USER}"
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    ssh-authorized-keys:
      - ${SSH_PUBLIC_KEY}

write_files:
  - path: ${DOCKER_COMPOSE_PATH}
    permissions: '0644'
    content: |
      services:
        growthbook:
          depends_on:
            - mongo
          environment:
            - MONGODB_URI=mongodb://${MONGO_USER}:${MONGO_PASSWORD}@mongo:27017/growthbook?authSource=admin
            - APP_ORIGIN=https://growthbook-ui.${APP_DOMAIN}
            - API_HOST=https://growthbook-api.${APP_DOMAIN}
            - JWT_SECRET=${JWT_SECRET}
          image: growthbook/growthbook:latest
          ports:
            - '${APP_UI_PORT}:3000'
            - '${APP_API_PORT}:3100'
          restart: always
          volumes:
            - ${GROWTHBOOK_UPLOADS}:/usr/local/src/app/packages/back-end/uploads
        mongo:
          environment:
            - MONGO_INITDB_ROOT_USERNAME=${MONGO_USER}
            - MONGO_INITDB_ROOT_PASSWORD=${MONGO_PASSWORD}
          image: mongo:latest
          volumes:
            - ${MONGODB_DATA}:/data/db

  - path: /etc/systemd/system/docker-compose-growthbook.service
    permissions: '0644'
    content: |
      [Unit]
      Description=Docker Compose Growthbook Application
      Requires=docker.service
      After=docker.service network.target
      StartLimitIntervalSec=60
      StartLimitBurst=3

      [Service]
      Type=oneshot
      RemainAfterExit=yes
      WorkingDirectory=${DOCKER_APP_PATH}
      User=${VM_USER}
      Group=docker
      ExecStartPre=/bin/sleep 10
      ExecStart=/usr/bin/docker compose -f ${DOCKER_COMPOSE_PATH} up -d
      ExecStop=/usr/bin/docker compose -f ${DOCKER_COMPOSE_PATH} down
      Restart=on-failure
      RestartSec=10

      [Install]
      WantedBy=multi-user.target

runcmd:
  # Installation de Docker
  - apt-get update
  - apt-get install -y ca-certificates curl gnupg lsb-release
  - mkdir -p /etc/apt/keyrings
  - curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  - echo "deb [arch=\$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \$(lsb_release -cs) stable" > /etc/apt/sources.list.d/docker.list
  - apt-get update
  - apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
  
  # Configuration Docker
  - usermod -aG docker ${VM_USER}
  - systemctl enable docker
  - systemctl start docker
  - sleep 10  # Attendre que Docker soit bien démarré
  
  # Préparation des répertoires
  - mkdir -p ${DOCKER_APP_PATH}
  - chown -R ${VM_USER}:${VM_USER} ${DOCKER_APP_PATH}
  - mkdir -p ${GROWTHBOOK_UPLOADS} ${MONGODB_DATA}
  - chown -R ${VM_USER}:${VM_USER} ${GROWTHBOOK_UPLOADS} ${MONGODB_DATA}
  
  # Configuration du service
  - systemctl daemon-reload
  - systemctl enable docker-compose-growthbook.service
  - sleep 5  # Attendre que tout soit bien en place
  - systemctl start docker-compose-growthbook.service
EOF

# Configurer les bons droits sur le snippet
chmod 644 "$CI_CUSTOM_USER_SNIPPET_PATH"

# Configurer cloud-init pour inclure le script
qm set "$TEMPLATE_ID" --cicustom "user=$SNIPPETS_STORAGE:snippets/ci-custom-user-snippet.yaml"

# Création réelle du template
echo "Création du template à partir de la VM..."
qm template "$TEMPLATE_ID"
echo "Modèle Cloud-Init Debian 12 créé avec succès avec l'ID $TEMPLATE_ID."