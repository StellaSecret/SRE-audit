#!/bin/bash
# update_premium_emails.sh
# Script pour mettre à jour la liste des emails premium
#
# Usage:
#   ./update_premium_emails.sh
#
# Prérequis:
#   - gh CLI installé et authentifié (gh auth login)
#   - environnement GitHub "build" existant et restreint à la branche main
#     (Settings > Environments)

set -e

REPO="StellaSecret/SRE-audit"
ENVIRONMENT="build"
SECRET_NAME="PREMIUM_EMAILS"
LIST_FILE=".premium_emails.txt"

echo "=== Mise à jour des emails premium ==="
echo ""

# Affiche la liste actuelle (fichier local)
if [ -f "$LIST_FILE" ]; then
  echo "Liste actuelle :"
  cat -n "$LIST_FILE"
  echo ""
else
  echo "Aucune liste locale trouvée. Création..."
  touch "$LIST_FILE"
fi

echo "Édition de la liste (un email par ligne) :"
echo "Sauvegarde puis quitte ton éditeur pour continuer."
echo ""
sleep 1
"${EDITOR:-vim}" "$LIST_FILE"

# Convertit en une seule ligne séparée par des virgules, lowercase, sans espaces
EMAILS=$(grep -v '^#' "$LIST_FILE" | grep -v '^$' | tr '[:upper:]' '[:lower:]' | tr -d ' ' | paste -sd ',' -)

if [ -z "$EMAILS" ]; then
  echo "❌ Liste vide, annulation."
  exit 1
fi

echo ""
echo "Emails qui seront enregistrés :"
echo "$EMAILS" | tr ',' '\n' | nl
echo ""

read -p "Confirmer la mise à jour du secret GitHub ? (o/N) " confirm
if [[ "$confirm" != "o" && "$confirm" != "O" ]]; then
  echo "Annulé."
  exit 0
fi

# Met à jour le secret de l'environnement "build" (utilisé par web-build-prod)
gh secret set "$SECRET_NAME" --env "$ENVIRONMENT" --body "$EMAILS" --repo "$REPO"

echo ""
echo "✓ Secret '$SECRET_NAME' mis à jour dans l'environnement '$ENVIRONMENT' sur $REPO"
echo "✓ Liste locale sauvegardée dans $LIST_FILE"
echo ""
echo "⚠️  .premium_emails.txt est ignoré par git : ne le commite pas (emails en clair)."
echo "⚠️  PREMIUM_EMAILS est compilé dans le WASM public : ce n'est pas une donnée"
echo "    confidentielle, seulement un filtre d'usage."
