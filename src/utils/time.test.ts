import { describe, expect, it } from "vitest";
import { string2time, time2string } from "./time";

describe("time2string", () => {
  it.each([
    [5, "0:00.005"],
    [500, "0:00.500"],
    [1_000, "0:01"],
    [45_000, "0:45"],
    [59_999, "0:59.999"],
    [60_000, "1:00"],
    [65_000, "1:05"],
    [225_000, "3:45"],
    [225_005, "3:45.005"],
    [225_500, "3:45.500"],
    [600_000, "10:00"],
    [3_599_000, "59:59"],
    [3_600_000, "1:00:00"],
    [3_630_000, "1:00:30"],
    [3_725_123, "1:02:05.123"],
    [86_400_000, "24:00:00"],
  ])("%i ms -> %s", (ms, expected) => {
    expect(time2string(ms)).toBe(expected);
  });

  it.each([null, undefined, 0])("%s -> null", (value) => {
    expect(time2string(value)).toBeNull();
  });
});

describe("string2time", () => {
  it.each([
    ["0:45", 45_000],
    ["3:45", 225_000],
    ["03:45", 225_000],
    ["10:00", 600_000],
    ["1:00:00", 3_600_000],
    ["1:02:05", 3_725_000],
    ["3:45.5", 225_500],
    ["3:45.05", 225_050],
    ["3:45.005", 225_005],
    ["1:02:05.123", 3_725_123],
  ])("%s -> %i ms", (text, expected) => {
    expect(string2time(text)).toBe(expected);
  });

  it.each([null, ""])("%j -> null", (value) => {
    expect(string2time(value)).toBeNull();
  });

  it("devuelve siempre un entero (sin errores de punto flotante)", () => {
    for (const text of ["0:01.001", "0:00.007", "59:59.999", "1:02:05.123"]) {
      expect(Number.isInteger(string2time(text))).toBe(true);
    }
  });
});

describe("ida y vuelta", () => {
  // Lo que muestra el formulario tiene que volver al mismo valor al guardar,
  // aunque el usuario no toque el campo.
  it.each([
    5, 500, 45_000, 59_999, 60_000, 225_000, 225_005, 225_500, 3_599_000,
    3_600_000, 3_630_000, 3_725_123, 86_400_000,
  ])("string2time(time2string(%i)) devuelve el mismo valor", (ms) => {
    expect(string2time(time2string(ms))).toBe(ms);
  });

  // El texto que genera time2string tiene que pasar la validación del form;
  // si no, una canción no se podría guardar sin retocar la duración.
  const DURATION_REGEX = /^(?:\d+:)?\d{1,2}:[0-5]\d(?:\.\d{1,3})?$/;

  it.each([5, 45_000, 225_005, 3_600_000, 3_630_000, 3_725_123])(
    "time2string(%i) pasa la validación del formulario",
    (ms) => {
      expect(time2string(ms)).toMatch(DURATION_REGEX);
    },
  );
});
