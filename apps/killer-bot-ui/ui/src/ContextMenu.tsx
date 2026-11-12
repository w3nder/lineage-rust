import { useEffect, useState, type ReactNode } from "react";

export type MenuItem = { label: string; run: () => void; danger?: boolean; divider?: boolean };

type Open = { x: number; y: number; items: MenuItem[] };

export function useContextMenu() {
  const [open, setOpen] = useState<Open | null>(null);
  useEffect(() => {
    if (!open) return;
    const close = () => setOpen(null);
    window.addEventListener("click", close);
    window.addEventListener("blur", close);
    window.addEventListener("keydown", (event) => event.key === "Escape" && close());
    return () => {
      window.removeEventListener("click", close);
      window.removeEventListener("blur", close);
    };
  }, [open]);
  const show = (event: React.MouseEvent, items: MenuItem[]) => {
    event.preventDefault();
    event.stopPropagation();
    setOpen({ x: Math.min(event.clientX, window.innerWidth - 200), y: Math.min(event.clientY, window.innerHeight - items.length * 32 - 16), items });
  };
  const menu: ReactNode = open && (
    <div className="ctx" style={{ left: open.x, top: open.y }} role="menu">
      {open.items.map((item, index) =>
        item.divider ? (
          <div key={index} className="ctx-div" />
        ) : (
          <button key={index} type="button" role="menuitem" className={`ctx-item${item.danger ? " danger" : ""}`} onClick={() => item.run()}>
            {item.label}
          </button>
        ),
      )}
    </div>
  );
  return { show, menu };
}
