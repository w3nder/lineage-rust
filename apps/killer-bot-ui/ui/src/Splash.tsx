import logo from "./assets/logo.png";

export const SPLASH_MIN_MS = 1600;

export function Splash({ leaving }: { leaving: boolean }) {
  return (
    <div className={`splash${leaving ? " leaving" : ""}`} aria-hidden>
      <img src={logo} alt="" />
      <div className="splash-bar"><div /></div>
    </div>
  );
}
