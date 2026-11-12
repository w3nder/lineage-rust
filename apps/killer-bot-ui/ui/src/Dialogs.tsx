import logo from "./assets/logo.png";
import { grouped } from "./format";
import { useT, type Lang } from "./i18n";
import type { Frame, TargetInfo } from "./model";
import { icons } from "./icons";
import { ItemName } from "./ItemName";
import { WindowFrame } from "./Windows";

export function About({ onClose, version }: { onClose: () => void; version: string }) {
  const t = useT();
  return (
    <WindowFrame title={t("Sobre")} onClose={onClose} width={420}>
      <img className="about-logo" src={logo} alt="L2UnkBot" />
      <div className="who">
        <b>L2UnkBot</b>
        <div>
          {t("versão")} <span className="num">{version}</span>
        </div>
      </div>
      <p className="hint">{t("Cliente de automação do servidor l2killer, escrito em Rust sobre o cliente Killer.")}</p>
      <p className="hint">{t("Fala o protocolo do servidor; não injeta nem lê memória do jogo.")}</p>
    </WindowFrame>
  );
}

export function Setup({ frame, lang, theme, onLang, onTheme, onCheck, onClose }: { frame: Frame; lang: Lang; theme: "dark" | "light"; onLang: (lang: Lang) => void; onTheme: (theme: "dark" | "light") => void; onCheck: () => void; onClose: () => void }) {
  const t = useT();
  return (
    <WindowFrame title={t("Configuração")} onClose={onClose} width={440}>
      <div className="setting">
        <span>{t("Idioma")}</span>
        <div className="subtabs">
          {(["pt", "en"] as const).map((value) => (
            <button key={value} type="button" className={`ptab${lang === value ? " on" : ""}`} onClick={() => onLang(value)}>
              {value === "pt" ? "Português" : "English"}
            </button>
          ))}
        </div>
      </div>
      <div className="setting">
        <span>{t("Tema")}</span>
        <div className="subtabs">
          {(["dark", "light"] as const).map((value) => (
            <button key={value} type="button" className={`ptab${theme === value ? " on" : ""}`} onClick={() => onTheme(value)}>
              {value === "dark" ? t("Escuro") : t("Claro")}
            </button>
          ))}
        </div>
      </div>
      <div className="setting">
        <span>
          {t("Dados do jogo")}
          <small className="dim">{frame.dataStatus || t("Em dia")}</small>
        </span>
        <button type="button" className="btn" onClick={onCheck}>{t("Verificar")}</button>
      </div>
    </WindowFrame>
  );
}

function Bar({ label, gauge, color }: { label: string; gauge: { current: number; max: number }; color: string }) {
  const pct = gauge.max > 0 ? Math.max(0, Math.min(100, (gauge.current / gauge.max) * 100)) : 0;
  return (
    <div className="ti-bar">
      <span className="ti-bar-label">{label}</span>
      <div className="ti-bar-track"><div style={{ width: `${pct}%`, background: color }} /></div>
      <span className="num ti-bar-text">{grouped(gauge.current)} / {grouped(gauge.max)}</span>
    </div>
  );
}

export function TargetInfoWindow({ info, onClose }: { info: TargetInfo | null; onClose: () => void }) {
  const t = useT();
  const yes = (value: boolean) => (value ? t("Sim") : t("Não"));
  const rows: [string, string][] = info
    ? [
        ["Object ID", String(info.objectId)],
        ...(info.npcId === null ? [] : ([["NPC ID", String(info.npcId)], ["NPC", info.npcName ?? "—"], [t("Atacável"), yes(info.attackable === true)]] as [string, string][])),
        ...(info.itemCount === null ? [] : ([[t("Quantidade"), grouped(info.itemCount)]] as [string, string][])),
        [t("Distância"), grouped(info.distance)],
        [t("Posição"), info.position.map(grouped).join(", ")],
        ...(info.kind === "item" ? [] : ([
          [t("Morto"), yes(info.dead)],
          [t("Em combate"), yes(info.inCombat)],
          [t("Estado"), info.sitting ? t("Sentado") : info.running ? t("Correndo") : t("Andando")],
          [t("Lançando"), info.casting ?? "—"],
          [t("Alvo dele"), info.targetName ?? "—"],
          [t("Vel. corrida"), grouped(info.runSpeed)],
          ["P. Atk. Spd", grouped(info.pAtkSpeed)],
          ["M. Atk. Spd", grouped(info.mAtkSpeed)],
        ] as [string, string][])),
        ...(info.kind === "player" ? ([[t("Recomendações"), grouped(info.recommendations)], [t("Loja"), yes(info.store)], [t("Herói"), yes(info.hero)]] as [string, string][]) : []),
        [t("Clã"), info.clan ?? (info.clanId ? `#${info.clanId}` : "—")],
        [t("Aliança"), info.ally ?? "—"],
        [t("PvP / Karma"), `${info.pvpFlag} / ${info.karma}`],
        [t("Efeitos"), info.abnormalMask],
      ]
    : [];
  return (
    <WindowFrame title={t("Info do alvo")} onClose={onClose} width={440}>
      {!info && <div className="empty">{t("Nenhum alvo")}</div>}
      {info && (
        <>
          <div className="ti-head">
            <b className="ti-name" style={info.kind === "player" ? { color: info.nameColor } : undefined}>{info.name || info.npcName || "—"}</b>
            {info.title && <span className="ti-title">{info.title}</span>}
            <span className="ti-sub">
              {info.kind !== "item" && info.level > 0 && `Lv ${info.level}`}
              {info.className ? `${info.level > 0 ? " · " : ""}${info.className}` : ""}
              {info.hero ? " · Hero" : ""}
            </span>
          </div>
          {info.kind !== "item" && (
            <div className="ti-bars">
              {info.cp && info.cp.max > 0 && <Bar label="CP" gauge={info.cp} color="var(--cp)" />}
              {info.hp && <Bar label="HP" gauge={info.hp} color="var(--hp)" />}
              {info.mp && info.mp.max > 0 && <Bar label="MP" gauge={info.mp} color="var(--mp)" />}
            </div>
          )}
          {info.equipment.some((piece) => piece.itemId > 0) && (
            <div className="ti-equip">
              {info.equipment.map((piece) => (
                <div key={`${piece.slot}-${piece.itemId}`} className={`ti-piece${piece.itemId > 0 ? "" : " empty-slot"}`}>
                  {piece.itemId > 0 ? <img className="icon" src={icons.item(piece.itemId)} alt="" loading="lazy" /> : <span className="icon blank" />}
                  <span className="ti-slot">{piece.slot}</span>
                  <span className="cut">{piece.itemId > 0 ? <ItemName item={piece} /> : "—"}</span>
                </div>
              ))}
            </div>
          )}
          {rows.map(([label, value]) => (
            <div key={label} className="stat">
              <span>{label}</span>
              <span className="num">{value}</span>
            </div>
          ))}
        </>
      )}
    </WindowFrame>
  );
}
