import adapter from '@sveltejs/adapter-auto';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	// 개발 중엔 /api 를 백엔드로 넘겨 CORS 없이 같은 출처로 부른다 ($lib/api/env.ts 기본 '/api'). SSE도 그대로 흐른다.
	server: {
		proxy: {
			'/api': { target: process.env.VITE_API_PROXY ?? 'http://127.0.0.1:8080', rewrite: (p) => p.replace(/^\/api/, '') }
		}
	},
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
			// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
			// See https://svelte.dev/docs/kit/adapters for more information about adapters.
			adapter: adapter()
		})
	]
});
