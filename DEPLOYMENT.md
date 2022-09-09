# Deployments

## Cloudflare Worker

**todo...** More steps will be required.

1. Create an API token for Cloudflare, as described in the Worker
`CLOUDFLARE_TOKEN` environment variable section below.

2. Create the following variables. Encrypted variables must be created with
`wrangler secret put <NAME>` followed by the variable value, whilst regular
variables can simply be added to the `wrangler.toml` file in the `[vars]`
section.

 - `CLOUDFLARE_TARGET`
 - `CLOUDFLARE_TOKEN` (encrypted)
 - `CLOUDFLARE_ZONE_ID`
 - `GH_KEY` (encrypted)
 - `SERVER_TOKEN` (encrypted)

3. Deploy the worker with `wrangler publish`. It may take some time to compile
and upload all of the required files.

4. Navigate to the Cloudflare Workers dashboard, and select the specified
worker. Link the worker to the domain in the Custom Domains section, and ensure
that the route `*/*` is added so that it is triggered for every request to that
domain.

5. Once certificates are generated for the domain, it should be possible to
access the worker.

## Web App

The web app is deployed with Vercel. Vercel has automatic deployments from a
linked GitHub repository, so all that is required is to create a new project and
add the required environment variables:

 - `VITE_CNAME_TARGET`
 - `VITE_GITHUB_CLIENT_ID`
 - `WORKER_URL`
 - `GITHUB_CLIENT_SECRET`
 - `WORKER_SECRET_TOKEN`
 - `JWT_SECRET`

A domain can be added to the project by navigating to the domains section,
entering the desired domain and then configuring the required DNS records for
verification.

## Database

The database is deployed on Google Cloud, with the following specifications:

 - vCPUs: 1
 - Memory: 614.4MB
 - SSD Storage: 20GB
 - Location: us-central1-a
 - PostgreSQL Version: 14.4

Currently there is no automation for pushing the Prisma schema to the database,
so it must be done manually as follows:

1. Setup [gcloud CLI](https://cloud.google.com/sdk/docs/downloads-interactive)

2. Download [CloudSQL proxy](https://cloud.google.com/sql/docs/postgres/sql-proxy)

3. Start CloudSQL proxy with the following command, where `instances` matches 
the 'Connection name' string located on the SQL overview page, and `1234` will
be the TCP port that the proxy will be running on:

`./cloud_sql_proxy -instances=github-blog-360902:us-central1:db=tcp:1234`

4. Configure the `DATABASE_URL` environment variable (currently located in
`.env`, but should be moved elsewhere), replacing `password` with the relevant
value:

`DATABASE_URL="postgresql://postgres:password@localhost:1234/postgres"`

5. Push changes with `npx prisma db push`

# Environment Variables

## Prisma

Currently, this file only contains an environment variable for the database
connection, and cloud be adopted into another env file. The database connection
string is only used by the web app, so it should be configured in the same
manner as the `webapp/.env` environment variables.

### `DATABASE_URL`

This is the connection URL required to connect to the database. For example,
`postgresql://tom:tom@localhost:5432/tom?schema=public` for connecting to the
local development database.

The method to generate this variable will depend on the database deployment
method.

#### Google Cloud SQL

**todo...**

## Web App

All of the environment variables in here must be added to Vercel's environment
secret manager for production. For development, the variables can simply be
defined directly in the `webapp/.env` file, which shouldn't be checked into
source control.

### `VITE_GITHUB_CLIENT_ID`

This is the client ID for GitHub OAuth. It should be generated thorugh
[Github's OAuth Apps page](https://github.com/settings/developers), and will be
labelled 'Client ID' on the app dashboard.

Ideally, the development instance should be a different application (including
client ID ect) to the production instance (but this is not the case yet).

### `VITE_CNAME_TARGET`

This is the required target for the CNAME records to point to. It should be
pointing to the running worker instance in Cloudflare for production (eg
`https://worker.thing.ando.gq`), but to the local miniflare instance for dev.

It must be hard coded (whether through a `.env` file or through an environment
secret manager), but it would be good if it could be dynamically generated some
how.

### `WORKER_URL`

This is the target for the worker API, used to make calls to modify KV stores,
manage hostnames ect. It will be the same as the `VITE_CNAME_TARGET` variable
mentioned above, except expressed as a URL (`http`/`https`, port number ect), so
probably could be combined, with an extra variable to specify the worker port
for development.

### `WORKER_SECRET_TOKEN`

This is the token that is used to authenticate the web app backend with the
worker API. It can be any random value, but must be the same as the
`SERVER_TOKEN` value in the `worker/.dev.vars` file.

### `GITHUB_CLIENT_SECRET`

This is the client secret for the GitHub API, and is generated alongside the
aforementioned GitHub Client ID. Many client secrets can be generated for the
same client ID, however only one should currently be needed, since the OAuth
flow only takes place on the web app.

### `JWT_SECRET`

This is the value that is used to sign and verify JWT tokens within the server.
It is a random value, but must be changed with caution, as it will invalidate
all current logged in users, requiring them to log in again.

## Worker

### `GH_KEY`

This is the API token that is used to access the GitHub GraphQL API. It is only
a temporary value, and should be replaced in the future to dynamically use a
specific user's token, in order to spread the usage (and limits) evenly among
all users, and to ensure that if one user uses up the limits, it only affects
them.

### `CLOUDFLARE_TOKEN`

This is the token used to authenticate with the Cloudflare API, and can be
generated from the
[API Tokens page](https://dash.cloudflare.com/profile/api-tokens). A single
token is required for each instance (dev and prod), with permissions for
`Zone.SSL and Certificates` for the zone that the worker is deployed to.

### `CLOUDFLARE_ZONE_ID`

This is the ID of the zone that the worker is deployed to and is used as the
CNAME target. It must matcht he zone that is set in the token above.

### `CLOUDFLARE_TARGET`

This is the target of the Cloudflare API, and should be something along the
lines of `https://api.cloudflare.com/client/v4`. It will be the same for dev and
prod. Since it isn't a sensitive variable, it can be defined in the
`wrangler.toml` file.

### `SERVER_TOKEN`

This is the token that any API requests that aren't for generating a page must
be accompanied with. It should match the `WORKER_SECRET_TOKEN` in the web app.

