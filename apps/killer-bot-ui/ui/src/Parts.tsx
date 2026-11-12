import logo from "./assets/logo.png";
import { useEffect, useRef, useState } from "react";
import { clockTime, compact, signed, fraction, grouped, percent } from "./format";
import { useContextMenu } from "./ContextMenu";
import { useT } from "./i18n";
import { nearbyMenu } from "./menus";
import { Buffs, Clan, Gains, Inventory, Friends, Quests, Skills } from "./Tabs";
import type { ChatChannel, ChatLine, Command, Frame, Nearby, World } from "./model";

const GLYPHS = {
  star: "M12 3l2.5 5.5L20 9.5l-4 4 1 6-5-2.8L7 19.5l1-6-4-4 5.5-1z",
  plus: "M12 5v14M5 12h14",
  moon: "M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z",
  sun: "M12 4V2M12 22v-2M4 12H2M22 12h-2M5.6 5.6 4.2 4.2M19.8 19.8l-1.4-1.4M5.6 18.4l-1.4 1.4M19.8 4.2l-1.4 1.4M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z",
  gear: "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
};

export function Glyph({ name, size = 14, width = 2 }: { name: keyof typeof GLYPHS; size?: number; width?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={width} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={GLYPHS[name]} />
    </svg>
  );
}

export function Top({ frame, theme, lang, onTheme, onLang, send, onAdd, onSetup }: { frame: Frame; theme: "dark" | "light"; lang: string; onTheme: () => void; onLang: () => void; send: (command: Command) => void; onAdd: () => void; onSetup: () => void }) {
  const t = useT();
  return (
    <header className="top">
      <div className="brand">
        <img className="brand-logo" src={logo} alt="L2UnkBot" />
      </div>
      <nav className="sessions" aria-label={t("Sessões")}>
        {frame.sessions.map((session) => (
          <button key={session.id} type="button" className={`tab${session.id === frame.selected ? " on" : ""}`} onClick={() => send({ type: "selectSession", id: session.id })}>
            <span className="dot" style={{ background: session.state === "world" ? "var(--good)" : session.state === "connecting" ? "var(--warn)" : "var(--ghost)" }} />
            {session.label}
          </button>
        ))}
        <button type="button" className="ico" aria-label={t("Nova sessão")} onClick={onAdd}>
          <Glyph name="plus" />
        </button>
      </nav>
      <div className="spacer" />
      <div className="tools">
        {frame.world && (
          <button type="button" className={`bot${frame.world.botOn ? " on" : ""}`} onClick={() => send({ type: "toggleBot" })}>
            <span className="dot" />
            {frame.world.botOn ? t("Bot ligado") : t("Bot desligado")}
          </button>
        )}
        <button type="button" className="ico" aria-label={t("Alternar tema")} onClick={onTheme}>
          <Glyph name={theme === "dark" ? "moon" : "sun"} size={15} width={1.8} />
        </button>
        <button type="button" className="tab small" onClick={onLang} aria-label={t("Idioma")}>{lang === "pt" ? "PT" : "EN"}</button>
        <button type="button" className="ico" aria-label={t("Configuração")} onClick={onSetup}>
          <Glyph name="gear" size={15} width={1.8} />
        </button>
      </div>
    </header>
  );
}

function Gauge({ label, text, value, color, hint }: { label: string; text: string; value: number; color: string; hint?: string }) {
  return (
    <div className="gauge">
      <div className="gauge-top">
        <span className="gauge-label">
          <span className="gauge-dot" style={{ background: color }} />
          {label}
        </span>
        <span className="num gauge-text">
          {text}
          {hint && <span className="gauge-hint">{hint}</span>}
        </span>
      </div>
      <div className="track gauge-track">
        <div className="fill" style={{ width: `${value * 100}%`, background: `linear-gradient(90deg, ${color}, color-mix(in srgb, ${color} 62%, white))`, boxShadow: value > 0 ? `0 0 10px color-mix(in srgb, ${color} 45%, transparent)` : "none" }} />
      </div>
    </div>
  );
}

