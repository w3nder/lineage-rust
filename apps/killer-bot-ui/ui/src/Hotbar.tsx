import { useEffect, useState } from "react";
import { acceptsDrag, readDrag, startDrag } from "./dnd";
import { useContextMenu } from "./ContextMenu";
import { grouped } from "./format";
import { useT } from "./i18n";
import { icons } from "./icons";
import type { Command, Shortcut, World } from "./model";

const PAGES = 10;
const PER_PAGE = 12;

function art(shortcut: Shortcut): string {
  switch (shortcut.kind) {
    case "item":
      return shortcut.itemId ? icons.item(shortcut.itemId) : "";
    case "skill":
      return icons.skill(shortcut.id, shortcut.level);
    case "action":
      return shortcut.icon ? icons.texture(shortcut.icon) : "";
    case "macro":
      return icons.texture(`L2UI.MacroWnd.Macro_Icon${shortcut.level}`);
    default:
      return "";
  }
}

export function Hotbar({ world, send }: { world: World; send: (command: Command) => void }) {
  const t = useT();
  const [page, setPage] = useState(0);
  const [over, setOver] = useState<number | null>(null);
  const { show, menu } = useContextMenu();
  const bySlot = new Map(world.shortcuts.map((shortcut) => [shortcut.slot, shortcut]));

  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      const match = /^F(\d{1,2})$/.exec(event.key);
      const number = match ? Number(match[1]) : 0;
      if (number < 1 || number > PER_PAGE || event.altKey || event.ctrlKey || event.metaKey) return;
      const target = event.target as HTMLElement | null;
      if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA")) return;
      event.preventDefault();
      send({ type: "useShortcut", slot: page * PER_PAGE + number - 1 });
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [page, send]);

  const drop = (slot: number) => (event: React.DragEvent) => {
    event.preventDefault();
    setOver(null);
    const dragged = readDrag(event);
    if (!dragged) return;
    send({ type: "registerShortcut", slot, kind: dragged.kind, id: dragged.id });
    if (dragged.from !== undefined && dragged.from !== slot) send({ type: "deleteShortcut", slot: dragged.from });
  };

  return (
    <div className="hotbar" aria-label={t("Barra de atalhos")}>
      <div className="hb-pager">
        <button type="button" className="hb-page" onClick={() => setPage((page + PAGES - 1) % PAGES)} aria-label={t("Página anterior")}>‹</button>
        <span className="num hb-num">{page + 1}<i>/{PAGES}</i></span>
        <button type="button" className="hb-page" onClick={() => setPage((page + 1) % PAGES)} aria-label={t("Próxima página")}>›</button>
      </div>
      <div className="hb-slots">
        {Array.from({ length: PER_PAGE }, (_, index) => {
          const slot = page * PER_PAGE + index;
          const shortcut = bySlot.get(slot);
          return (
            <button
              key={slot}
              type="button"
              className={`hb-slot${shortcut ? " filled" : ""}${over === slot ? " over" : ""}${shortcut && !shortcut.ready ? " cooling" : ""}${shortcut?.auto ? " auto" : ""}`}
              title={shortcut ? `${shortcut.name || `${shortcut.kind} #${shortcut.id}`}${shortcut.shot ? ` — ${shortcut.auto ? t("automático ligado") : t("clique direito liga o automático")}` : ""}` : t("Solte aqui uma skill, item, ação ou macro")}
              draggable={Boolean(shortcut)}
              onDragStart={(event) => shortcut && startDrag(event, { kind: shortcut.kind, id: shortcut.id, from: slot })}
              onDragEnd={(event) => {
                if (!shortcut || event.dataTransfer.dropEffect !== "none") return;
                if (shortcut.shot && shortcut.auto) send({ type: "toggleAutoShot", itemId: shortcut.itemId, enable: false });
                send({ type: "deleteShortcut", slot });
              }}
              onDragOver={(event) => {
                if (!acceptsDrag(event)) return;
                event.preventDefault();
                setOver(slot);
              }}
              onDragLeave={() => setOver(null)}
              onDrop={drop(slot)}
              onClick={() => shortcut && send({ type: "useShortcut", slot })}
              onContextMenu={(event) => {
                if (!shortcut) return;
                if (shortcut.shot) {
                  event.preventDefault();
                  send({ type: "toggleAutoShot", itemId: shortcut.itemId, enable: !shortcut.auto });
                  return;
                }
                show(event, [{ label: t("Remover atalho"), danger: true, run: () => send({ type: "deleteShortcut", slot }) }]);
              }}
            >
              <span className="hb-key">F{index + 1}</span>
              {shortcut && art(shortcut) && <img src={art(shortcut)} alt="" draggable={false} />}
              {shortcut?.kind === "macro" && <span className="hb-acr">{shortcut.acronym}</span>}
              {shortcut?.kind === "recipe" && <span className="hb-acr">R</span>}
              {shortcut?.count != null && <span className="hb-count">{grouped(shortcut.count)}</span>}
            </button>
          );
        })}
      </div>
      {world.macroRun && (
        <button type="button" className="btn" onClick={() => send({ type: "stopMacro" })}>
          {t("Parar macro")}
        </button>
      )}
      {menu}
    </div>
  );
}
