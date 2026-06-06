import { writable, derived } from 'svelte/store';
import { activeTab } from '$lib/stores/tabs';
import { getGitignoreInfo, isTauri, type GitignoreInfo } from '$lib/tauri';

export const currentGitignoreInfo = writable<GitignoreInfo | null>(null);
export const gitignoreInfoLoading = writable<boolean>(false);

export const isGitRepo = derived(currentGitignoreInfo, ($info) => $info?.is_git_repo ?? false);

export async function refreshGitignoreInfo(path: string): Promise<void> {
	if (!isTauri() || !path) {
		currentGitignoreInfo.set(null);
		return;
	}
	gitignoreInfoLoading.set(true);
	try {
		currentGitignoreInfo.set(await getGitignoreInfo(path));
	} catch {
		currentGitignoreInfo.set(null);
	} finally {
		gitignoreInfoLoading.set(false);
	}
}

// Atualiza ao trocar de pasta ativa.
activeTab.subscribe((tab) => {
	if (tab?.path) refreshGitignoreInfo(tab.path);
});