const KIND_LABEL = { player: "Jogador", npc: "NPC", monster: "Monstro", item: "Item" } as const;

export function Rail({ world, send }: { world: World; send: (command: Command) => void }) {
  const t = useT();
  const me = world.character;
  const target = world.target;
  const { show, menu } = useContextMenu();
  return (
    <aside className="rail">
      <section className="card">
        <div className="head">
          <b>{me.name}</b>
          <span className="pill num">Lv {me.level}</span>
        </div>
        <div className="gauges">
          <Gauge label="CP" text={`${grouped(me.cp.current)} / ${grouped(me.cp.max)}`} value={fraction(me.cp.current, me.cp.max)} color="var(--cp)" hint={` ${Math.round(fraction(me.cp.current, me.cp.max) * 100)}%`} />
          <Gauge label="HP" text={`${grouped(me.hp.current)} / ${grouped(me.hp.max)}`} value={fraction(me.hp.current, me.hp.max)} color="var(--hp)" hint={` ${Math.round(fraction(me.hp.current, me.hp.max) * 100)}%`} />
          <Gauge label="MP" text={`${grouped(me.mp.current)} / ${grouped(me.mp.max)}`} value={fraction(me.mp.current, me.mp.max)} color="var(--mp)" hint={` ${Math.round(fraction(me.mp.current, me.mp.max) * 100)}%`} />
          <Gauge label="XP" text={percent(me.xpPercent)} value={me.xpPercent / 100} color="var(--xp)" />
        </div>
      </section>
      <section className="card">
        <div className="lbl" style={{ marginBottom: 8 }}>{t("Alvo")}</div>
        {target ? (
          <>
            <div className="target-name">
              <b>{target.name}</b>
              <span className="pill">{target.dead ? t("Morto") : t(KIND_LABEL[target.kind])}</span>
            </div>
            <Gauge label="HP" text={target.hpPercent === null ? "—" : `${target.hpPercent}%`} value={(target.hpPercent ?? 0) / 100} color="var(--hp)" />
            <div className="target-sub">
              {t("Distância")} <span className="num">{grouped(target.distance)}</span>
            </div>
          </>
        ) : (
          <div className="target-empty">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true"><circle cx="12" cy="12" r="8" /><circle cx="12" cy="12" r="2.5" /></svg>
            <span>{t("Nenhum alvo")}</span>
          </div>
        )}
      </section>
      {world.party.length > 0 && (
        <section className="card" style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <div className="lbl">{t("Grupo")} <span className="num">{world.party.length}</span></div>
          {world.party.map((member) => (
            <div
              key={member.id}
              className="party-member"
              onClick={() => send({ type: "selectTarget", id: member.id })}
              onContextMenu={(event) =>
                show(event, [
                  { label: t("Selecionar"), run: () => send({ type: "selectTarget", id: member.id }) },
                  { label: t("Tornar líder"), run: () => send({ type: "partyLeader", name: member.name }) },
                  { label: t("Expulsar"), danger: true, run: () => send({ type: "partyKick", name: member.name }) },
                  { divider: true, label: "", run: () => undefined },
                  { label: t("Sair do grupo"), run: () => send({ type: "partyLeave" }) },
                  { label: t("Dissolver grupo"), danger: true, run: () => send({ type: "partyDismiss" }) },
                ])
              }
            >
              <div className="top-row"><span>{member.name}</span><span className="num dim">{Math.round(fraction(member.hp.current, member.hp.max) * 100)}%</span></div>
              <div className="track"><div className="fill" style={{ width: `${fraction(member.hp.current, member.hp.max) * 100}%`, background: "var(--hp)" }} /></div>
              <div className="track" style={{ height: 4 }}><div className="fill" style={{ width: `${fraction(member.mp.current, member.mp.max) * 100}%`, background: "var(--mp)" }} /></div>
            </div>
          ))}
        </section>
      )}
      {menu}
      <section className="card session">
        <div className="lbl">{t("Resumo da sessão")}</div>
        <div className="stat"><span>{t("Tempo ativo")}</span><span className="num">{world.uptime}</span></div>
        <div className="stat"><span>{t("Mortes")}</span><span className="num">{world.deaths}</span></div>
        <div className="stat"><span>{t("Adena / h")}</span><span className="num">{compact(world.adenaPerHour)}</span></div>
        <div className="stat last"><span>XP / h</span><span className="num">{percent(world.xpPerHour)}</span></div>
      </section>
    </aside>
  );
}

