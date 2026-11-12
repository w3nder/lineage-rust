export const BLOCK = 32768;
export const ORIGIN: [number, number] = [20, 18];
export const LIMITS_X: [number, number] = [11, 26];
export const LIMITS_Y: [number, number] = [10, 25];
export const URL = "https://raw.githubusercontent.com/npetrovski/l2-world-map/master/Maps";

export type TileKey = [number, number];

export function tileOf(position: number, origin: number): number {
  return Math.floor(position / BLOCK) + origin;
}

export function tileMin(key: TileKey): [number, number] {
  return [(key[0] - ORIGIN[0]) * BLOCK, (key[1] - ORIGIN[1]) * BLOCK];
}

export function inside(key: TileKey): boolean {
  return key[0] >= LIMITS_X[0] && key[0] <= LIMITS_X[1] && key[1] >= LIMITS_Y[0] && key[1] <= LIMITS_Y[1];
}

export function visible(center: [number, number], half: [number, number]): TileKey[] {
  const x0 = tileOf(center[0] - half[0], ORIGIN[0]);
  const x1 = tileOf(center[0] + half[0], ORIGIN[0]);
  const y0 = tileOf(center[1] - half[1], ORIGIN[1]);
  const y1 = tileOf(center[1] + half[1], ORIGIN[1]);
  const out: TileKey[] = [];
  for (let x = x0; x <= x1; x++) for (let y = y0; y <= y1; y++) if (inside([x, y])) out.push([x, y]);
  return out;
}

export function fileName(key: TileKey): string {
  return `${key[0]}_${key[1]}.jpg`;
}

type Entry = { image: HTMLImageElement; state: "loading" | "ready" | "missing" };

const cache = new Map<string, Entry>();
let pending = 0;
const MAX_PENDING = 12;

export function tile(key: TileKey, onReady: () => void): HTMLImageElement | null {
  const id = fileName(key);
  const found = cache.get(id);
  if (found) return found.state === "ready" ? found.image : null;
  if (pending >= MAX_PENDING) return null;
  const image = new Image();
  const entry: Entry = { image, state: "loading" };
  cache.set(id, entry);
  pending += 1;
  image.onload = () => {
    entry.state = "ready";
    pending -= 1;
    onReady();
  };
  image.onerror = () => {
    entry.state = "missing";
    pending -= 1;
    onReady();
  };
  image.src = `${URL}/${id}`;
  return null;
}

export function loading(): boolean {
  return pending > 0;
}

export function reset() {
  for (const [id, entry] of cache) if (entry.state === "loading") cache.delete(id);
  pending = 0;
}
