export function grouped(value: number): string {
  const sign = value < 0 ? "-" : "";
  const digits = Math.abs(Math.trunc(value)).toString();
  let out = "";
  for (let index = 0; index < digits.length; index++) {
    if (index > 0 && (digits.length - index) % 3 === 0) out += " ";
    out += digits[index];
  }
  return sign + out;
}

export function percent(value: number): string {
  return value.toFixed(2).replace(".", ",") + "%";
}

export function compact(value: number): string {
  if (value >= 1_000_000) return (value / 1_000_000).toFixed(1).replace(".", ",") + " M";
  if (value >= 1_000) return Math.round(value / 1_000) + " k";
  return String(Math.round(value));
}

export function fraction(current: number, max: number): number {
  if (max <= 0) return 0;
  return Math.min(1, Math.max(0, current / max));
}

export function clockTime(at: number): string {
  if (at <= 0) return "";
  return new Date(at * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false });
}

export function signed(value: number): string {
  const rounded = Math.round(value);
  return rounded > 0 ? `+${rounded}` : String(rounded);
}
