import type { MenuItem } from "./ContextMenu";
import type { Command, Nearby } from "./model";

export function nearbyMenu(found: Nearby, t: (pt: string) => string, send: (command: Command) => void, onInfo: () => void): MenuItem[] {
  const items: MenuItem[] = [
    { label: t("Selecionar"), run: () => send({ type: "selectTarget", id: found.id }) },
    { label: t("Info do alvo"), run: () => { send({ type: "selectTarget", id: found.id }); onInfo(); } },
  ];
  if (found.kind === "player") {
    items.push(
      { label: t("Convidar para o grupo"), run: () => send({ type: "partyInvite", name: found.name }) },
      { label: t("Trocar"), run: () => send({ type: "tradeRequest", id: found.id }) },
      { label: t("Convidar para o clã"), run: () => send({ type: "clanInvite", id: found.id }) },
      { label: t("Adicionar aos amigos"), run: () => send({ type: "friendInvite", name: found.name }) },
    );
  } else if (found.kind === "item") {
    items.push({ label: t("Ignorar este item"), run: () => send({ type: "ignoreItem", itemId: found.itemId }) });
  } else {
    if (found.kind === "npc") items.push({ label: t("Ir até e falar"), run: () => send({ type: "talk", id: found.id }) });
    items.push(
      { label: t("Pôr na lista negra"), run: () => send({ type: "blacklist", id: found.id }) },
      { label: t("Ignorar este NPC"), run: () => send({ type: "ignoreNpc", npcId: found.npcId }) },
    );
  }
  return items;
}
