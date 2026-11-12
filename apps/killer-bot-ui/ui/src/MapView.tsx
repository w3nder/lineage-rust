import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useContextMenu } from "./ContextMenu";
import { useT } from "./i18n";
import { nearbyMenu } from "./menus";
import { BLOCK, loading as tilesLoading, tile, tileMin, visible as visibleTiles } from "./tiles";
import type { Command, Kind, Nearby, World } from "./model";

const COLORS: Record<Kind, string> = { player: "#4fd1e8", npc: "#3dd68c", monster: "#e5534b", item: "#e8b84f" };
const ORDER: Kind[] = ["monster", "npc", "player", "item"];
const MIN_ZOOM = 0.04;
const MAX_ZOOM = 3;
const DEFAULT_ZOOM = 0.22;
const PICK_RADIUS = 9;
const CROWD_PLAYERS = 30;
const CROWD_ZOOM = 0.9;

type View = { zoom: number; panX: number; panY: number };

export type Filters = Record<Kind, boolean>;

type Props = {
  world: World;
  filters: Filters;
  onFilters: (filters: Filters) => void;
  onSelect: (id: number) => void;
  onInfo: () => void;
  onMove: (x: number, y: number, z: number) => void;
  zRange: number;
  options: { map: boolean; polygon: boolean };
  onOptions: (options: Props["options"]) => void;
  onZRange: (value: number) => void;
  send: (command: Command) => void;
};

function toScreen(view: View, center: [number, number], size: [number, number], x: number, y: number): [number, number] {
  return [size[0] / 2 + (x - center[0] - view.panX) * view.zoom, size[1] / 2 + (y - center[1] - view.panY) * view.zoom];
}

function toWorld(view: View, center: [number, number], size: [number, number], px: number, py: number): [number, number] {
  return [center[0] + view.panX + (px - size[0] / 2) / view.zoom, center[1] + view.panY + (py - size[1] / 2) / view.zoom];
}

function visibleNearby(world: World, filters: Filters, zRange: number): Nearby[] {
  const z = world.character.position[2];
  return world.nearby.filter((entry) => filters[entry.kind] && Math.abs(entry.z - z) <= zRange);
}

