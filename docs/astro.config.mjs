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
					label: 'Tutorials',
					autogenerate: { directory: 'tutorials' }
				},
				{
					label: 'Guides',
					items: [
						{ label: 'Example Guide', slug: 'guides/example' },
					],
				},
				{
					label: 'Design',
					autogenerate: { directory: 'explanation' },
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
