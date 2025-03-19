#!/bin/bash

# Configuration
URL_LIST_GIT="https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/raw/master/apps/agent/disk-image-sync/disk-images-url-to-download-locally.txt?ref_type=heads"
LOCAL_DIR="/var/lib/vz/images/0"  # Répertoire où stocker les images sur Proxmox
MAX_STORAGE_GB=15  # Limite maximale en Go
LOG_FILE="/var/log/cloud-image-sync.log"
TMP_LIST="/tmp/cloud-images-list.txt"
TEMP_DIR="/tmp/cloud-image-sync"

# Création des répertoires si nécessaires
mkdir -p "$LOCAL_DIR"
mkdir -p "$TEMP_DIR"

# Fonction de logging
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

# Vérifier l'espace disque utilisé par le répertoire en Go
check_disk_usage() {
    # Get size in bytes and convert to GB
    local size_bytes=$(du -sb "$LOCAL_DIR" | awk '{print $1}')
    echo "scale=2; $size_bytes / 1024 / 1024 / 1024" | bc
}

# Obtenir la taille d'un fichier en Go
get_file_size_gb() {
    local file="$1"
    if [ -f "$file" ]; then
        local size_bytes=$(stat -c%s "$file")
        echo "scale=2; $size_bytes / 1024 / 1024 / 1024" | bc
    else
        echo "0"
    fi
}

# Télécharger la liste depuis GitLab
log "Récupération de la liste d'URL depuis GitLab..."
curl -s "$URL_LIST_GIT" -o "$TMP_LIST"

if [ ! -f "$TMP_LIST" ]; then
    log "Erreur: Impossible de récupérer la liste d'URL"
    exit 1
fi

# Vérifier l'espace disque actuel
CURRENT_USAGE=$(check_disk_usage)
log "Utilisation actuelle: ${CURRENT_USAGE}G / ${MAX_STORAGE_GB}G maximum"