export function MapView(props: Props) {
  const { world, filters, onFilters, onSelect, onInfo, onMove, zRange, options, onOptions, onZRange, send } = props;
  const t = useT();
  const light = typeof document !== "undefined" && document.documentElement.dataset.theme === "light";
  const { show, menu } = useContextMenu();
  const canvas = useRef<HTMLCanvasElement>(null);
  const holder = useRef<HTMLDivElement>(null);
  const [view, setView] = useState<View>({ zoom: DEFAULT_ZOOM, panX: 0, panY: 0 });
  const [size, setSize] = useState<[number, number]>([800, 420]);
  const [hover, setHover] = useState<{ x: number; y: number } | null>(null);
  const drag = useRef<{ x: number; y: number; moved: boolean } | null>(null);
  const [tick, setTick] = useState(0);

  const cx = world.character.position[0];
  const cy = world.character.position[1];
  const center = useMemo<[number, number]>(() => [cx, cy], [cx, cy]);

  useEffect(() => {
    const node = holder.current;
    if (!node) return;
    const observer = new ResizeObserver(() => setSize([node.clientWidth, node.clientHeight]));
    observer.observe(node);
    setSize([node.clientWidth, node.clientHeight]);
    return () => observer.disconnect();
  }, []);

  const pick = useCallback(
    (px: number, py: number): Nearby | null => {
      let best: Nearby | null = null;
      let bestDistance = PICK_RADIUS;
      for (const entry of visibleNearby(world, filters, zRange)) {
        const [sx, sy] = toScreen(view, center, size, entry.x, entry.y);
        const distance = Math.hypot(sx - px, sy - py);
        if (distance <= bestDistance) {
          best = entry;
          bestDistance = distance;
        }
      }
      return best;
    },
    [world, filters, zRange, view, center, size],
  );

  const hovered = hover ? pick(hover.x, hover.y) : null;
  const targetId = world.target?.id ?? null;

  useEffect(() => {
    const node = canvas.current;
    if (!node) return;
    const ratio = window.devicePixelRatio || 1;
    node.width = Math.round(size[0] * ratio);
    node.height = Math.round(size[1] * ratio);
    const ctx = node.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    ctx.clearRect(0, 0, size[0], size[1]);

    if (options.map) {
      const half: [number, number] = [size[0] / 2 / view.zoom, size[1] / 2 / view.zoom];
      const origin: [number, number] = [center[0] + view.panX, center[1] + view.panY];
      let drew = false;
      for (const key of visibleTiles(origin, half)) {
        const image = tile(key, () => setTick((value) => value + 1));
        if (!image) continue;
        const [minX, minY] = tileMin(key);
        const [sx, sy] = toScreen(view, center, size, minX, minY);
        const side = BLOCK * view.zoom;
        ctx.drawImage(image, Math.floor(sx), Math.floor(sy), Math.ceil(side) + 1, Math.ceil(side) + 1);
        drew = true;
      }
      ctx.fillStyle = light ? "rgba(255,255,255,0.28)" : "rgba(8,11,15,0.45)";
      ctx.fillRect(0, 0, size[0], size[1]);
      const step = 1000 * view.zoom;
      if (step > 12 && !drew) {
        ctx.strokeStyle = light ? "#e3e9f0" : "#131922";
        ctx.lineWidth = 1;
        const [ox] = toScreen(view, center, size, Math.floor((center[0] + view.panX) / 1000) * 1000, 0);
        const [, oy] = toScreen(view, center, size, 0, Math.floor((center[1] + view.panY) / 1000) * 1000);
        ctx.beginPath();
        for (let x = ox - Math.ceil(size[0] / step) * step; x < size[0] + step; x += step) {
          ctx.moveTo(Math.round(x) + 0.5, 0);
          ctx.lineTo(Math.round(x) + 0.5, size[1]);
        }
        for (let y = oy - Math.ceil(size[1] / step) * step; y < size[1] + step; y += step) {
          ctx.moveTo(0, Math.round(y) + 0.5);
          ctx.lineTo(size[0], Math.round(y) + 0.5);
        }
        ctx.stroke();
      }
    }

    const me = toScreen(view, center, size, center[0], center[1]);
    const glow = ctx.createRadialGradient(me[0], me[1], 0, me[0], me[1], Math.max(size[0], size[1]) * 0.55);
    glow.addColorStop(0, light ? "rgba(14,155,180,0.10)" : "rgba(79,209,232,0.08)");
    glow.addColorStop(1, "rgba(79,209,232,0)");
    ctx.fillStyle = glow;
    ctx.fillRect(0, 0, size[0], size[1]);

    ctx.setLineDash([4, 6]);
    ctx.strokeStyle = light ? "#b8c6d4" : "#2b3a48";
    for (const radius of [1200, 600]) {
      ctx.beginPath();
      ctx.arc(me[0], me[1], radius * view.zoom, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.setLineDash([]);

    const points = (((world.options as { polygon?: { points?: { x: number; y: number }[] } }).polygon?.points) ?? []).map((point) => toScreen(view, center, size, point.x, point.y));
    if (points.length > 0) {
      ctx.beginPath();
      points.forEach(([px, py], index) => (index === 0 ? ctx.moveTo(px, py) : ctx.lineTo(px, py)));
      if (points.length > 2) ctx.closePath();
      ctx.fillStyle = "rgba(232,184,79,0.08)";
      if (points.length > 2) ctx.fill();
      ctx.setLineDash([6, 4]);
      ctx.strokeStyle = "#e8b84f";
      ctx.lineWidth = 1.5;
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = "#e8b84f";
      for (const [px, py] of points) {
        ctx.beginPath();
        ctx.arc(px, py, 3.5, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    const visible = visibleNearby(world, filters, zRange)
      .map((entry) => ({ entry, at: toScreen(view, center, size, entry.x, entry.y) }))
      .filter(({ at }) => at[0] > -20 && at[0] < size[0] + 20 && at[1] > -20 && at[1] < size[1] + 20);

    for (const { entry, at } of visible) {
      ctx.globalAlpha = entry.dead ? 0.4 : 0.9;
      ctx.fillStyle = COLORS[entry.kind];
      ctx.beginPath();
      ctx.arc(at[0], at[1], entry.kind === "item" ? 2.5 : 3.2, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.globalAlpha = 1;

    const crowd = visible.filter(({ entry }) => entry.kind === "player").length > CROWD_PLAYERS && view.zoom < CROWD_ZOOM;
    const placed: [number, number, number, number][] = [];
    ctx.font = '500 11px Inter, system-ui, sans-serif';
    ctx.textBaseline = "middle";
    const sorted = [...visible].sort((a, b) => {
      const rank = (entry: Nearby) => (entry.id === targetId ? 0 : entry.id === hovered?.id ? 1 : 2 + ORDER.indexOf(entry.kind));
      return rank(a.entry) - rank(b.entry);
    });
    for (const { entry, at } of sorted) {
      const strong = entry.id === targetId || entry.id === hovered?.id;
      if (strong) {
        ctx.strokeStyle = entry.id === targetId ? "#e8b84f" : light ? "#0f1720" : "#e6eaf0";
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        ctx.arc(at[0], at[1], 8, 0, Math.PI * 2);
        ctx.stroke();
      }
      const wanted = strong || (entry.kind !== "item" && !(crowd && entry.kind === "player") && view.zoom >= 0.1);
      if (!wanted) continue;
      let text = entry.name;
      if (strong && entry.title) text += `  <${entry.title}>`;
      if (strong) text += `  ·  ${Math.round(entry.distance)}`;
      const width = ctx.measureText(text).width;
      let x = at[0] + 9;
      if (x + width > size[0] - 6) x = at[0] - 9 - width;
      const frame: [number, number, number, number] = [x - 4, at[1] - 9, width + 8, 18];
      const overlaps = placed.some((other) => frame[0] < other[0] + other[2] + 1 && frame[0] + frame[2] > other[0] - 1 && frame[1] < other[1] + other[3] + 1 && frame[1] + frame[3] > other[1] - 1);
      if (!strong && overlaps) continue;
      placed.push(frame);
      ctx.fillStyle = light ? (strong ? "rgba(255,255,255,0.95)" : "rgba(255,255,255,0.72)") : strong ? "rgba(11,14,18,0.92)" : "rgba(11,14,18,0.62)";
      ctx.beginPath();
      ctx.roundRect(frame[0], frame[1], frame[2], frame[3], 5);
      ctx.fill();
      ctx.fillStyle = light ? "#1d2733" : "#c7cfda";
      ctx.fillText(text, x, at[1] + 0.5);
    }

    ctx.fillStyle = light ? "#ffffff" : "#0e1218";
    ctx.strokeStyle = light ? "#0f1720" : "#e6eaf0";
    ctx.lineWidth = 2.5;
    ctx.beginPath();
    ctx.arc(me[0], me[1], 7, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
  }, [world, filters, zRange, view, size, options.map, hovered, targetId, center, light, tick]);

  const point = (event: React.PointerEvent | React.MouseEvent): [number, number] => {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return [event.clientX - rect.left, event.clientY - rect.top];
  };

  const zoomBy = (factor: number) => setView((current) => ({ ...current, zoom: Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, current.zoom * factor)) }));

  const counts = (kind: Kind) => world.nearby.filter((entry) => entry.kind === kind).length;
  const chip = (kind: Kind, label: string, bg: string, border: string, ink: string) => (
    <button
      key={kind}
      type="button"
      className="pill"
      style={{ borderColor: border, background: bg, color: ink, opacity: filters[kind] ? 1 : 0.45 }}
      onClick={() => onFilters({ ...filters, [kind]: !filters[kind] })}
      aria-pressed={filters[kind]}
    >
      <span className="dot" style={{ background: COLORS[kind] }} />
      {label} <span className="num">{counts(kind)}</span>
    </button>
  );

  const cursor = hover ? toWorld(view, center, size, hover.x, hover.y) : null;

  return (
    <section className="map" aria-label={t("Mapa")} ref={holder}>
      <canvas
        ref={canvas}
        onPointerDown={(event) => {
          const [x, y] = point(event);
          drag.current = { x, y, moved: false };
          (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
        }}
        onPointerMove={(event) => {
          const [x, y] = point(event);
          setHover({ x, y });
          const current = drag.current;
          if (!current) return;
          const dx = x - current.x;
          const dy = y - current.y;
          if (!current.moved && Math.hypot(dx, dy) < 4) return;
          current.moved = true;
          current.x = x;
          current.y = y;
          setView((old) => ({ ...old, panX: old.panX - dx / old.zoom, panY: old.panY - dy / old.zoom }));
        }}
        onPointerUp={(event) => {
          const current = drag.current;
          drag.current = null;
          if (!current || current.moved) return;
          const [x, y] = point(event);
          const found = pick(x, y);
          if (options.polygon) {
            const [wx, wy] = toWorld(view, center, size, x, y);
            send({ type: "addPolygonPoint", x: Math.round(wx), y: Math.round(wy) });
          } else if (found) {
            onSelect(found.id);
          } else {
            const [wx, wy] = toWorld(view, center, size, x, y);
            onMove(Math.round(wx), Math.round(wy), world.character.position[2]);
          }
        }}
        onPointerLeave={() => setHover(null)}
        onContextMenu={(event) => {
          const [x, y] = point(event);
          const found = pick(x, y);
          const [wx, wy] = toWorld(view, center, size, x, y);
          const walk = { label: t("Andar até aqui"), run: () => onMove(Math.round(wx), Math.round(wy), world.character.position[2]) };
          if (!found) return show(event, [walk]);
          const items = nearbyMenu(found, t, send, onInfo);
          items.push({ label: "", run: () => undefined, divider: true }, walk);
          show(event, items);
        }}
        onWheel={(event) => zoomBy(event.deltaY < 0 ? 1.15 : 1 / 1.15)}
        style={{ cursor: hovered ? "pointer" : "crosshair", touchAction: "none" }}
      />
      {menu}
      <div className="map-chips">
        {chip("monster", t("Monstros"), "#1b1313", "#3a2626", "#f0a39e")}
        {chip("npc", "NPCs", "#111b16", "#1f3a2e", "#8de3b9")}
        {chip("player", t("Jogadores"), "#101b20", "#1f3a42", "#8fe4f2")}
        {chip("item", t("Itens"), "#1b1710", "#3a3220", "#f0d28e")}
      </div>
      <div className="map-zoom">
        <button type="button" className="ico" aria-label={t("Aproximar")} onClick={() => zoomBy(1.4)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
        </button>
        <button type="button" className="ico" aria-label={t("Afastar")} onClick={() => zoomBy(1 / 1.4)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" aria-hidden="true"><path d="M5 12h14" /></svg>
        </button>
        <button type="button" className="ico" aria-label={t("Centralizar no personagem")} onClick={() => setView({ zoom: DEFAULT_ZOOM, panX: 0, panY: 0 })}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true"><circle cx="12" cy="12" r="7" /><circle cx="12" cy="12" r="2" /><path d="M12 2v3M12 19v3M2 12h3M19 12h3" /></svg>
        </button>
      </div>
      {options.map && tilesLoading() && <div className="glass coords num" style={{ left: "50%", top: 60, bottom: "auto", transform: "translateX(-50%)" }}>{t("Baixando o mapa…")}</div>}
      {cursor && (
        <div className="glass coords num">
          {Math.round(cursor[0])}, {Math.round(cursor[1])}
        </div>
      )}
      <div className="map-bar">
        <div className="glass segmented">
          {(["map", "polygon"] as const).map((key) => (
            <button key={key} type="button" className={`ptab${options[key] ? " on" : ""}`} onClick={() => onOptions({ ...options, [key]: !options[key] })}>
              {key === "map" ? t("Mapa") : t("Polígono")}
            </button>
          ))}
        </div>
        <div className="spacer" />
        <label className="glass field-glass">
          <span className="lbl">Z</span>
          <input className="num" type="text" inputMode="numeric" value={zRange} aria-label={t("Filtro de altura Z")} onChange={(event) => onZRange(Number(event.target.value.replace(/\D/g, "")) || 0)} />
        </label>
        <label className="glass field-glass">
          <span className="lbl">{t("Zoom")}</span>
          <input
            type="range"
            min={0}
            max={100}
            value={Math.round(((Math.log(view.zoom) - Math.log(MIN_ZOOM)) / (Math.log(MAX_ZOOM) - Math.log(MIN_ZOOM))) * 100)}
            aria-label={t("Zoom do mapa")}
            onChange={(event) => setView((old) => ({ ...old, zoom: Math.exp(Math.log(MIN_ZOOM) + (Number(event.target.value) / 100) * (Math.log(MAX_ZOOM) - Math.log(MIN_ZOOM))) }))}
          />
        </label>
      </div>
    </section>
  );
}
