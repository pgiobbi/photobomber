#!/bin/sh
# One-time bootstrap to obtain the first Let's Encrypt certificate.
# After this runs successfully, renewal is fully automatic (see compose.yaml).
#
# Prerequisites (must be true BEFORE running):
#   - DNS A/AAAA records for every domain below point to this host.
#   - Ports 80 and 443 are open/forwarded to this host.
#   - The backend is running (nginx proxies /api/ to it).
set -e

# ---- Edit these if needed --------------------------------------------------
domains="photobomber.xyz"   # drop www if it has no DNS record
email="support@photobomber.xyz"
rsa_key_size=4096
staging=0   # set to 1 first to dry-run against LE staging (avoids rate limits)
# ----------------------------------------------------------------------------

cd "$(dirname "$0")"
primary=$(echo "$domains" | cut -d' ' -f1)
live_path="/etc/letsencrypt/live/$primary"

echo "### Seeding a dummy certificate so nginx can start ..."
docker compose run --rm --entrypoint sh certbot -c "\
  mkdir -p $live_path && \
  openssl req -x509 -nodes -newkey rsa:$rsa_key_size -days 1 \
    -keyout $live_path/privkey.pem \
    -out $live_path/fullchain.pem \
    -subj '/CN=localhost'"

echo "### Starting nginx ..."
docker compose up -d nginx

echo "### Removing dummy certificate ..."
docker compose run --rm --entrypoint sh certbot -c "\
  rm -rf /etc/letsencrypt/live/$primary \
         /etc/letsencrypt/archive/$primary \
         /etc/letsencrypt/renewal/$primary.conf"

echo "### Requesting the real certificate for: $domains ..."
domain_args=""
for d in $domains; do domain_args="$domain_args -d $d"; done
staging_arg=""
[ "$staging" != "0" ] && staging_arg="--staging"

docker compose run --rm --entrypoint certbot certbot certonly --webroot -w /var/www/certbot \
  $staging_arg $domain_args \
  --email "$email" \
  --rsa-key-size "$rsa_key_size" \
  --agree-tos --no-eff-email --non-interactive --force-renewal

echo "### Reloading nginx with the real certificate ..."
docker compose exec nginx nginx -s reload

echo "### Bringing up the full stack (nginx + auto-renewing certbot) ..."
docker compose up -d

echo "### Done. https://$primary should be live and will auto-renew."
