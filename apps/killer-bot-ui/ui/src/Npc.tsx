import { useState, type ReactNode } from "react";
import { useT } from "./i18n";
import type { Command, HtmlNode, Npc } from "./model";
import { WindowFrame } from "./Windows";

type Edits = Record<string, string>;

function plain(nodes: HtmlNode[]): string {
  return nodes
    .map((node) => {
      if (node.t === "text") return node.text;
      if (node.t === "font" || node.t === "center" || node.t === "link") return plain(node.children);
      return "";
    })
    .join("");
}

function substitute(action: string, edits: Edits): string {
  let out = action;
  for (const [name, value] of Object.entries(edits)) out = out.split(`$${name}`).join(value);
  return out;
}

function render(nodes: HtmlNode[], edits: Edits, setEdit: (name: string, value: string) => void, run: (action: string | null) => void): ReactNode[] {
  return nodes.map((node, index) => {
    switch (node.t) {
      case "text": {
        const collapsed = node.text.replace(/\s+/g, " ");
        return collapsed.trim() === "" && collapsed !== " " ? null : <span key={index}>{collapsed}</span>;
      }
      case "br":
        return <br key={index} />;
      case "font":
        return (
          <span key={index} style={node.color ? { color: node.color } : undefined}>
            {render(node.children, edits, setEdit, run)}
          </span>
        );
      case "link": {
        const label = plain(node.children).replace(/\s+/g, " ").trim();
        return (
          <button key={index} type="button" className="npc-link" disabled={node.action === null} onClick={() => run(node.action)}>
            {label}
          </button>
        );
      }
      case "center":
        return (
          <div key={index} className="npc-center">
            {render(node.children, edits, setEdit, run)}
          </div>
        );
      case "table":
        return (
          <table key={index} className="npc-table" style={node.width ? { maxWidth: node.width * 1.4 } : undefined}>
            <tbody>
              {node.rows.map((row, r) => (
                <tr key={r}>
                  {row.map((cell, c) => (
                    <td key={c}>{render(cell.children, edits, setEdit, run)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        );
      case "button":
        return (
          <button key={index} type="button" className="btn" disabled={node.action === null} onClick={() => run(node.action)}>
            {node.value}
          </button>
        );
      case "edit":
        return node.multi ? (
          <textarea key={index} className="npc-edit" rows={3} value={edits[node.var] ?? ""} onChange={(event) => setEdit(node.var, event.target.value)} />
        ) : (
          <input key={index} className="npc-edit" value={edits[node.var] ?? ""} onChange={(event) => setEdit(node.var, event.target.value)} />
        );
      case "img":
        return null;
    }
  });
}

export function NpcWindow({ npc, send }: { npc: Npc; send: (command: Command) => void }) {
  const t = useT();
  const [edits, setEdits] = useState<Edits>({});
  const [manual, setManual] = useState("");
  const close = () => {
    setEdits({});
    send({ type: "closeNpc" });
  };
  const run = (action: string | null) => {
    if (action) send({ type: "npcAction", action: substitute(action, edits) });
  };
  const sendManual = () => {
    const value = manual.trim();
    if (!value) return;
    send({ type: "bypass", command: value.split("%objectId%").join(String(npc.objectId)) });
    setManual("");
  };
  return (
    <WindowFrame
      title={npc.name || "NPC"}
      width={440}
      onClose={close}
      footer={
        <>
          <form
            className="npc-manual"
            onSubmit={(event) => {
              event.preventDefault();
              sendManual();
            }}
          >
            <input className="say" value={manual} placeholder={t("bypass manual (ex.: npc_%objectId%_Chat 1)")} onChange={(event) => setManual(event.target.value)} />
            <button type="submit" className="btn" disabled={!manual.trim()}>{t("Enviar")}</button>
          </form>
          <button type="button" className="btn" onClick={close}>{t("Fechar")}</button>
        </>
      }
    >
      {npc.title && <div className="npc-title">{npc.title}</div>}
      <div className="npc-body">{render(npc.nodes, edits, (name, value) => setEdits((current) => ({ ...current, [name]: value })), run)}</div>
    </WindowFrame>
  );
}
