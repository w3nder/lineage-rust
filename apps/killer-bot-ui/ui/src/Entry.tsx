import { useState } from "react";
import { grouped } from "./format";
import { useT } from "./i18n";
import { WindowFrame } from "./Windows";
import type { Command, Frame, Saved } from "./model";

const KEY = "unkbot.last";
const DEFAULT_HOST = "server.l2killer.com";
const DEFAULT_PORT = "2106";

type Last = { host: string; port: string; account: string; character: string };

function load(): Last {
  try {
    const raw = window.localStorage.getItem(KEY);
    if (raw) return { host: DEFAULT_HOST, port: DEFAULT_PORT, account: "", character: "", ...JSON.parse(raw) };
  } catch {
    return { host: DEFAULT_HOST, port: DEFAULT_PORT, account: "", character: "" };
  }
  return { host: DEFAULT_HOST, port: DEFAULT_PORT, account: "", character: "" };
}

function save(value: Last) {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(value));
  } catch {
    return;
  }
}

export function AccountForm({ status, saved: accounts, send, onDone }: { status: string; saved: Saved[]; send: (command: Command) => void; onDone: () => void }) {
  const t = useT();
  const [saved, setSaved] = useState<Last>(load);
  const [password, setPassword] = useState("");
  const [remember, setRemember] = useState(true);
  const ready = saved.host.trim() !== "" && saved.account.trim() !== "" && password !== "" && Number(saved.port) > 0;
  const custom = saved.host !== DEFAULT_HOST || saved.port !== DEFAULT_PORT;
  return (
    <div className="stage">
      <form
        className="panel narrow"
        onSubmit={(event) => {
          event.preventDefault();
          if (!ready) return;
          save(saved);
          send({ type: "addAccount", host: saved.host.trim(), port: Number(saved.port), account: saved.account.trim(), password, character: "", remember });
          setPassword("");
          onDone();
        }}
      >
        <h1>{t("Entrar no jogo")}</h1>
        <p className="hint">{t("Use a mesma conta e senha do jogo. O personagem você escolhe depois de entrar.")}</p>
        {accounts.length > 0 && (
          <div className="saved">
            <div className="lbl">{t("Contas salvas")}</div>
            {accounts.map((entry) => (
              <div key={entry.label} className="saved-row">
                <button type="button" className="main" disabled={entry.online || !entry.hasPassword} onClick={() => { send({ type: "connectSaved", label: entry.label }); onDone(); }}>
                  <span>{entry.character || entry.account}</span>
                  <small>{entry.hasPassword ? `${entry.account} · ${entry.host}` : t("senha não guardada, use o formulário")}</small>
                </button>
                {entry.online && <span className="badge">{t("Conectada")}</span>}
                <button type="button" className="ico" aria-label={`${t("Esquecer")} ${entry.account}`} title={t("Esquecer esta conta")} onClick={() => send({ type: "forgetSaved", label: entry.label })}>×</button>
              </div>
            ))}
          </div>
        )}
        <div className="form-stack">
          <label className="form-row">
            <span>{t("Conta")}</span>
            <input value={saved.account} autoComplete="username" autoFocus onChange={(event) => setSaved({ ...saved, account: event.target.value })} />
          </label>
          <label className="form-row">
            <span>{t("Senha")}</span>
            <input type="password" value={password} autoComplete="current-password" onChange={(event) => setPassword(event.target.value)} />
          </label>
        </div>
        <details className="advanced" open={custom}>
          <summary>{t("Servidor")} <span className="num dim">{saved.host}:{saved.port}</span></summary>
          <div className="form-grid">
            <label className="form-row">
              <span>{t("Endereço")}</span>
              <input value={saved.host} onChange={(event) => setSaved({ ...saved, host: event.target.value })} />
            </label>
            <label className="form-row">
              <span>{t("Porta")}</span>
              <input value={saved.port} inputMode="numeric" onChange={(event) => setSaved({ ...saved, port: event.target.value.replace(/\D/g, "") })} />
            </label>
          </div>
        </details>
        <label className="check">
          <input type="checkbox" checked={remember} onChange={(event) => setRemember(event.target.checked)} />
          {t("Guardar a senha no chaveiro deste computador")}
        </label>
        {status && <div className="alert">{status}</div>}
        <button type="submit" className="send wide" disabled={!ready}>{t("Entrar")}</button>
      </form>
    </div>
  );
}

export function Waiting({ frame, send }: { frame: Frame; send: (command: Command) => void }) {
  const t = useT();
  const label = frame.screen === "disconnected" ? t("Conexão encerrada") : t("Conectando…");
  return (
    <div className="stage">
      <div className="panel narrow center-text">
        <h1>{label}</h1>
        {frame.status && <div className="alert">{frame.status}</div>}
        <p className="hint">{frame.screen === "disconnected" ? t("Feche a sessão e entre de novo.") : t("Aguarde a resposta do servidor.")}</p>
        {frame.selected !== null && <button type="button" className="btn" onClick={() => send({ type: "closeSession", id: frame.selected as number })}>{frame.screen === "disconnected" ? t("Fechar sessão") : t("Cancelar")}</button>}
      </div>
    </div>
  );
}

