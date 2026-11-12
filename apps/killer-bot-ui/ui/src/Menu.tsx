import { useEffect, useRef, useState } from "react";
import { useT } from "./i18n";

export type Action =
  | "pet"
  | "actions"
  | "macros"
  | "setup"
  | "exit"
  | "cancelTarget"
  | "blacklistTarget"
  | "targetInfo"
  | "toggleBot"
  | "restart"
  | "logout"
  | "disconnect"
  | "options"
  | "setScript"
  | "startScript"
  | "stopScript"
  | "scriptEditor"
  | "scriptDebugger"
  | "packets"
  | "about"
  | "updates"
  | "language";

type Entry = { label: string; action: Action; hint?: string } | "divider";

const MENUS: { label: string; direct?: Action; items: Entry[] }[] = [
  {
    label: "Arquivo",
    items: [
      { label: "Janela do pet", action: "pet" },
      { label: "Ações", action: "actions" },
      { label: "Macros", action: "macros" },
      "divider",
      { label: "Configuração", action: "setup" },
      "divider",
      { label: "Sair", action: "exit" },
    ],
  },
  {
    label: "Comandos",
    items: [
      { label: "Cancelar alvo", action: "cancelTarget" },
      { label: "Pôr alvo na lista negra", action: "blacklistTarget" },
      { label: "Informações do alvo", action: "targetInfo" },
      "divider",
      { label: "Ligar ou desligar o bot", action: "toggleBot", hint: "Alt+B" },
      "divider",
      { label: "Reiniciar", action: "restart" },
      { label: "Voltar ao personagem", action: "logout" },
      "divider",
      { label: "Desconectar", action: "disconnect" },
    ],
  },
  { label: "Opções do bot", direct: "options", items: [] },
  {
    label: "Scripts",
    items: [
      { label: "Definir script principal", action: "setScript" },
      { label: "Iniciar script", action: "startScript", hint: "Alt+R" },
      { label: "Parar script", action: "stopScript", hint: "Alt+S" },
      "divider",
      { label: "Editor de scripts", action: "scriptEditor" },
      { label: "Depurador", action: "scriptDebugger" },
      "divider",
      { label: "Janela de pacotes", action: "packets" },
    ],
  },
  {
    label: "Ajuda",
    items: [
      { label: "Sobre", action: "about" },
      { label: "Verificar dados do jogo", action: "updates" },
      "divider",
      { label: "Idioma", action: "language" },
    ],
  },
];

export function Menu({ onAction }: { onAction: (action: Action) => void }) {
  const t = useT();
  const [open, setOpen] = useState<number | null>(null);
  const holder = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open === null) return;
    const away = (event: MouseEvent) => {
      if (holder.current && !holder.current.contains(event.target as Node)) setOpen(null);
    };
    const escape = (event: KeyboardEvent) => event.key === "Escape" && setOpen(null);
    window.addEventListener("mousedown", away);
    window.addEventListener("keydown", escape);
    return () => {
      window.removeEventListener("mousedown", away);
      window.removeEventListener("keydown", escape);
    };
  }, [open]);

  return (
    <div className="menu-bar" ref={holder} role="menubar">
      {MENUS.map((menu, index) => (
        <div key={menu.label} className="menu-slot">
          <button
            type="button"
            role="menuitem"
            aria-haspopup={menu.items.length > 0}
            aria-expanded={open === index}
            className={`menu${open === index ? " on" : ""}`}
            onClick={() => {
              if (menu.direct) {
                setOpen(null);
                onAction(menu.direct);
              } else setOpen(open === index ? null : index);
            }}
            onMouseEnter={() => open !== null && menu.items.length > 0 && setOpen(index)}
          >
            {t(menu.label)}
          </button>
          {open === index && (
            <div className="dropdown" role="menu">
              {menu.items.map((entry, position) =>
                entry === "divider" ? (
                  <div key={position} className="ctx-div" />
                ) : (
                  <button
                    key={entry.action}
                    type="button"
                    role="menuitem"
                    className="ctx-item drop"
                    onClick={() => {
                      setOpen(null);
                      onAction(entry.action);
                    }}
                  >
                    <span>{t(entry.label)}</span>
                    {entry.hint && <span className="num dim">{entry.hint}</span>}
                  </button>
                ),
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
