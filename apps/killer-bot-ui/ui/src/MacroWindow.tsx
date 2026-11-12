import { useState } from "react";
import { startDrag } from "./dnd";
import { useT } from "./i18n";
import { icons } from "./icons";
import type { Command, Macro, World } from "./model";
import { WindowFrame } from "./Windows";

const LINES = 12;
const LINE_MAX = 32;
const NAME_MAX = 12;
const ACRONYM_MAX = 4;
const DESCRIPTION_MAX = 32;

type Draft = { id: number; name: string; acronym: string; description: string; icon: number; lines: string[] };

const blank = (): Draft => ({ id: 0, name: "", acronym: "", description: "", icon: 1, lines: Array(LINES).fill("") });
const draftOf = (macro: Macro): Draft => ({ ...macro, lines: Array.from({ length: LINES }, (_, index) => macro.lines[index] ?? "") });

export function MacroWindow({ world, send, onClose }: { world: World; send: (command: Command) => void; onClose: () => void }) {
  const t = useT();
  const [draft, setDraft] = useState<Draft | null>(null);
  const [confirming, setConfirming] = useState<number | null>(null);
  const running = world.macroRun;

  if (draft) {
    const set = (patch: Partial<Draft>) => setDraft({ ...draft, ...patch });
    const save = () => {
      send({ type: "saveMacro", id: draft.id, name: draft.name, acronym: draft.acronym, description: draft.description, icon: draft.icon, lines: draft.lines });
      setDraft(null);
    };
    return (
      <WindowFrame
        title={draft.id ? t("Editar macro") : t("Novo macro")}
        width={520}
        onClose={() => setDraft(null)}
        footer={
          <>
            <button type="button" className="btn" onClick={() => setDraft(null)}>{t("Cancelar")}</button>
            <button type="button" className="btn primary" disabled={!draft.name.trim() || !draft.acronym.trim()} onClick={save}>{t("Salvar")}</button>
          </>
        }
      >
        <div className="macro-head">
          <label className="field">
            <span>{t("Nome")}</span>
            <input value={draft.name} maxLength={NAME_MAX} onChange={(event) => set({ name: event.target.value })} />
          </label>
          <label className="field">
            <span>{t("Sigla")}</span>
            <input value={draft.acronym} maxLength={ACRONYM_MAX} onChange={(event) => set({ acronym: event.target.value })} />
          </label>
        </div>
        <label className="field">
          <span>{t("Descrição")}</span>
          <input value={draft.description} maxLength={DESCRIPTION_MAX} onChange={(event) => set({ description: event.target.value })} />
        </label>
        <div className="lbl" style={{ margin: "10px 0 6px" }}>{t("Ícone")}</div>
        <div className="slots">
          {Array.from({ length: 7 }, (_, index) => index + 1).map((icon) => (
            <button key={icon} type="button" className={`slot${draft.icon === icon ? " on" : ""}`} onClick={() => set({ icon })} aria-pressed={draft.icon === icon}>
              <img src={icons.texture(`L2UI.MacroWnd.Macro_Icon${icon}`)} alt="" />
            </button>
          ))}
        </div>
        <div className="lbl" style={{ margin: "10px 0 6px" }}>{t("Comandos")}</div>
        {draft.lines.map((line, index) => (
          <input
            key={index}
            className={`macro-line${running && running.id === draft.id && running.line === index ? " live" : ""}`}
            value={line}
            maxLength={LINE_MAX}
            placeholder={index === 0 ? "/useskill Wind Walk" : ""}
            onChange={(event) => set({ lines: draft.lines.map((current, at) => (at === index ? event.target.value : current)) })}
          />
        ))}
      </WindowFrame>
    );
  }

  return (
    <WindowFrame
      title="Macros"
      width={460}
      onClose={onClose}
      footer={<button type="button" className="btn primary" onClick={() => setDraft(blank())}>{t("Novo macro")}</button>}
    >
      {world.macros.length === 0 && <div className="empty">{t("Nenhum macro. Crie um para usar na barra de atalhos.")}</div>}
      {world.macros.map((macro) => {
        const live = running?.id === macro.id;
        return (
          <div key={macro.id} className={`macro-row${live ? " live" : ""}`} draggable onDragStart={(event) => startDrag(event, { kind: "macro", id: macro.id })}>
            <span className="slot"><img src={icons.texture(`L2UI.MacroWnd.Macro_Icon${macro.icon}`)} alt="" /></span>
            <span className="macro-name">
              <b>{macro.name}</b>
              <span className="dim">{macro.acronym}{macro.description ? ` · ${macro.description}` : ""}</span>
            </span>
            {confirming === macro.id ? (
              <>
                <button type="button" className="btn danger" onClick={() => { send({ type: "deleteMacro", id: macro.id }); setConfirming(null); }}>{t("Confirmar exclusão")}</button>
                <button type="button" className="btn" onClick={() => setConfirming(null)}>{t("Cancelar")}</button>
              </>
            ) : (
              <>
                <button type="button" className="btn" onClick={() => send(live ? { type: "stopMacro" } : { type: "runMacro", id: macro.id })}>{live ? t("Parar") : t("Executar")}</button>
                <button type="button" className="btn" onClick={() => setDraft(draftOf(macro))}>{t("Editar")}</button>
                <button type="button" className="btn" onClick={() => setConfirming(macro.id)}>{t("Excluir")}</button>
              </>
            )}
          </div>
        );
      })}
    </WindowFrame>
  );
}
