import { useEffect, useRef, useState } from "react";
import { useT } from "./i18n";
import type { Command, World } from "./model";
import { WindowFrame } from "./Windows";

type Send = (command: Command) => void;

function parseId(text: string): number | null {
  const clean = text.trim().replace(/^0x/i, "");
  if (!clean) return null;
  const hex = /^0x/i.test(text.trim()) || /[a-f]/i.test(clean);
  const value = hex ? Number.parseInt(clean, 16) : Number.parseInt(clean, 10);
  return Number.isFinite(value) && value >= 0 && value <= 0xffff ? value : null;
}

export function PacketWindow({ world, send, onClose, onFilters, onConverter }: { world: World; send: Send; onClose: () => void; onFilters: () => void; onConverter: () => void }) {
  const t = useT();
  const p = world.packets;
  const [follow, setFollow] = useState(true);
  const [query, setQuery] = useState("");
  const [searchClient, setSearchClient] = useState(true);
  const [byName, setByName] = useState(false);
  const [format, setFormat] = useState("");
  const [hex, setHex] = useState("");
  const [toClient, setToClient] = useState(false);
  const [onlyVisible, setOnlyVisible] = useState(true);
  const list = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const node = list.current;
    if (follow && node) node.scrollTop = node.scrollHeight;
  }, [p.rows.length, follow]);

  const view = (change: Partial<{ filterOn: boolean; hideServer: boolean; hideClient: boolean }>) => send({ type: "pkView", filterOn: p.filterOn, hideServer: p.hideServer, hideClient: p.hideClient, ...change });
  const find = (direction: "first" | "next" | "prev") => send({ type: "pkSearch", query, client: searchClient, byName, direction });

  return (
    <WindowFrame title={t("Janela de pacotes")} width={860} onClose={onClose}>
      <div className="pk-toolbar">
        <button type="button" className={`btn${p.recording ? " primary" : ""}`} onClick={() => send({ type: "pkRecord", on: !p.recording })}>{p.recording ? t("Parar gravação") : t("Gravar")}</button>
        <button type="button" className="btn" onClick={() => send({ type: "pkClear" })}>{t("Limpar")}</button>
        <button type="button" className="btn" onClick={() => send({ type: "pkSave", onlyVisible })}>{t("Salvar log")}</button>
        <label className="check"><input type="checkbox" checked={onlyVisible} onChange={(event) => setOnlyVisible(event.target.checked)} />{t("só os visíveis")}</label>
        <button type="button" className="btn" onClick={() => send({ type: "pkLoad" })}>{t("Carregar log")}</button>
        <span className="spacer" />
        <button type="button" className="btn" onClick={onFilters}>{t("Filtros")}</button>
        <label className="check"><input type="checkbox" checked={p.filterOn} onChange={(event) => view({ filterOn: event.target.checked })} />{t("Usar filtros")}</label>
        <label className="check"><input type="checkbox" checked={p.hideServer} onChange={(event) => view({ hideServer: event.target.checked })} />{t("Esconder servidor")}</label>
        <label className="check"><input type="checkbox" checked={p.hideClient} onChange={(event) => view({ hideClient: event.target.checked })} />{t("Esconder cliente")}</label>
        <label className="check"><input type="checkbox" checked={follow} onChange={(event) => setFollow(event.target.checked)} />{t("Seguir o fim")}</label>
      </div>
      <div className="pk-search">
        <input className="say" style={{ height: 32 }} placeholder={t("Buscar por ID (0x05) ou nome")} value={query} onChange={(event) => setQuery(event.target.value)} />
        <div className="subtabs" style={{ width: 170 }}>
          <button type="button" className={`ptab${searchClient ? " on" : ""}`} onClick={() => setSearchClient(true)}>{t("Cliente")}</button>
          <button type="button" className={`ptab${!searchClient ? " on" : ""}`} onClick={() => setSearchClient(false)}>{t("Servidor")}</button>
        </div>
        <label className="check"><input type="checkbox" checked={byName} onChange={(event) => setByName(event.target.checked)} />{t("por nome")}</label>
        <button type="button" className="btn" onClick={() => find("first")}>{t("Primeiro")}</button>
        <button type="button" className="btn" onClick={() => find("prev")}>↑</button>
        <button type="button" className="btn" onClick={() => find("next")}>↓</button>
      </div>
      <div className="pk-grid">
        <div className="pk-list" ref={list}>
          <div className="row head-row" style={{ gridTemplateColumns: "64px 28px 110px 56px 1fr 56px", position: "sticky", top: 0, background: "var(--card)" }}>
            <span>#</span><span>C/S</span><span>{t("Hora")}</span><span className="r">{t("Bytes")}</span><span>{t("Nome")}</span><span className="r">Op</span>
          </div>
          {p.rows.map((row) => (
            <div key={row.number} className={`row item num${p.selected === row.number ? " on" : ""}`} style={{ gridTemplateColumns: "64px 28px 110px 56px 1fr 56px", fontSize: 12 }} onClick={() => send({ type: "pkSelect", number: row.number })}>
              <span>{row.number}</span>
              <span style={{ color: row.client ? "var(--accent)" : "var(--warn)" }}>{row.client ? "C" : "S"}</span>
              <span className="dim">{row.time}</span>
              <span className="r">{row.size}</span>
              <span className="cut" style={{ fontFamily: "Inter, sans-serif" }}>{row.name}</span>
              <span className="r dim">{row.opcode}</span>
            </div>
          ))}
          {p.rows.length === 0 && <div className="empty">{p.recording ? t("Aguardando pacotes…") : t("Clique em Gravar")}</div>}
        </div>
        <pre className="pk-detail num">{p.detail || " "}</pre>
      </div>
      <div className="pk-footer">
        <input className="say num" style={{ height: 32 }} placeholder={t("Formato (ex.: cdhS)")} value={format} onChange={(event) => setFormat(event.target.value)} />
        <button type="button" className="btn" onClick={() => send({ type: "pkParse", format })}>{t("Processar")}</button>
        <button type="button" className="btn" onClick={() => send({ type: "pkHelp" })}>?</button>
        <button type="button" className="btn" onClick={onConverter}>{t("Conversor")}</button>
      </div>
      <div className="pk-footer">
        <input className="say num" style={{ height: 32 }} placeholder={t("Bytes em hexadecimal para enviar")} value={hex} onChange={(event) => setHex(event.target.value)} />
        <div className="subtabs" style={{ width: 220 }}>
          <button type="button" className={`ptab${!toClient ? " on" : ""}`} onClick={() => setToClient(false)}>{t("Ao servidor")}</button>
          <button type="button" className={`ptab${toClient ? " on" : ""}`} onClick={() => setToClient(true)}>{t("Ao cliente")}</button>
        </div>
        <button type="button" className="btn primary" disabled={!hex.trim()} onClick={() => send({ type: "pkInject", hex, toClient })}>{t("Enviar")}</button>
      </div>
      <div className="dim" style={{ fontSize: 12 }}>
        {p.status} · {p.shown}/{p.total}
      </div>
    </WindowFrame>
  );
}

