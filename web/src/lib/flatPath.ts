/** Stable string key so caches do not invalidate when array ref changes but data is same. */
export function flatKey(flat: number[]): string {
  if (flat.length === 0) return "0";
  const mid = flat[Math.floor(flat.length / 2)] ?? 0;
  return `${flat.length}:${flat[0]}:${mid}:${flat[flat.length - 1]}`;
}
