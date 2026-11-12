import { startDrag } from "./dnd";
import { useState } from "react";
import { useContextMenu } from "./ContextMenu";
import { useT } from "./i18n";
import { grouped } from "./format";
import { icons } from "./icons";
import { ItemName } from "./ItemName";
import type { Command, World } from "./model";

type Send = (command: Command) => void;

function Icon({ src }: { src: string }) {
  const [broken, setBroken] = useState(false);
  if (broken) return <span className="icon blank" />;
  return <img className="icon" src={src} alt="" loading="lazy" onError={() => setBroken(true)} />;
}

function time(seconds: number): string {
  if (seconds < 0) return "∞";
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

function Switch({ labels, value, onChange }: { labels: string[]; value: number; onChange: (index: number) => void }) {
  return (
    <div className="subtabs" role="tablist">
      {labels.map((label, index) => (
        <button key={label} type="button" role="tab" aria-selected={value === index} className={`ptab${value === index ? " on" : ""}`} onClick={() => onChange(index)}>
          {label}
        </button>
      ))}
    </div>
  );
}

export function Inventory({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const [mode, setMode] = useState(0);
  const { show, menu } = useContextMenu();
  const equipped = world.inventory.filter((item) => item.equipped);
  const list = mode === 1 ? equipped : mode === 2 ? [] : world.inventory;
  return (
    <div className="side-body">
      <Switch labels={[t("Itens"), t("Equipados"), "Quest"]} value={mode} onChange={setMode} />
      {equipped.length > 0 && (
        <div className="slots">
          {equipped.map((item) => (
            <div key={item.id} className="slot" title={`${item.name} ${item.addName}`.trim()}>
              <img src={icons.item(item.itemId)} alt={item.name} />
            </div>
          ))}
        </div>
      )}
      <div className="lbl">
        {t("Itens")} <span className="num">{world.inventory.length}</span>
      </div>
      <div className="table">
        {list.length === 0 && <div className="empty">{mode === 2 ? t("Sem itens de quest") : t("Nada por aqui")}</div>}
        {list.map((item) => (
          <div
            key={item.id}
            className="row item"
            style={{ gridTemplateColumns: "1fr auto" }}
            draggable
            onDragStart={(event) => startDrag(event, { kind: "item", id: item.id })}
            onDoubleClick={() => send({ type: "useItem", id: item.id })}
            onContextMenu={(event) =>
              show(event, [
                { label: t("Usar"), run: () => send({ type: "useItem", id: item.id }) },
                { divider: true, label: "", run: () => undefined },
                { label: item.count > 1 ? t("Largar pilha") : t("Largar"), run: () => send({ type: "dropItem", id: item.id, count: item.count }) },
                { label: t("Cristalizar"), run: () => send({ type: "crystallize", id: item.id, count: item.count }) },
                { label: item.count > 1 ? t("Destruir pilha") : t("Destruir"), danger: true, run: () => send({ type: "destroyItem", id: item.id, count: item.count }) },
                { divider: true, label: "", run: () => undefined },
                { label: t("Ignorar este item"), run: () => send({ type: "ignoreItem", itemId: item.itemId }) },
              ])
            }
          >
            <span className="cell">
              <Icon src={icons.item(item.itemId)} />
              <span className="cut"><ItemName item={item} /></span>
              {item.equipped && <span className="badge">{t("Eq.")}</span>}
            </span>
            <span className="num r">{grouped(item.count)}</span>
          </div>
        ))}
      </div>
      {menu}
    </div>
  );
}

export function Skills({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const [passive, setPassive] = useState(0);
  const list = world.skills.filter((skill) => skill.passive === (passive === 1));
  return (
    <div className="side-body">
      <Switch labels={[t("Ativas"), t("Passivas")]} value={passive} onChange={setPassive} />
      <div className="table">
        {list.length === 0 && <div className="empty">Nada por aqui</div>}
        {list.map((skill) => (
          <div key={skill.id} className="row item" style={{ gridTemplateColumns: "1fr auto", opacity: skill.ready ? 1 : 0.5 }} draggable={!skill.passive} onDragStart={(event) => startDrag(event, { kind: "skill", id: skill.id })} onDoubleClick={() => !skill.passive && send({ type: "useSkill", id: skill.id })}>
            <span className="cell">
              <Icon src={icons.skill(skill.id, skill.level)} />
              <span className="cut">{skill.name}</span>
            </span>
            <span className="num r">Lv {skill.level}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function Buffs({ world }: { world: World }) {
  const t = useT();
  return (
    <div className="side-body">
      <div className="lbl">
        {t("Efeitos ativos")} <span className="num">{world.buffs.length}</span>
      </div>
      <div className="table">
        {world.buffs.length === 0 && <div className="empty">{t("Nenhum efeito ativo")}</div>}
        {world.buffs.map((buff, index) => (
          <div key={`${buff.skillId}-${index}`} className="row item" style={{ gridTemplateColumns: "1fr auto" }}>
            <span className="cell">
              <Icon src={icons.skill(buff.skillId, buff.level)} />
              <span className="cut">{buff.name}</span>
              <span className="num dim">Lv {buff.level}</span>
            </span>
            <span className="num r">{time(buff.seconds)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function Clan({ world }: { world: World }) {
  const t = useT();
  const clan = world.clan;
  if (!clan) return <div className="side-body"><div className="empty">{world.clanId === 0 ? t("Sem clã") : t("Carregando clã…")}</div></div>;
  const online = clan.members.filter((member) => member.online).length;
  const pair = (label: string, value: string) => (
    <div className="stat">
      <span>{label}</span>
      <span className="num">{value}</span>
    </div>
  );
  return (
    <div className="side-body">
      <div className="who">
        <b>{clan.name}</b>
        <div>{t("Líder")} {clan.leader}</div>
      </div>
      <div className="grid2">
        {pair(t("Nível"), String(clan.level))}
        {pair(t("Guerra"), clan.atWar ? t("Sim") : t("Não"))}
        {pair(t("Castelo"), clan.castle ?? (clan.castleId ? `#${clan.castleId}` : "—"))}
        {pair("Hall", clan.hideoutId ? `#${clan.hideoutId}` : "—")}
        {pair(t("Aliança"), clan.ally || "—")}
      </div>
      <div className="lbl">
        {t("Membros")} <span className="num">{online}/{clan.members.length}</span>
      </div>
      <div className="table">
        {clan.members.map((member) => (
          <div key={member.name} className="row item" style={{ gridTemplateColumns: "1fr auto" }}>
            <span className="cell">
              <span className="dot" style={{ background: member.online ? "var(--good)" : "var(--ghost)" }} />
              <span className="cut">{member.name}</span>
              <span className="dim cut">{member.className}</span>
            </span>
            <span className="num r">Lv {member.level}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function Gains({ world, send }: { world: World; send: Send }) {
  const t = useT();
  const g = world.gains;
  const stat = (label: string, value: string, last = false) => (
    <div className={`stat${last ? " last" : ""}`}>
      <span>{label}</span>
      <span className="num">{value}</span>
    </div>
  );
  return (
    <div className="side-body">
      <div className="grid2">
        {stat("XP / h", grouped(g.expPerHour))}
        {stat("SP / h", grouped(g.spPerHour))}
        {stat("Adena / h", grouped(g.adenaPerHour))}
        {stat("Adena", grouped(g.adenaTotal))}
        {stat(t("Mobs ignorados"), String(g.badMobs), true)}
      </div>
      <button type="button" className="btn" onClick={() => send({ type: "clearStats" })}>{t("Zerar ganhos")}</button>
      <div className="lbl">{t("Drops")}</div>
      <div className="table">
        {world.loot.length === 0 && <div className="empty">{t("Nenhum drop ainda")}</div>}
        {world.loot.map((row, index) => (
          <div key={index} className="row item" style={{ gridTemplateColumns: "1fr auto" }}>
            <span className="cell">
              <Icon src={icons.item(row.itemId)} />
              <span className="cut"><ItemName item={row} /></span>
              <span className="dim cut">{row.source}</span>
            </span>
            <span className="num r">{grouped(row.count)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function Quests({ world, send }: { world: World; send: Send }) {
  const t = useT();
  return (
    <div className="side-body">
      <div className="head-line">
        <span className="lbl">Quests <span className="num">{world.windows.quests.length}</span></span>
        <button type="button" className="btn" onClick={() => send({ type: "requestQuests" })}>{t("Atualizar")}</button>
      </div>
      <div className="table">
        {world.windows.quests.length === 0 && <div className="empty">{t("Nenhuma quest ativa.")}</div>}
        {world.windows.quests.map((quest) => (
          <div key={quest.questId} className="row item quest-row" style={{ gridTemplateColumns: "1fr auto" }}>
            <span className="quest-text">
              <span>{quest.name}</span>
              {quest.step && <span className="dim">{quest.step}</span>}
            </span>
            <button
              type="button"
              className="btn"
              onClick={() => {
                send({ type: "abortQuest", questId: quest.questId });
                send({ type: "requestQuests" });
              }}
            >
              {t("Abandonar")}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

export function Friends({ world, send }: { world: World; send: Send }) {
  const t = useT();
  return (
    <div className="side-body">
      <div className="head-line">
        <span className="lbl">{t("Amigos")} <span className="num">{world.friends.filter((friend) => friend.online).length}/{world.friends.length}</span></span>
        <button type="button" className="btn" onClick={() => send({ type: "requestFriends" })}>{t("Atualizar")}</button>
      </div>
      <div className="table">
        {world.friends.length === 0 && <div className="empty">{t("Nenhum amigo na lista.")}</div>}
        {world.friends.map((friend) => (
          <div key={friend.name} className="row item quest-row" style={{ gridTemplateColumns: "1fr auto" }}>
            <span className="quest-text">
              <span>{friend.name}</span>
              <span className={friend.online ? "ok" : "dim"}>{friend.online ? t("Online") : t("Offline")}</span>
            </span>
            <button type="button" className="btn" onClick={() => send({ type: "deleteFriend", name: friend.name })}>{t("Remover")}</button>
          </div>
        ))}
      </div>
    </div>
  );
}