# Parcourir chaque URL dans la liste
while read -r image_url; do
    # Ignorer les lignes vides ou commentées
    if [ -z "$image_url" ] || [[ "$image_url" == \#* ]]; then
        continue
    fi
    
    # Extraire le nom du fichier de l'URL
    filename=$(basename "$image_url")
    local_path="$LOCAL_DIR/$filename"
    temp_path="$TEMP_DIR/$filename"
    
    # Vérifier si le téléchargement est déjà en cours (via un fichier .lock)
    lock_file="${TEMP_DIR}/${filename}.lock"
    if [ -f "$lock_file" ]; then
        lock_age=$(($(date +%s) - $(stat -c %Y "$lock_file")))
        # Si le verrou existe depuis plus d'une heure, on considère qu'il est obsolète
        if [ $lock_age -gt 3600 ]; then
            log "Verrou obsolète pour $filename (âge: ${lock_age}s), suppression..."
            rm -f "$lock_file"
        else
            log "Téléchargement déjà en cours pour $filename, ignoré"
            continue
        fi
    fi
    
    # Vérifier si l'image existe déjà
    if [ -f "$local_path" ]; then
        log "L'image $filename existe déjà localement"
        
        # Pour les URLs de Debian et Ubuntu, utiliser un traitement spécial
        if [[ "$image_url" == *cloud.debian.org* ]] || [[ "$image_url" == *ubuntu.com* ]]; then
            # Pour Debian, vérifier la date dans l'URL
            if [[ "$image_url" == *cloud.debian.org* ]]; then
                # Extraire la date de l'URL
                url_date=$(echo "$image_url" | grep -o -E "[0-9]{8}-[0-9]{4}")
                local_file_basename=$(basename "$local_path")
                filename_date=$(echo "$local_file_basename" | grep -o -E "[0-9]{8}-[0-9]{4}")
                
                # Vérifier si nous avons une date à comparer
                if [ -n "$url_date" ] && [ -n "$filename_date" ]; then
                    # Si les dates sont différentes, il faut télécharger
                    if [ "$url_date" != "$filename_date" ]; then
                        log "Date différente dans l'URL ($url_date) et le fichier local ($filename_date), téléchargement de la nouvelle version"
                        # Continuer avec le téléchargement
                    else
                        # Même date, pas besoin de télécharger
                        log "L'image $filename est à jour basé sur la date dans l'URL, pas besoin de la télécharger"
                        continue
                    fi
                else
                    # Pas de date à comparer, on essaie une autre méthode
                    log "Impossible de comparer les dates par le nom de fichier pour $filename"
                    
                    # On essaie une comparaison de taille
                    local_size=$(stat -c%s "$local_path")
                    
                    # On fait une requête pour obtenir la taille du fichier distant
                    if command -v wget >/dev/null 2>&1; then
                        remote_size=$(wget --spider --server-response "$image_url" 2>&1 | grep -i "content-length" | awk '{print $2}' | tr -d '\r\n')
                    fi
                    
                    if [ -n "$remote_size" ] && [ "$remote_size" -eq "$local_size" ]; then
                        log "Taille identique pour $filename, pas besoin de télécharger"
                        continue
                    fi
                    
                    # Si on ne peut pas comparer les tailles, on télécharge
                    log "Impossible de vérifier si $filename est à jour, téléchargement..."
                fi
            # Pour Ubuntu, comparer les tailles
            elif [[ "$image_url" == *ubuntu.com* ]]; then
                log "Vérification de l'image Ubuntu avec HEAD request..."
                local_size=$(stat -c%s "$local_path")
                
                # Essayer d'obtenir la taille avec wget
                if command -v wget >/dev/null 2>&1; then
                    remote_size=$(wget --spider --server-response "$image_url" 2>&1 | grep -i "content-length" | awk '{print $2}' | tr -d '\r\n')
                fi
                
                if [ -n "$remote_size" ] && [ "$remote_size" -eq "$local_size" ]; then
                    log "Taille identique pour $filename, pas besoin de télécharger"
                    continue
                else
                    log "Taille différente ou non vérifiable pour $filename, téléchargement de la nouvelle version"
                fi
            fi
        # Pour les autres URLs, utiliser l'approche standard
        else
            # Vérifier si l'image a été modifiée (en utilisant l'en-tête HTTP)
            remote_last_modified=$(curl -s -I "$image_url" | grep -i "last-modified" | awk '{$1=""; print $0}' | xargs)
            
            if [ -n "$remote_last_modified" ]; then
                # Comparer avec la date locale
                if touch -d "$remote_last_modified" /tmp/remote_date 2>/dev/null; then
                    if [ /tmp/remote_date -nt "$local_path" ]; then
                        log "Une nouvelle version de $filename est disponible, téléchargement..."
                    else
                        log "L'image $filename est à jour, pas besoin de la télécharger"
                        continue
                    fi
                fi
            fi
        fi
    fi
    
    # Créer un fichier de verrou pour indiquer que le téléchargement est en cours
    touch "$lock_file"
    
    # Calculer la taille potentielle pour la gestion de l'espace
    log "Vérification de la taille pour $filename..."
    
    # Essayer d'obtenir la taille avec wget (plus fiable que curl pour certains serveurs)
    image_size=""
    if command -v wget >/dev/null 2>&1; then
        image_size=$(wget --spider --server-response "$image_url" 2>&1 | grep -i "content-length" | awk '{print $2}' | tr -d '\r\n')
    fi
    
    # Si wget échoue, essayer avec curl
    if [ -z "$image_size" ] && curl -sI "$image_url" | grep -q "200 OK"; then
        image_size=$(curl -sI "$image_url" | grep -i "content-length" | awk '{print $2}' | tr -d '\r\n')
    fi
    
    # Si une taille a été trouvée, vérifier l'espace
    if [ -n "$image_size" ] && [ "$image_size" -gt 0 ]; then
        image_size_gb=$(echo "scale=2; $image_size / 1024 / 1024 / 1024" | bc)
        log "Taille estimée de $filename: ${image_size_gb}G"
        
        # Recalculer l'utilisation actuelle pour s'assurer qu'elle est correcte
        CURRENT_USAGE=$(check_disk_usage)
        
        # Vérifier si l'ajout de cette image dépasserait la limite
        new_total=$(echo "scale=2; $CURRENT_USAGE + $image_size_gb" | bc)
        if (( $(echo "$new_total > $MAX_STORAGE_GB" | bc -l) )); then
            log "Avertissement: Télécharger $filename ($image_size_gb G) dépasserait la limite de ${MAX_STORAGE_GB}G (nouveau total: ${new_total}G)"
            
            # Libérer de l'espace
            space_needed=$(echo "scale=2; $new_total - $MAX_STORAGE_GB" | bc)
            log "Besoin de libérer ${space_needed}G d'espace"
            
            # Trouver les fichiers les plus anciens
            old_files=$(find "$LOCAL_DIR" -type f -printf "%T@ %p\n" | sort -n | awk '{print $2}')
            
            for old_file in $old_files; do
                if [ "$old_file" != "$local_path" ]; then
                    old_file_size_gb=$(get_file_size_gb "$old_file")
                    old_file_name=$(basename "$old_file")
                    
                    log "Suppression de l'ancien fichier: $old_file_name (${old_file_size_gb}G)"
                    rm -f "$old_file"
                    
                    # Recalculer l'espace
                    CURRENT_USAGE=$(check_disk_usage)
                    new_total=$(echo "scale=2; $CURRENT_USAGE + $image_size_gb" | bc)
                    
                    if (( $(echo "$new_total <= $MAX_STORAGE_GB" | bc -l) )); then
                        log "Espace suffisant libéré ($CURRENT_USAGE G utilisés), poursuite du téléchargement"
                        break
                    fi
                fi
            done
        fi
    else
        log "Impossible de déterminer la taille de $filename à l'avance"
    fi
    
    # Télécharger l'image avec un compteur de progression
    log "Téléchargement de $filename en cours..."
    
    # Utiliser wget avec barre de progression dans le journal si disponible
    if command -v wget >/dev/null 2>&1; then
        if wget -q --show-progress --progress=bar:force:noscroll -O "$temp_path" "$image_url" 2>&1 | tee -a "$LOG_FILE"; then
            download_success=true
        else
            download_success=false
        fi
    else
        # Fallback à curl si wget n'est pas disponible
        if curl -# -o "$temp_path" "$image_url" 2>&1 | tee -a "$LOG_FILE"; then
            download_success=true
        else
            download_success=false
        fi
    fi
    
    # Vérifier si le téléchargement a réussi
    if [ -f "$temp_path" ] && [ -s "$temp_path" ]; then
        # Obtenir la taille réelle du fichier téléchargé
        downloaded_size_gb=$(get_file_size_gb "$temp_path")
        
        # Mise à jour de l'utilisation actuelle
        CURRENT_USAGE=$(check_disk_usage)
        
        # Vérifier si nous avons assez d'espace pour déplacer le fichier
        new_total=$(echo "scale=2; $CURRENT_USAGE + $downloaded_size_gb" | bc)
        
        if (( $(echo "$new_total > $MAX_STORAGE_GB" | bc -l) )); then
            log "Avertissement: L'image téléchargée $filename (${downloaded_size_gb}G) dépasserait la limite de ${MAX_STORAGE_GB}G"
            
            # Libérer de l'espace pour le fichier téléchargé
            space_needed=$(echo "scale=2; $new_total - $MAX_STORAGE_GB" | bc)
            log "Besoin de libérer ${space_needed}G d'espace"
            
            old_files=$(find "$LOCAL_DIR" -type f -printf "%T@ %p\n" | sort -n | awk '{print $2}')
            
            for old_file in $old_files; do
                if [ "$old_file" != "$local_path" ]; then
                    old_file_size_gb=$(get_file_size_gb "$old_file")
                    old_file_name=$(basename "$old_file")
                    
                    log "Suppression de l'ancien fichier: $old_file_name (${old_file_size_gb}G)"
                    rm -f "$old_file"
                    
                    # Recalculer l'espace
                    CURRENT_USAGE=$(check_disk_usage)
                    new_total=$(echo "scale=2; $CURRENT_USAGE + $downloaded_size_gb" | bc)
                    
                    if (( $(echo "$new_total <= $MAX_STORAGE_GB" | bc -l) )); then
                        log "Espace suffisant libéré"
                        break
                    fi
                fi
            done
        fi
        
        # Déplacer le fichier du répertoire temporaire vers le répertoire final
        mv "$temp_path" "$local_path"
        log "Téléchargement de $filename terminé avec succès (${downloaded_size_gb}G)"
        
        # Mettre à jour l'utilisation actuelle
        CURRENT_USAGE=$(check_disk_usage)
        log "Nouvelle utilisation: ${CURRENT_USAGE}G / ${MAX_STORAGE_GB}G"
    else
        log "Erreur lors du téléchargement de $filename"
        # Supprimer le fichier partiellement téléchargé
        rm -f "$temp_path"
    fi
    
    # Supprimer le fichier de verrou
    rm -f "$lock_file"
    
done < "$TMP_LIST"

# Nettoyer le répertoire temporaire
rm -rf "$TEMP_DIR"/*

log "Synchronisation terminée"
exit 0