type Named = { name: string; addName?: string; grade?: string | null; enchant?: number; known?: boolean };

export function ItemName({ item }: { item: Named }) {
  return (
    <span className={`item-name${item.known === false ? " unknown" : ""}`} title={item.known === false ? "Sem nome na tabela do cliente" : undefined}>
      {item.enchant && item.enchant > 0 ? <b className="enchant">+{item.enchant} </b> : null}
      <span>{item.name}</span>
      {item.addName ? <span className="add-name"> {item.addName}</span> : null}
      {item.grade ? <span className="grade">{item.grade}</span> : null}
    </span>
  );
}
