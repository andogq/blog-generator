<script lang="ts">
    import { page } from "$app/stores";

    const GITHUB_OAUTH_URL = new URL("https://github.com/login/oauth/authorize");
    GITHUB_OAUTH_URL.search = new URLSearchParams({
        client_id: import.meta.env.VITE_GITHUB_CLIENT_ID,
        redirect_uri: new URL("/oauth", $page.url.origin).href,
        scope: ["read:user", "user:email"].join(" ")
    }).toString();
</script>

<div id="content">
    <h1>Portfolio Blog Thing</h1>

    <p>
        This is a thing to handle keeping your portfolio/blog up to date
        through your GitHub account! Simply log in through GitHub, set up your
        templates and your README, and link with your personal domain! Once
        it's up and running, you'll always be showing your most up-to-date
        projects and can easily update your portfolio.
    </p>

    <p>
        By pulling information from your GitHub profile, your site will always
        contain the most up to date information like your name, bio, links,
        photo, projects, and more! All of this can be presented in any way that
        suits you best, total customisability with your own HTML templates, CSS
        and JS.
    </p>

    <p>
        Oh, you wanted a blog too? Use GitHub issues as blog posts! Every issue
        on your user's repository (mentioned below) is used as a post, including
        full markdown support!
    </p>

    <h2>The Repository</h2>

    <p>
        All of the required information is contained within a repository named
        the same as your username. For example, for the username
        <code>andogq</code>, the repository <code>andogq/andogq</code> will be
        used.
    </p>

    <p>
        The repository contains two sections, the <code>README.md</code> file
        and the <code>templates</code> folder. The <code>README.md</code> file
        is used to create the body of your portfolio, and is a good place to
        introduce yourself, talk about your skills, work history and more. It
        corresponds with GitHub's
        <a href="https://docs.github.com/en/account-and-profile/setting-up-and-managing-your-github-profile/customizing-your-profile/managing-your-profile-readme">profile README</a>,
        meaning that this file will also be shown when somebody visits your
        GitHub profile directly. This file will be rendered as a markdown file.
    </p>

    <p>
        The <code>templates</code> folder is where all of the HTML templates
        belong. The templates are defined using <a href="https://handlebarsjs.com/">Handlebars</a>
        style templates, with a number of variables available (listed below).
        The following templates are required:
    </p>

    <ul>
        <li>
            <code>core.html</code>: This template provides the base structure
            for the document, including the HTML <code>head</code> tag, and any
            desired imports or styling.
        </li>
        <li>
            <code>home.html</code>: This template is used to render the index
            page. It will be rendered, and then inserted within the
            <code>core.html</code> mentioned above.
        </li>
        <li>
            <code>post.html</code>: This template is used to render a post. It
            isn't required if you aren't using the post feature.
        </li>
    </ul>

    <h3>External Resources</h3>

    <p>
        Only the HTML templates above are used, everything else within the
        repository is ignored, including any CSS and JS files. In order to make
        these files available, it is suggested to set up a <a href="https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site">GitHub Pages site</a>,
        and import these files with regular URLs.
    </p>

    <h2>The Domain</h2>

    <p>
        The second aspect of your new portfolio is the domain! Currently, only
        BYO domain is supported (although free subdomains might be in the
        works), so make sure you have one of these available, and that you have
        the ability to add DNS records.
    </p>

    <p>
        <code>CNAME</code> records are used to direct traffic from your domain
        to your portfolio hosted here. During sign up, you will be instructed to
        add a <code>CNAME</code> record and some <code>TXT</code> records for
        verification, to allow SSL certificates to be generated and for traffic
        to be directed properly.
    </p>

    <h2>That's It!</h2>

    <p>
        That should be everything you need to get started! To see an example,
        check out my <a href="https://ando.gq">website</a> and the accompanying
        <a href="https://github.com/andogq/andogq">repository</a>. Feel free to
        use my templates as a base and go from there!
    </p>

    <p>
        Finally, if you have any feedback, suggestions or complaints, please do
        not hesitate to let me know, I'd love to hear them!
    </p>

    <div class="block">
        <p>Haven't been scared off yet?</p>
        <p>To get started, <a href={GITHUB_OAUTH_URL.href}>login with GitHub</a>.</p>
    </div>

    <h2>Available Variables</h2>

    <p>
        <a href="https://handlebarsjs.com/">Handlebars</a> style templating is
        used to generate the site, using the following object below. Nested keys
        can be accessed using dot notation (as in regular JS), and some simple
        logic is available, such as conditionals and loops.
    </p>

    <p>
        Whenever Markdown content is being embedded (eg user's README or a post
        body), use three brackets instead of two, to ensure the rendered HTML
        isn't escaped (eg <code>{`{{{readme}}}`}</code>).
    </p>

    <h3>Examples</h3>

    <ul>
        <li>User's name: <code>{`{{user.name}}`}</code></li>
        <li>Listing each post: <code>{`{{#each posts}}<p>{{this.title}}</p>{{/each}}`}</code></li>
        <li>Conditionally showing company: <code>{`{{#if user.company}}<p>I'm working at a company!</p>{{/if}}`}</code></li>
    </ul>

    <pre class="block"><code lang="json">{`{
    "user": {
        "name": "John Smith",
        "profile_picture": "https://example.com/image.jpg",
        "email": "email@example.com",
        "bio": "Aliquid atque amet voluptate non minima nostrum officiis.",

        "location": "Melbourne, Australia",
        "hireable": true,
        "company": "Some Company",

        "github_profile": "https://github.com/andogq",
        "twitter_profile": "https://twitter.com/andogq",

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
}`}</code></pre>

    <h2>Still Wanting More?</h2>

    <p>
        Here's a sneak peak of features that I'm working on.
    </p>

    <ul>
        <li>Hosted sub domains</li>
        <li>Nicer static files (no more GitHub pages)</li>
        <li>Analytics</li>
        <li>Templates - pre made options and a better way to share them</li>
        <li>...and more!</li>
    </ul>
</div>

<style>
    #content {
        padding: var(--space-md) var(--space-lg);
        height: 100%;

        display: flex;
        flex-direction: column;
        gap: var(--space-sm);

        overflow: scroll;
    }

    .block {
        background: var(--background-light);
        border-radius: var(--border-radius);
        padding: var(--space-sm);
    }
</style>

