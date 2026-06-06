import { writable, derived, get } from 'svelte/store';
import type { AppSettings, IgnoreRules, GitignoreSettings } from '$lib/tauri';
import {
	getSettings,
	updateIgnoreRules,
	updateGitignoreSettings,
	DEFAULT_IGNORE_RULES,
	DEFAULT_GITIGNORE_SETTINGS
} from '$lib/tauri';
import { activeTabId, refreshTab } from './tabs';
import { showToast } from './bookmarks';

const MAX_NAME_LEN = 64;

export const appSettings = writable<AppSettings>({
	version: 1,
	ignore_rules: { ...DEFAULT_IGNORE_RULES },
	gitignore: { ...DEFAULT_GITIGNORE_SETTINGS }
});

export const ignoreRules = derived(appSettings, ($s) => $s.ignore_rules);
export const gitignoreSettings = derived(appSettings, ($s) => $s.gitignore);
export const respectsGitignore = derived(gitignoreSettings, ($g) => $g.respect_gitignore);

let initialized = false;

/** Inicialização — chamada no initExplorer. */
export async function initSettings(): Promise<void> {
	try {
		appSettings.set(await getSettings());
	} catch {
		// Falha silenciosa: mantém defaults.
	}

	// Recarrega o diretório ativo quando gitignore settings mudam.
	gitignoreSettings.subscribe(() => {
		if (initialized) {
			const id = get(activeTabId);
			if (id) refreshTab(id);
		} else {
			initialized = true;
		}
	});
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

// ── Gitignore ───────────────────────────────────────────────────
export async function patchGitignoreSettings(patch: Partial<GitignoreSettings>): Promise<void> {
	const previous = get(appSettings);
	const newGit: GitignoreSettings = { ...previous.gitignore, ...patch };
	appSettings.set({ ...previous, gitignore: newGit }); // otimista
	try {
		appSettings.set(await updateGitignoreSettings(newGit));
	} catch (err) {
		appSettings.set(previous); // reverte
		showToast(`Falha ao salvar gitignore: ${String(err)}`, 'error');
	}
}

export const toggleRespectGitignore = () =>
	patchGitignoreSettings({ respect_gitignore: !get(gitignoreSettings).respect_gitignore });

export const toggleGlobalGitignore = () =>
	patchGitignoreSettings({
		respect_global_gitignore: !get(gitignoreSettings).respect_global_gitignore
	});

export const toggleIgnoreFiles = () =>
	patchGitignoreSettings({ respect_ignore_files: !get(gitignoreSettings).respect_ignore_files });

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
