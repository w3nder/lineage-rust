function base(): string {
  return typeof navigator !== "undefined" && navigator.userAgent.includes("Windows") ? "http://gameicon.localhost" : "gameicon://localhost";
}

export const icons = {
  item: (itemId: number) => `${base()}/item/${itemId}`,
  skill: (skillId: number, level = 1) => `${base()}/skill/${skillId}/${level}`,
  texture: (name: string) => `${base()}/texture/${name}`,
};
