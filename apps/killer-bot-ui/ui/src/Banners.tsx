import { useT } from "./i18n";
import type { Command, World } from "./model";

export function Banners({ world, send }: { world: World; send: (command: Command) => void }) {
  const t = useT();
  const death = world.death;
  const question = world.questions[0];
  if (!death && !question) return null;
  const spots: [string, boolean, number][] = death
    ? [[t("Cidade"), death.town, 0], ["Clan Hall", death.hideout, 1], [t("Castelo"), death.castle, 2], ["Siege HQ", death.siegeHq, 3], [t("Fortaleza"), death.fortress, 4]]
    : [];
  return (
    <>
      {death && (
        <div className="banner" role="alertdialog" aria-label={t("Você morreu")}>
          <b style={{ color: "#f0a39e" }}>{t("Você morreu")}</b>
          <div className="row-buttons">
            {spots.map(([label, enabled, kind]) => (
              <button key={kind} type="button" className={`btn${kind === 0 ? " primary" : ""}`} disabled={!enabled} onClick={() => send({ type: "restartPoint", kind })}>
                {label}
              </button>
            ))}
          </div>
        </div>
      )}
      {question && (
        <div className="banner ask" style={death ? { top: 150 } : undefined} role="alertdialog" aria-label={t("Convite")}>
          <b>{question.text}</b>
          <div className="row-buttons">
            <button type="button" className="btn primary" onClick={() => send({ type: "answer", index: question.index, accept: true })}>{t("Aceitar")}</button>
            <button type="button" className="btn" onClick={() => send({ type: "answer", index: question.index, accept: false })}>{t("Recusar")}</button>
          </div>
        </div>
      )}
    </>
  );
}