const CHANNELS: { key: ChatChannel; label: string; color: string }[] = [
  { key: "all", label: "Tudo", color: "" },
  { key: "system", label: "Sistema", color: "var(--ch-system)" },
  { key: "bot", label: "Bot", color: "var(--ch-bot)" },
  { key: "local", label: "Local", color: "var(--ch-local)" },
  { key: "trade", label: "Trade", color: "var(--ch-trade)" },
  { key: "party", label: "Grupo", color: "var(--ch-party)" },
  { key: "clan", label: "Clã", color: "var(--ch-clan)" },
  { key: "alliance", label: "Aliança", color: "var(--ch-alliance)" },
];

const EXTRA_COLORS: Partial<Record<ChatChannel, string>> = { shout: "var(--ch-shout)", whisper: "var(--ch-whisper)", hero: "var(--ch-hero)" };

const SENDABLE = CHANNELS.filter((channel) => ["local", "trade", "party", "clan"].includes(channel.key));

function colorOf(channel: ChatChannel): string {
  return CHANNELS.find((entry) => entry.key === channel)?.color || EXTRA_COLORS[channel] || "var(--ch-local)";
}

export function Chat({ lines, send }: { lines: ChatLine[]; send: (command: Command) => void }) {
  const t = useT();
  const [tab, setTab] = useState<ChatChannel>("all");
  const [channel, setChannel] = useState<ChatChannel>("local");
  const [text, setText] = useState("");
  const log = useRef<HTMLDivElement>(null);
  const shown = tab === "all" ? lines : lines.filter((line) => line.channel === tab);

  useEffect(() => {
    const node = log.current;
    if (node) node.scrollTop = node.scrollHeight;
  }, [shown.length, tab]);

  const submit = () => {
    const value = text.trim();
    if (!value) return;
    send({ type: "say", channel, text: value });
    setText("");
  };

  return (
    <section className="chat" aria-label="Chat">
      <div className="chat-tabs" role="tablist">
        {CHANNELS.map((entry) => (
          <button key={entry.key} type="button" role="tab" aria-selected={tab === entry.key} className={`ctab${tab === entry.key ? " on" : ""}`} onClick={() => setTab(entry.key)}>
            {entry.color && <span className="dot" style={{ background: entry.color }} />}
            {t(entry.label)}
          </button>
        ))}
      </div>
      <div className="chat-log" ref={log}>
        {shown.map((line, index) => (
          <div className="msg" key={index}>
            <span className="num time">{clockTime(line.at)}</span>
            {line.sender && (
              <>
                <b style={{ color: colorOf(line.channel) }}>{line.sender}</b> <span className="sep">·</span>{" "}
              </>
            )}
            <span style={line.sender ? undefined : { color: colorOf(line.channel) }}>{line.text}</span>
          </div>
        ))}
      </div>
      <form
        className="chat-input"
        onSubmit={(event) => {
          event.preventDefault();
          submit();
        }}
      >
        <label className="channel">
          <span className="dot" style={{ background: colorOf(channel) }} />
          <select aria-label={t("Canal")} value={channel} onChange={(event) => setChannel(event.target.value as ChatChannel)}>
            {SENDABLE.map((entry) => (
              <option key={entry.key} value={entry.key}>{t(entry.label)}</option>
            ))}
          </select>
        </label>
        <input className="say" placeholder={t("Escreva uma mensagem")} aria-label={t("Mensagem")} value={text} onChange={(event) => setText(event.target.value)} />
        <button type="submit" className="send">{t("Enviar")}</button>
      </form>
    </section>
  );
}

