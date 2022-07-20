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
            "github_url": "https://github.com/andogq/some_repo"
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
