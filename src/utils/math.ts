export function intDiv(
  div: number,
  divider: number,
): { quotient: number; remainder: number } {
  return {
    quotient: Math.trunc(div / divider),
    remainder: div % divider,
  };
}
