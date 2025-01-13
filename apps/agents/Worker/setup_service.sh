#!/bin/bash
#  ______    ___  _____ ______      _____  ____  _        ___
# |      |  /  _]/ ___/|      |    |     ||    || |      /  _]
# |      | /  [_(   \_ |      |    |   __| |  | | |     /  [_
# |_|  |_||    _]\__  ||_|  |_|    |  |_   |  | | |___ |    _]
#   |  |  |   [_ /  \ |  |  |      |   _]  |  | |     ||   [_
#   |  |  |     |\    |  |  |      |  |    |  | |     ||     |
#   |__|  |_____| \___|  |__|      |__|   |____||_____||_____|

# Variables
SERVICE_NAME="metrics-monitor"
SCRIPT_NAME="france-nuage-agent.py"
SCRIPT_SOURCE="$(dirname "$(realpath "$0")")/$SCRIPT_NAME" # Chemin complet vers france-nuage-agent.py
SERVICE_SOURCE="$(dirname "$(realpath "$0")")/metrics-monitor.service" # Chemin complet vers metrics-monitor.service
INSTALL_DIR="/usr/local/bin"
SYSTEMD_DIR="/etc/systemd/system"
SERVICE_TARGET="$SYSTEMD_DIR/$SERVICE_NAME.service"

# Vérification des permissions
if [ "$(id -u)" -ne 0 ]; then
    echo "Ce script doit être exécuté en tant que root ou avec sudo."
    exit 1
fi

echo "Début de l'installation automatique..."

# Étape 1 : Copier le script Python dans /usr/local/bin
echo "Copie du script Python dans $INSTALL_DIR..."
if [ -f "$SCRIPT_SOURCE" ]; then
    cp "$SCRIPT_SOURCE" "$INSTALL_DIR/$SERVICE_NAME.py"
    chmod +x "$INSTALL_DIR/$SERVICE_NAME.py"
    echo "Script copié avec succès."
else
    echo "Erreur : le script Python $SCRIPT_SOURCE n'existe pas."
    exit 1
fi

# Étape 2 : Configurer le fichier service systemd
echo "Configuration du fichier service systemd..."
if [ -f "$SERVICE_SOURCE" ]; then
    cp "$SERVICE_SOURCE" "$SERVICE_TARGET"
    sed -i "s|ExecStart=.*|ExecStart=/usr/bin/python3 $INSTALL_DIR/$SERVICE_NAME.py|g" "$SERVICE_TARGET"
    echo "Fichier service configuré avec succès."
else
    echo "Erreur : le fichier service $SERVICE_SOURCE n'existe pas."
    exit 1
fi

# Étape 3 : Activer et démarrer le service
echo "Activation et démarrage du service $SERVICE_NAME..."
systemctl daemon-reload
systemctl enable "$SERVICE_NAME.service"
systemctl restart "$SERVICE_NAME.service"

# Vérification du statut du service
echo "Vérification du statut du service..."
systemctl is-active --quiet "$SERVICE_NAME.service"
if [ $? -eq 0 ]; then
    echo "Le service $SERVICE_NAME est actif et en cours d'exécution."
else
    echo "Erreur : le service $SERVICE_NAME ne fonctionne pas. Consultez les journaux."
    journalctl -u "$SERVICE_NAME.service"
    exit 1
fi

# Étape 4 : Nettoyage des fichiers temporaires (optionnel)
echo "Nettoyage terminé. Aucun fichier temporaire à supprimer."

echo "Installation automatique terminée avec succès !"
exit 0