const SIDE_TABS = ["Char", "Inventário", "Skills", "Buffs", "Ao redor", "Clã", "Amigos", "Quests", "Diálogo", "Ganhos"] as const;

function Stat({ label, value, last, accent }: { label: string; value: string; last?: boolean; accent?: boolean }) {
  return (
    <div className={`stat${last ? " last" : ""}`}>
      <span>{label}</span>
      <span className="num" style={accent ? { color: "var(--warn)" } : undefined}>{value}</span>
    </div>
  );
}

function Tile({ label, value, small }: { label: string; value: string; small?: boolean }) {
  return (
    <div className={`tile${small ? " small" : ""}`}>
      <div className="lbl">{label}</div>
      <div className="num">{value}</div>
    </div>
  );
}

function CharPanel({ world }: { world: World }) {
  const t = useT();
  const me = world.character;
  const s = me.stats;
  return (
    <div className="side-body">
      <div className="who">
        <b>{me.name}</b>
        <div>
          {me.className} · <span className="num">Lv {me.level}</span>
          {me.title ? ` · ${me.title}` : ""}
        </div>
      </div>
      <div className="section">
        <div className="lbl">{t("Combate")}</div>
        <div className="grid2">
          <Stat label="P. Atk." value={grouped(s.pAtk)} />
          <Stat label="M. Atk." value={grouped(s.mAtk)} />
          <Stat label="P. Def." value={grouped(s.pDef)} />
          <Stat label="M. Def." value={grouped(s.mDef)} />
          <Stat label={t("Precisão")} value={grouped(s.accuracy)} />
          <Stat label={t("Evasão")} value={grouped(s.evasion)} />
          <Stat label={t("Crítico")} value={grouped(s.critical)} />
          <Stat label={t("Velocidade")} value={grouped(s.speed)} />
          <Stat label="Atk. Spd." value={grouped(s.atkSpd)} last />
          <Stat label="Cast. Spd." value={grouped(s.castSpd)} last />
        </div>
      </div>
      <div className="section">
        <div className="lbl" style={{ marginBottom: 8 }}>{t("Atributos")}</div>
        <div className="grid3">
          <Tile label="STR" value={String(s.str)} />
          <Tile label="DEX" value={String(s.dex)} />
          <Tile label="CON" value={String(s.con)} />
          <Tile label="INT" value={String(s.int)} />
          <Tile label="WIT" value={String(s.wit)} />
          <Tile label="MEN" value={String(s.men)} />
        </div>
        {world.henna && (
          <Stat label={t("Dyes")} value={`${world.henna.used}/${world.henna.slots}${world.henna.bonuses.map(([name, value]) => ` · ${name} ${value > 0 ? "+" : ""}${value}`).join("")}`} last />
        )}
      </div>
      <div className="section">
        <div className="lbl">{t("Reputação")}</div>
        <Stat label="Karma" value={grouped(me.karma)} />
        <Stat label="PvP / PK" value={`${grouped(me.pvp)} / ${grouped(me.pk)}`} />
        <Stat label="Recommend" value={`${me.recommendHave} / ${me.recommendLeft}`} />
        <Stat label={t("Carga")} value={`${grouped(s.load)} / ${grouped(s.maxLoad)}`} />
        <Stat label="SP" value={grouped(s.sp)} />
        <Stat label="Adena" value={grouped(me.adena)} last accent />
      </div>
      <div className="section">
        <div className="lbl" style={{ marginBottom: 8 }}>{t("Posição")}</div>
        <div className="grid3">
          <Tile label="X" value={grouped(me.position[0])} small />
          <Tile label="Y" value={grouped(me.position[1])} small />
          <Tile label="Z" value={grouped(me.position[2])} small />
        </div>
      </div>
    </div>
  );
}

