/** Tamanho legível (bytes → B/KB/MB/GB). */
export function formatIndexSize(bytes: number): string {
	if (bytes === 0) return '0 B';
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1_073_741_824) return `${(bytes / 1_048_576).toFixed(1)} MB`;
	return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
}

/** Tempo restante legível. */
export function formatEta(seconds: number | null): string {
	if (seconds === null) return '';
	if (seconds < 60) return 'menos de 1 min';
	if (seconds < 3600) return `~${Math.round(seconds / 60)} min`;
	const h = Math.floor(seconds / 3600);
	const min = Math.round((seconds % 3600) / 60);
	return `~${h} h ${min} min`;
}
