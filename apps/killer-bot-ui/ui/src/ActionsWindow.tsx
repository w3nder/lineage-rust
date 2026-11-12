import { startDrag } from "./dnd";
import { useState } from "react";
import { useT } from "./i18n";
import { icons } from "./icons";
import type { Command, World } from "./model";
import { useStatic } from "./staticData";
import { WindowFrame } from "./Windows";

export function ActionsWindow({ send, onClose }: { world?: World; send: (command: Command) => void; onClose: () => void }) {
  const t = useT();
  const { actions } = useStatic();
  const titles = ["Basic", "Party", "Social"];
  return (
    <WindowFrame title={t("Ações")} width={460} onClose={onClose}>
      {actions.length === 0 && <div className="empty">{t("Dados do jogo ainda não carregados.")}</div>}
      {titles.map((title, group) => {
        const list = actions.filter((action) => action.group === group);
        if (list.length === 0) return null;
        return (
          <div key={title}>
            <div className="lbl" style={{ marginBottom: 8 }}>{title}</div>
            <div className="slots">
              {list.map((action) => (
                <Icon key={action.id} name={action.name} icon={action.icon} id={action.id} onClick={() => send({ type: "action", id: action.id })} />
              ))}
            </div>
          </div>
        );
      })}
    </WindowFrame>
  );
}

function Icon({ name, icon, id, onClick }: { name: string; icon: string; id: number; onClick: () => void }) {
  const [broken, setBroken] = useState(false);
  return (
    <button type="button" className="slot" title={name} draggable onDragStart={(event) => startDrag(event, { kind: "action", id })} onClick={onClick}>
      {broken ? <span className="dim">{name.slice(0, 2)}</span> : <img src={icons.texture(icon)} alt={name} onError={() => setBroken(true)} />}
    </button>
  );
}