function NearbyPanel({ world, send, onInfo }: { world: World; send: (command: Command) => void; onInfo: () => void }) {
  const t = useT();
  const [kind, setKind] = useState<Nearby["kind"]>("player");
  const { show, menu } = useContextMenu();
  const list = world.nearby.filter((entry) => entry.kind === kind).sort((a, b) => a.distance - b.distance).slice(0, 300);
  const labels: [Nearby["kind"], string][] = [["player", t("Jogadores")], ["npc", "NPCs"], ["monster", t("Monstros")], ["item", t("Itens")]];
  const sub = (entry: Nearby) => (kind === "player" ? [entry.className, entry.clan].filter(Boolean).join(" · ") : kind === "item" ? [entry.addName, `x${grouped(entry.count)}`].filter(Boolean).join(" ") : entry.title);
  return (
    <div className="side-body">
      <div className="chat-tabs" style={{ padding: 0, border: 0, flexWrap: "wrap" }}>
        {labels.map(([key, label]) => (
          <button key={key} type="button" className={`ctab${kind === key ? " on" : ""}`} onClick={() => setKind(key)}>
            {label} <span className="num">{world.nearby.filter((entry) => entry.kind === key).length}</span>
          </button>
        ))}
      </div>
      <div className="table">
        <div className="row head-row" style={{ gridTemplateColumns: "1fr 64px 56px" }}>
          <span>{t("Nome")}</span>
          <span className="r">{t("Dist.")}</span>
          <span className="r">Δz</span>
        </div>
        {list.length === 0 && <div className="empty">{t("Nada por aqui")}</div>}
        {list.map((entry) => (
          <div
            key={entry.id}
            className={`row item${world.target?.id === entry.id ? " on" : ""}`}
            style={{ gridTemplateColumns: "1fr 64px 56px" }}
            onClick={() => send({ type: "selectTarget", id: entry.id })}
            onDoubleClick={() => send({ type: "selectTarget", id: entry.id })}
            onContextMenu={(event) => show(event, nearbyMenu(entry, t, send, onInfo))}
          >
            <span className="cell">
              <span className="cut">{entry.name}</span>
              <span className="dim cut">{sub(entry)}</span>
            </span>
            <span className="r num">{grouped(entry.distance)}</span>
            <span className="r num dim">{signed(entry.z - world.character.position[2])}</span>
          </div>
        ))}
      </div>
      {menu}
    </div>
  );
}

export function Side({ world, send, onInfo }: { world: World; send: (command: Command) => void; onInfo: () => void }) {
  const t = useT();
  const [tab, setTab] = useState<(typeof SIDE_TABS)[number]>("Char");
  useEffect(() => {
    if (tab === "Clã") send({ type: "requestClan" });
    if (tab === "Quests") send({ type: "requestQuests" });
    if (tab === "Amigos") send({ type: "requestFriends" });
  }, [tab]);
  return (
    <aside className="side">
      <div className="side-tabs" role="tablist">
        {SIDE_TABS.map((label) => (
          <button key={label} type="button" role="tab" aria-selected={tab === label} className={`ptab${tab === label ? " on" : ""}`} onClick={() => setTab(label)}>
            {t(label)}
          </button>
        ))}
      </div>
      {tab === "Char" && <CharPanel world={world} />}
      {tab === "Ao redor" && <NearbyPanel world={world} send={send} onInfo={onInfo} />}
      {tab === "Inventário" && <Inventory world={world} send={send} />}
      {tab === "Skills" && <Skills world={world} send={send} />}
      {tab === "Buffs" && <Buffs world={world} />}
      {tab === "Clã" && <Clan world={world} />}
      {tab === "Amigos" && <Friends world={world} send={send} />}
      {tab === "Quests" && <Quests world={world} send={send} />}
      {tab === "Ganhos" && <Gains world={world} send={send} />}
      {tab === "Diálogo" && (
        <div className="side-body">
          <div className="empty">{world.npc ? t("O diálogo está aberto numa janela") : t("Clique num NPC no mapa para conversar")}</div>
        </div>
      )}
    </aside>
  );
}
