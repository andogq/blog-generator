import adapter from '@sveltejs/adapter-auto';
import preprocess from 'svelte-preprocess';
import dotenv from "dotenv";

// TODO: Replace this with bulit-in env in new sveltekit version
dotenv.config({
    path: "../.env.dev"
});

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://github.com/sveltejs/svelte-preprocess
	// for more information about preprocessors
	preprocess: preprocess(),

	kit: {
		adapter: adapter()
    }
};

export default config;
