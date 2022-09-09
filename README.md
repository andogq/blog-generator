# Available Variables

Any blocks that will contain markdown must be wrapped in `{{{` (instead of the
usual `{{`), as this will ensure that the rendered HTML remains un-escaped.

Some blocks include:
 - `readme`
 - `post.body`
 - `content` (in `core.html`)

## Template Variables

```json
{
    "user": {
        "name": "John Smith",
        "profile_picture": "https://example.com/image.jpg",
        "email": "email@example.com",
        "bio": "Aliquid atque amet voluptate non minima nostrum officiis.",

        "location": "Melbourne, Australia",
        "hireable": true,
        "company": "Some Company",

        "github_profile": "https://github.com/andogq",
        "twitter_profile": https://twitter.com/andogq",

        "followers": 10,
        "following: 10
    },
    "readme": "<h1>About me</h1><p>Lorem ipsum dolor sit amet...</p>", // Rendered contents of user's README
    "posts": [
        {
            "title": "Lorem ipsum",
            "link": "/post/1"
            "labels": [
                {
                    "name": "Some Label",
                    "color": "#ffcc00"
                }
            ],
            "created": "2022-07-10T09:20:00Z",
            "updated": "2022-07-11T13:05:00Z"
        }
    ],
    "pinned": [ // User's pinned repositories
        {
            "name": "Some repo",
            "description": "This is a repo that does something.",
            "languages": "JavaScript, HTML, CSS",
            "homepage": "https://example.com",
            "stargazers": 10,
            "forks": 10,
            "github_url": "https://github.com/andogq/some_repo",
            "uses_custom_image": true,
            "image_url": "https://repository-images.githubusercontent.com/000000/0000-0000-00-000000"
        }
    ],
    "post": { // Only present if loading a post
        "title": Lorem ipsum",
        "body": "<p>Lorem ipsum dolor sit amet...</p>",
        "labels": [
            {
                "name": "Some Label",
                "color": "#ffcc00"
            }
        ],
        "created": "2022-07-10T09:20:00Z",
        "updated": "2022-07-11T13:05:00Z"
    },
    "page": "home", // Type of the page that is being rendered
}
```

## Core Variables

These variables are required in the `core.html` template.

 - `title`: Renders the title of the page (normally in the `<title>` HTML tag)
 - `content`: Renders the contend of the page (normally directly in the `<body>` HTML tag)

Also within `core.html` is the `page_variables` variable, which contains the
object in the same form as above.

---

# Stack

## Data

### Cloudflare KV

Only data that is required to be accessed from the Worker API should be stored
in here. Ideally, anything that is performance critical to responding a
request, in order to take advantage of the low latency. Data that is stored
here should be written to very infrequently, ideally using it as a look up
table.

 - Domain and user mappings
   - Key: Domain
   - Value: `{ username, api_token }`
     - `username`: GitHub username linked to domain
     - `api_token`: GitHub API token for specific user

### Cloudflare DO

Data that is stored here should also be performance critical and used by the
Worker API. The difference with Cloudflare KV is that it should be used for
objects that frequently need to be written to and must be in sync (eg
counters).

### Database

Should contain the bulk of the data, anything that doesn't need to be accessed
from the worker. Even for data that is stored within Cloudflare's network, this
should be the source of truth for everything, and synced when required to the
worker so it has relatively up-to-date information.

 - `user`
   - `username`
   - `api_token`
   - `referral_code`
   - `date_created`
   - `last_login`
   - `status`
 - `referral_code`
   - `code`
   - `creator`
   - `date_created`
   - `count`
   - `status`
 - `domain`
   - `cloudflare_id`
   - `domain`
   - `user`
   - `hostname_status`
   - `ssl_status`
   - `date_added`
   - `date_updated`
   - `status`

## APIs

### Cloudflare Worker

The Worker should only be used for page generation and accessing Cloudflare
objects, such as KV, DO and APIs.

#### APIs

 - `custom_hostname`

### SvelteKit

Most of the API heavy lifting should take place within the SvelteKit backend.
It is responsible for taking requests from the client and performing all logic,
including forwarding any necessary requests to the Cloudflare Worker (and by
extension KV, DO and APIs), and interacting with the database.

 - OAuth
   - GitHub API
   - Database
 - Domain Verification
   - Cloudflare Worker
 - Profile
   - Database

---

1. Authenticate with GitHub

[done]

Perform authentication with GitHub to retrieve `api_token` and `username`.
Store this information in the database.

2. Referral code

[done]

Allow user to input referral code. Verify referral code in database and add to
user to redeem the referral code.

3. Add domain

[done]

Allow user to input domain. Add domain to Cloudflare (via Worker), then store
domain details in database.

4. Verify domain

[done]

Get domain verification details from Cloudflare (via Worker, this can be done
in the previous step). Don't store them in the database, just present them to
the user.

To check the verification, make a request to the Worker, to return the required
records (if any), and the status. If the status is active, update the record in
the database and make a request to the Worker to add to the KV store the
domain, GitHub username and API token.

# Authentication

## JWT

JWT will contain user ID (and maybe expiry?). It will be decoded in the
SvelteKit `hook.ts`, verified and the user will be fetched from the database. The
fetched user will be assigned to the locals for the request.

## Client Side

An endpoint will be available for the user that will return the information for
the user that is currently logged in. This can be used to fill in user
information in the UI, and as a starting location for other information like
domains.

