export function toResetDate(value: string | number | null): Date | null {
  if (value == null || (typeof value === "string" && value.trim() === "")) return null;
  const numeric = typeof value === "number" || /^[+-]?\d+(?:\.\d+)?$/.test(value.trim());
  const number = numeric ? Number(value) : null;
  const date = number == null
    ? new Date(value)
    : new Date(Math.abs(number) < 1e12 ? number * 1000 : number);
  return Number.isFinite(date.getTime()) ? date : null;
}

/** Local Date getters intentionally use the computer's current timezone and DST. */
export function formatResetTime(value: string | number | null): string | null {
  const date = toResetDate(value);
  if (!date) return null;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${String(date.getFullYear()).padStart(4, "0")}.${pad(date.getMonth() + 1)}.${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

/** 套餐到期时间，精确到分钟，按本机时区显示。 */
export function formatPlanExpiry(value: string | number | null): string | null {
  const date = toResetDate(value);
  if (!date) return null;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}.${pad(date.getMonth() + 1)}.${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export interface PlanRemaining {
  days: number;
  expired: boolean;
}

/** 套餐剩余的整天数（向下取整）；刚好到期或已过期时 expired 为 true。 */
export function planRemaining(
  value: string | number | null,
  now: number,
): PlanRemaining | null {
  const date = toResetDate(value);
  if (!date) return null;
  const diff = date.getTime() - now;
  return {
    days: Math.max(0, Math.floor(diff / 86_400_000)),
    expired: diff <= 0,
  };
}
