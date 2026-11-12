import { useEffect, useRef, useState } from "react";
import { load, store, useT } from "./i18n";
import type { Command, World } from "./model";
import { useStatic } from "./staticData";
import { WindowFrame } from "./Windows";

type Send = (command: Command) => void;

const KIND_COLOR: Record<string, string> = { running: "var(--good)", waiting: "var(--good)", failed: "var(--bad)", finished: "var(--muted)", none: "var(--faint)" };

export function ScriptEditor({ world, send, onClose }: { world: World; send: Send; onClose: () => void }) {
  const t = useT();
  const script = world.script;
  const { examples } = useStatic();
  const [text, setTextState] = useState(() => load<string>("unkbot.script.text", ""));
  const [name, setName] = useState(() => load<string>("unkbot.script.name", ""));
  const setText = (value: string) => {
    setTextState(value);
    store("unkbot.script.text", value);
  };
  const seen = useRef<number>(script.loaded?.seq ?? 0);

  useEffect(() => {
    if (script.loaded && script.loaded.seq !== seen.current) {
      seen.current = script.loaded.seq;
      setText(script.loaded.text);
      setName(script.loaded.name);
      store("unkbot.script.name", script.loaded.name);
    }
  }, [script.loaded]);

  const lines = Math.max(1, text.split("\n").length);
  const [scrollTop, setScrollTop] = useState(0);
  const mainTask = script.tasks.find((task) => !task.handler) ?? script.tasks[0];
  const activeLines = new Set(script.tasks.map((task) => task.line).filter((line) => line > 0));
  return (
    <WindowFrame
      title={t("Editor de scripts")}
      width={720}
      onClose={onClose}
      footer={
        <>
          <span className="grow" style={{ color: KIND_COLOR[script.kind], display: "flex", gap: 8, alignItems: "center" }}>
            <span className="dot" style={{ background: KIND_COLOR[script.kind] }} />
            {script.status}
            {script.message && <span className="dim">· {script.message}</span>}
          </span>
          <button type="button" className="btn" onClick={() => {
            const report = [
              `estado: ${script.kind} / ${script.status}${script.paused ? " (pausado)" : ""}`,
              `bot ligado: ${world.botOn ? "sim" : "não"}`,
              `personagem: ${world.character.name} Lv ${world.character.level} HP ${world.character.hp.current}/${world.character.hp.max} MP ${world.character.mp.current}/${world.character.mp.max}`,
              `skills ativas: ${world.catalog.skills.map(([id, skillName]) => `${id}=${skillName}`).join(", ") || "nenhuma"}`,
              `variáveis: ${script.variables.map((variable) => `${variable.name}=${variable.value}`).join(", ") || "nenhuma"}`,
              "--- script ---",
              text,
              "--- saída ---",
              ...script.log,
            ].join("\n");
            navigator.clipboard?.writeText(report);
          }}>{t("Copiar relatório")}</button>
          <button type="button" className="btn" onClick={() => send({ type: "stopScript" })}>{t("Parar")}</button>
          <button type="button" className="btn primary" disabled={!text.trim()} onClick={() => send({ type: "runScript", text })}>{t("Rodar")}</button>
        </>
      }
    >
      <div className="script-bar">
        <input className="say" style={{ height: 32, maxWidth: 220 }} placeholder={t("Nome do script")} value={name} onChange={(event) => { setName(event.target.value); store("unkbot.script.name", event.target.value); }} />
        <button type="button" className="btn" disabled={!name.trim()} onClick={() => send({ type: "saveScript", name, text })}>{t("Salvar")}</button>
        <select className="opt-input" style={{ width: 170 }} value="" onChange={(event) => {
          const example = examples.find((entry) => entry.title === event.target.value);
          if (example) {
            setText(example.body);
            setName(example.title);
          }
        }}>
          <option value="">{t("Exemplos")}</option>
          {examples.map((example) => <option key={example.title} value={example.title}>{example.title}</option>)}
        </select>
        <select className="opt-input" style={{ width: 170 }} value="" onChange={(event) => event.target.value && send({ type: "loadScript", name: event.target.value })}>
          <option value="">{script.files.length === 0 ? t("Nenhum salvo") : t("Meus scripts")}</option>
          {script.files.map((file) => <option key={file} value={file}>{file}</option>)}
        </select>
      </div>
      {mainTask && mainTask.line > 0 && (
        <div className="script-now num">
          {t("Linha")} {mainTask.line}: <b>{text.split("\n")[mainTask.line - 1]?.trim() ?? ""}</b>
          {mainTask.waitLeftMs !== null && <span className="dim"> · {t("esperando")} {(mainTask.waitLeftMs / 1000).toFixed(1)} s</span>}
        </div>
      )}
      <div className="script-log num" aria-label={t("Saída do script")}>
        {script.log.length === 0 ? <span className="dim">{t("A saída do script (print, erros) aparece aqui")}</span> : script.log.map((line, index) => <div key={index}>{line}</div>)}
      </div>
      <div className="editor">
        <div className="gutter num" aria-hidden="true" style={{ transform: `translateY(${-scrollTop}px)` }}>
          {Array.from({ length: lines }, (_, index) => (
            <div key={index} className={activeLines.has(index + 1) ? "now" : undefined}>{index + 1}</div>
          ))}
        </div>
        <div className="code-wrap">
          {mainTask && mainTask.line > 0 && <div className="now-line" style={{ top: 10 + (mainTask.line - 1) * 20 - scrollTop }} />}
          <textarea className="code num" spellCheck={false} value={text} onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)} onChange={(event) => setText(event.target.value)} placeholder={t("Escreva o script aqui")} />
        </div>
      </div>
    </WindowFrame>
  );
}

