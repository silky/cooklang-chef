import { readonly, writable } from 'svelte/store';

export type UiConfig = {
	tags: Record<string, TagProps>;
};

export type TagProps = {
	emoji: string | null;
};

const apiUrl = '/ui_config';

async function getConfig() {
	try {
		const config = await fetch(apiUrl).then((r) => r.json());
		return config as UiConfig;
	} catch (e) {
		console.error(e);
		return null;
	}
}

const store = writable<Awaited<ReturnType<typeof getConfig>>>(null);

getConfig().then((c) => store.set(c));

export const uiConfig = readonly(store);
