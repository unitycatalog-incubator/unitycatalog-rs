// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightOpenAPI, { openAPISidebarGroups } from 'starlight-openapi'

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Unity Catalog Rust',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/unitycatalog-incubator/unitycatalog-rs' }],
			sidebar: [
				{
					label: 'Guides',
					items: [
						// Each item here is one entry in the navigation menu.
						{ label: 'Example Guide', slug: 'guides/example' },
					],
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
				...openAPISidebarGroups,
			],
			plugins: [
				// Generate the OpenAPI documentation pages.
				starlightOpenAPI([
					{
						base: 'api',
						schema: '../openapi/openapi.yaml',
					},
				]),
			],
		}),
	],
});