export function Servers({ frame, send }: { frame: Frame; send: (command: Command) => void }) {
  const t = useT();
  return (
    <div className="stage">
      <div className="panel narrow">
        <h1>{t("Escolha o servidor")}</h1>
        <div className="pick-list">
          {frame.servers.length === 0 && <div className="empty">{t("Nenhum servidor na lista")}</div>}
          {frame.servers.map((server) => (
            <button key={server.id} type="button" className="pick" onClick={() => send({ type: "chooseServer", id: server.id })}>
              <span className="dot" style={{ background: server.online ? "var(--good)" : "var(--ghost)" }} />
              <span className="grow">{server.name || `${t("Servidor")} ${server.id}`}</span>
              <span className="num dim">{grouped(server.players)} / {grouped(server.maxPlayers)}</span>
            </button>
          ))}
        </div>
        {frame.selected !== null && <button type="button" className="btn" onClick={() => send({ type: "closeSession", id: frame.selected as number })}>{t("Voltar")}</button>}
      </div>
    </div>
  );
}

const RACES = ["Human", "Elf", "Dark Elf", "Orc", "Dwarf"];
const HAIR_COLORS = ["A", "B", "C", "D"];

function CreateCharacter({ status, send, onClose }: { status: string; send: (command: Command) => void; onClose: () => void }) {
  const t = useT();
  const [name, setName] = useState("");
  const [race, setRace] = useState(0);
  const [female, setFemale] = useState(false);
  const [mage, setMage] = useState(false);
  const [hairStyle, setHairStyle] = useState(0);
  const [hairColor, setHairColor] = useState(0);
  const [face, setFace] = useState(0);
  const select = (label: string, value: number, set: (value: number) => void, options: string[]) => (
    <label className="form-row">
      <span>{label}</span>
      <select className="opt-input" style={{ height: 38 }} value={value} onChange={(event) => set(Number(event.target.value))}>
        {options.map((option, index) => <option key={option} value={index}>{option}</option>)}
      </select>
    </label>
  );
  return (
    <WindowFrame
      title={t("Criar personagem")}
      width={420}
      onClose={onClose}
      footer={
        <>
          <button type="button" className="btn" onClick={onClose}>{t("Cancelar")}</button>
          <button type="button" className="btn primary" onClick={() => {
            send({ type: "createCharacter", name, race, female, mage, hairStyle, hairColor, face });
          }}>{t("Criar")}</button>
        </>
      }
    >
      <label className="form-row">
        <span>{t("Nome")}</span>
        <input value={name} maxLength={16} autoFocus onChange={(event) => setName(event.target.value)} />
      </label>
      <div className="form-grid2">
        {select(t("Raça"), race, setRace, RACES)}
        {select(t("Sexo"), female ? 1 : 0, (value) => setFemale(value === 1), [t("Masculino"), t("Feminino")])}
        {select(t("Tipo"), mage ? 1 : 0, (value) => setMage(value === 1), ["Fighter", "Mystic"])}
        {select(t("Estilo do cabelo"), hairStyle, setHairStyle, ["A", "B", "C", "D", "E", "F", "G"])}
        {select(t("Cor do cabelo"), hairColor, setHairColor, HAIR_COLORS)}
        {select(t("Rosto"), face, setFace, ["A", "B", "C"])}
      </div>
      {status && <div className="alert">{status}</div>}
    </WindowFrame>
  );
}

export function Characters({ frame, send }: { frame: Frame; send: (command: Command) => void }) {
  const t = useT();
  const [selected, setSelected] = useState<number | null>(null);
  const [confirm, setConfirm] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
  const chosen = frame.characters.find((entry) => entry.slot === selected) ?? null;
  return (
    <div className="stage">
      <div className="panel narrow">
        <h1>{t("Escolha o personagem")}</h1>
        <div className="pick-list">
          {frame.characters.length === 0 && <div className="empty">{t("Nenhum personagem nesta conta. Crie o primeiro!")}</div>}
          {frame.characters.map((entry) => (
            <button
              key={entry.slot}
              type="button"
              className={`pick${selected === entry.slot ? " on" : ""}`}
              onClick={() => { setSelected(entry.slot); setConfirm(null); }}
              onDoubleClick={() => send({ type: "chooseCharacter", slot: entry.slot })}
            >
              <span className="grow">
                <b>{entry.name}</b>
                <span className="dim"> · {entry.className}</span>
              </span>
              <span className="pill num">Lv {entry.level}</span>
            </button>
          ))}
        </div>
        {frame.status && !creating && <div className="alert">{frame.status}</div>}
        <div className="char-actions">
          {frame.selected !== null && <button type="button" className="btn" onClick={() => send({ type: "closeSession", id: frame.selected as number })}>{t("Voltar")}</button>}
          <button type="button" className="btn" onClick={() => { setCreating(true); send({ type: "requestNewCharacter" }); }}>{t("Criar")}</button>
          <button
            type="button"
            className={`btn${confirm !== null ? " danger" : ""}`}
            disabled={!chosen}
            onClick={() => {
              if (!chosen) return;
              if (confirm === chosen.slot) {
                send({ type: "deleteCharacter", slot: chosen.slot });
                setConfirm(null);
                setSelected(null);
              } else setConfirm(chosen.slot);
            }}
          >
            {confirm !== null ? t("Confirmar") : t("Excluir")}
          </button>
          <button type="button" className="btn primary grow" disabled={!chosen} onClick={() => chosen && send({ type: "chooseCharacter", slot: chosen.slot })}>{t("Jogar")}</button>
        </div>
      </div>
      {creating && <CreateCharacter status={frame.status} send={send} onClose={() => setCreating(false)} />}
    </div>
  );
}
