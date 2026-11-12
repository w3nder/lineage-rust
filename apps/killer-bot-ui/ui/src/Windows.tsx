import { useEffect, useRef, useState, type ReactNode } from "react";

export function WindowFrame({ title, onClose, width = 480, children, footer }: { title: string; onClose: () => void; width?: number; children: ReactNode; footer?: ReactNode }) {
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const drag = useRef<{ x: number; y: number; ox: number; oy: number } | null>(null);

  useEffect(() => {
    const move = (event: PointerEvent) => {
      const current = drag.current;
      if (current) setOffset({ x: current.ox + event.clientX - current.x, y: current.oy + event.clientY - current.y });
    };
    const up = () => {
      drag.current = null;
    };
    const escape = (event: KeyboardEvent) => event.key === "Escape" && onClose();
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    window.addEventListener("keydown", escape);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("keydown", escape);
    };
  }, [onClose]);

  return (
    <div className="win-layer">
      <div className="win" role="dialog" aria-label={title} style={{ width, transform: `translate(${offset.x}px, ${offset.y}px)` }}>
        <div className="win-head" onPointerDown={(event) => (drag.current = { x: event.clientX, y: event.clientY, ox: offset.x, oy: offset.y })}>
          <b>{title}</b>
          <button type="button" className="ico small" aria-label="Fechar" onPointerDown={(event) => event.stopPropagation()} onClick={onClose}>
            ×
          </button>
        </div>
        <div className="win-body">{children}</div>
        {footer && <div className="win-foot">{footer}</div>}
      </div>
    </div>
  );
}