export function PacketFilters({ world, send, onClose }: { world: World; send: Send; onClose: () => void }) {
  const t = useT();
  const f = world.packets.filters;
  const [server, setServer] = useState("");
  const [client, setClient] = useState("");
  const apply = (next: { server?: number[]; client?: number[]; whitelist?: boolean }) =>
    send({ type: "pkFilters", server: next.server ?? f.server.map((entry) => entry.id), client: next.client ?? f.client.map((entry) => entry.id), whitelist: next.whitelist ?? f.whitelist });
  const column = (title: string, entries: { id: number; name: string }[], value: string, setValue: (value: string) => void, key: "server" | "client") => (
    <div className="pk-filter-col">
      <div className="lbl">{title}</div>
      <div className="idlist-add">
        <input className="opt-input num" placeholder="0x05" value={value} onChange={(event) => setValue(event.target.value)} />
        <button type="button" className="btn" disabled={parseId(value) === null} onClick={() => {
          const id = parseId(value);
          if (id !== null && !entries.some((entry) => entry.id === id)) apply({ [key]: [...entries.map((entry) => entry.id), id] });
          setValue("");
        }}>{t("Adicionar")}</button>
      </div>
      <div className="table" style={{ maxHeight: 220, overflow: "auto" }}>
        {entries.length === 0 && <div className="empty" style={{ padding: 12 }}>{t("Lista vazia")}</div>}
        {entries.map((entry) => (
          <div key={entry.id} className="row item" style={{ gridTemplateColumns: "60px 1fr auto" }}>
            <span className="num">{`0x${entry.id.toString(16).toUpperCase().padStart(2, "0")}`}</span>
            <span className="cut dim">{entry.name}</span>
            <button type="button" className="ico small" aria-label={t("Remover")} onClick={() => apply({ [key]: entries.filter((other) => other.id !== entry.id).map((other) => other.id) })}>×</button>
          </div>
        ))}
      </div>
      <button type="button" className="btn" onClick={() => apply({ [key]: [] })}>{t("Limpar lista")}</button>
    </div>
  );
  return (
    <WindowFrame
      title={t("Filtros de pacotes")}
      width={640}
      onClose={onClose}
      footer={
        <>
          <button type="button" className="btn" onClick={() => send({ type: "pkFiltersLoad" })}>{t("Carregar")}</button>
          <button type="button" className="btn" onClick={() => send({ type: "pkFiltersSave" })}>{t("Salvar")}</button>
          <button type="button" className="btn primary" onClick={onClose}>{t("Fechar")}</button>
        </>
      }
    >
      <label className="check"><input type="checkbox" checked={f.whitelist} onChange={(event) => apply({ whitelist: event.target.checked })} />{t("Lista branca: mostrar só os listados")}</label>
      <div className="two">
        {column(t("Do servidor"), f.server, server, setServer, "server")}
        {column(t("Do cliente"), f.client, client, setClient, "client")}
      </div>
    </WindowFrame>
  );
}

export function Converter({ world, send, onClose }: { world: World; send: Send; onClose: () => void }) {
  const t = useT();
  const [text, setText] = useState("");
  return (
    <WindowFrame title={t("Conversor")} width={460} onClose={onClose}>
      <input className="say" style={{ height: 34 }} placeholder={t("Número, texto ou bytes em hexadecimal")} value={text} onChange={(event) => { setText(event.target.value); send({ type: "pkConvert", text: event.target.value }); }} />
      <div className="table">
        {world.packets.converter.map(([label, value]) => (
          <div key={label} className="row item" style={{ gridTemplateColumns: "150px 1fr" }}>
            <span className="dim">{label}</span>
            <span className="num" style={{ overflowWrap: "anywhere" }}>{value}</span>
          </div>
        ))}
      </div>
    </WindowFrame>
  );
}
