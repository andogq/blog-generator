#!/bin/bash

echo "Deploying to production..."

cloudflare_request() {
    method="$1"
    endpoint="$2"
    body="$3"

    curl -H "Authorization: Bearer $CLOUDFLARE_TOKEN" -X "$method" "$CLOUDFLARE_TARGET/$endpoint" -d "$body"
}

# Deploy worker
cd worker

# Remove existing secrets (not working due to lack of non-interactive support)
#wrangler secret list | jq -r '.[].name' | while read var ; do
#    echo "y" | wrangler secret delete "$var"
#done

# Parse .env file for secrets
cat ../.env.dev | grep -v '^\w*$' | grep -v '^#.*$' | while read var ; do
    key=${var%=*}
    value=${var#*=}

    if [[ $key == PUBLIC_* ]] ; then
        # Add to wrangler.toml file
        cloudflare_request "PUT" "/accounts/workers"
    else
        # Add secret to wrangler
        echo "$value" | wrangler secret put "$key"
    fi
done

#public_vars=$(echo "$vars" | grep '^PUBLIC_')
#secret_vars=$(echo "$vars" | grep -v '^PUBLIC_')

cd ..
