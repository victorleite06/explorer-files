import { writable, derived, get } from 'svelte/store';
import type { AppSettings, IgnoreRules } from '$lib/tauri';
import { getSettings, updateIgnoreRules, DEFAULT_IGNORE_RULES } from '$lib/tauri';
import { showToast } from './bookmarks';

const MAX_NAME_LEN = 64;

export const appSettings = writable<AppSettings>({
	version: 1,
	ignore_rules: { ...DEFAULT_IGNORE_RULES }
});

export const ignoreRules = derived(appSettings, ($s) => $s.ignore_rules);

/** Inicialização — chamada no initExplorer. */
export async function initSettings(): Promise<void> {
	try {
		appSettings.set(await getSettings());
	} catch {
		// Falha silenciosa: mantém defaults.
	}
}

/** Atualiza rules com merge; otimista, reverte em erro. */
export async function patchIgnoreRules(patch: Partial<IgnoreRules>): Promise<void> {
	const previous = get(appSettings);
	const newRules: IgnoreRules = { ...previous.ignore_rules, ...patch };
	appSettings.set({ ...previous, ignore_rules: newRules }); // otimista
	try {
		appSettings.set(await updateIgnoreRules(newRules));
	} catch (err) {
		appSettings.set(previous); // reverte
		showToast(`Falha ao salvar configurações: ${String(err)}`, 'error');
	}
}

// ── Toggles rápidos ─────────────────────────────────────────────
export const toggleDotfiles = () =>
	patchIgnoreRules({ show_dotfiles: !get(ignoreRules).show_dotfiles });

export const toggleNodeModules = () =>
	patchIgnoreRules({ show_node_modules: !get(ignoreRules).show_node_modules });

export const toggleBuildArtifacts = () =>
	patchIgnoreRules({ show_build_artifacts: !get(ignoreRules).show_build_artifacts });

// ── Listas custom ───────────────────────────────────────────────
export async function addCustomHidden(name: string): Promise<void> {
	const n = name.trim();
	const r = get(ignoreRules);
	if (!n || n.length > MAX_NAME_LEN || r.custom_hidden.includes(n)) return;
	await patchIgnoreRules({ custom_hidden: [...r.custom_hidden, n] });
}

export async function removeCustomHidden(name: string): Promise<void> {
	const r = get(ignoreRules);
	await patchIgnoreRules({ custom_hidden: r.custom_hidden.filter((x) => x !== name) });
}

export async function addCustomShown(name: string): Promise<void> {
	const n = name.trim();
	const r = get(ignoreRules);
	if (!n || n.length > MAX_NAME_LEN || r.custom_shown.includes(n)) return;
	await patchIgnoreRules({ custom_shown: [...r.custom_shown, n] });
}

export async function removeCustomShown(name: string): Promise<void> {
	const r = get(ignoreRules);
	await patchIgnoreRules({ custom_shown: r.custom_shown.filter((x) => x !== name) });
}
