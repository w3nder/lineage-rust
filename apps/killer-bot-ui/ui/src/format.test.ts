import { describe, expect, it } from "vitest";
import { clockTime, compact, fraction, grouped, percent, signed } from "./format";

describe("format", () => {
  it("groups thousands with a space and keeps the sign", () => {
    expect(grouped(1234567)).toBe("1\u202f234\u202f567");
    expect(grouped(-1000)).toBe("-1\u202f000");
    expect(grouped(999)).toBe("999");
  });

  it("formats percent with a comma and two decimals", () => {
    expect(percent(12.345)).toBe("12,35%");
  });

  it("compacts large numbers", () => {
    expect(compact(1_500_000)).toBe("1,5 M");
    expect(compact(48_000)).toBe("48 k");
    expect(compact(12)).toBe("12");
  });

  it("clamps a fraction between zero and one and survives a zero max", () => {
    expect(fraction(5, 10)).toBe(0.5);
    expect(fraction(50, 10)).toBe(1);
    expect(fraction(-5, 10)).toBe(0);
    expect(fraction(5, 0)).toBe(0);
  });

  it("shows no clock for an unstamped chat line and a time for a stamped one", () => {
    expect(clockTime(0)).toBe("");
    expect(clockTime(1_761_145_200)).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });

  it("signs the vertical difference", () => {
    expect(signed(12.4)).toBe("+12");
    expect(signed(-7.6)).toBe("-8");
    expect(signed(0)).toBe("0");
  });
});
