export function getMaxBotName(info: Record<string, unknown> | null): string | null {
  if (!info || typeof info !== "object") return null;
  const name = (info as Record<string, unknown>).name;
  return typeof name === "string" ? name : null;
}
