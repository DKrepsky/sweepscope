const pad = (n: number) => String(n).padStart(2, '0');

/** The backend sends ISO 8601 UTC; the date shown is local time, `2026-11-24 14:05`. Undefined for text that is no date. */
export function formatStoredDate(iso: string | null | undefined): string | undefined {
  if (!iso) return undefined;
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return undefined;
  const day = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  return `${day} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