export function Debugger({ world, send, onClose }: { world: World; send: Send; onClose: () => void }) {
  const t = useT();
  const script = world.script;
  const [snapshot, setSnapshot] = useState<Record<string, string> | null>(null);
  const now = Object.fromEntries(script.variables.map((variable) => [variable.name, variable.value]));
  const changes: string[] = [];
  if (snapshot) {
    for (const [name, value] of Object.entries(now)) {
      if (!(name in snapshot)) changes.push(`+ ${name} = ${value}`);
      else if (snapshot[name] !== value) changes.push(`~ ${name}: ${snapshot[name]} → ${value}`);
    }
    for (const name of Object.keys(snapshot)) if (!(name in now)) changes.push(`- ${name}`);
  }
  return (
    <WindowFrame
      title={t("Depurador")}
      width={480}
      onClose={onClose}
      footer={
        <>
          <button type="button" className="btn" onClick={() => setSnapshot(now)}>{t("Guardar estado")}</button>
          <button type="button" className="btn primary" onClick={() => send({ type: "pauseScript", paused: !script.paused })}>{script.paused ? t("Continuar") : t("Pausar")}</button>
        </>
      }
    >
      <div className="head-line">
        <span style={{ color: KIND_COLOR[script.kind] }}>{script.status}</span>
        {script.paused && <span className="badge">{t("Pausado")}</span>}
      </div>
      <div className="table">
        {script.variables.length === 0 && <div className="empty">{t("Nenhuma variável")}</div>}
        {script.variables.map((variable) => (
          <div key={variable.name} className="row item" style={{ gridTemplateColumns: "1fr auto auto" }}>
            <span className="num">{variable.name}</span>
            <span className="num">{variable.value}</span>
            <span className="dim">{variable.kind}</span>
          </div>
        ))}
      </div>
      {snapshot && (
        <>
          <div className="lbl">{t("Desde o estado guardado")}</div>
          {changes.length === 0 && <div className="dim">{t("Nada mudou")}</div>}
          {changes.map((line) => <div key={line} className="num">{line}</div>)}
        </>
      )}
    </WindowFrame>
  );
}

export function ScriptState({ script, send }: { script: World["script"]; send: Send }) {
  const t = useT();
  const stored = load<string>("unkbot.script.text", "").split("\n");
  const main = script.tasks.find((task) => !task.handler) ?? script.tasks[0];
  const color = script.kind === "failed" ? "var(--bad)" : script.kind === "finished" ? "var(--muted)" : "var(--accent)";
  const code = main && main.line > 0 ? stored[main.line - 1]?.trim() : "";
  return (
    <span className="script-box">
    <span className="script-state num" style={{ color }} title={script.status}>
      Script: {script.status}
      {main && main.line > 0 && (
        <>
          {" · "}{t("linha")} {main.line}
          {code ? <><span> · </span><span className="cut-code">{code}</span></> : null}
          {main.waitLeftMs !== null && ` · ${(main.waitLeftMs / 1000).toFixed(1)} s`}
        </>
      )}
      {script.tasks.filter((task) => task.handler).length > 0 && ` · ${script.tasks.filter((task) => task.handler).length} ${t("evento(s) em andamento")}`}
    </span>
    {script.kind !== "finished" && script.kind !== "failed" && (
      <button type="button" className="foot-btn" title={script.paused ? t("Continuar") : t("Pausar")} onClick={() => send({ type: "pauseScript", paused: !script.paused })}>
        {script.paused ? t("Continuar") : t("Pausar")}
      </button>
    )}
    <button type="button" className="foot-btn stop" title={`${t("Parar o script")} (Alt+S)`} onClick={() => send({ type: "stopScript" })}>
      {t("Parar")}
    </button>
    </span>
  );
}
