import { intDiv } from "./math";

export function string2time(str: string | null) {
  if (!str) return null;

  const milliSeconds = str.includes(".")
    ? Number(str.split(".")[1].padEnd(3, "0"))
    : 0;
  const time_split = str.split(".")[0].split(":").map(Number);

  let time = milliSeconds;

  for (let i = 0; i < time_split.length; i++) {
    time += time_split[i] * 1000 * 60 ** (time_split.length - 1 - i);
  }

  return time;
}

export function time2string(time: number | null | undefined) {
  if (!time) return null;

  const { quotient: timeSeconds, remainder: milliSeconds } = intDiv(time, 1000);
  const { quotient: timeMinutes, remainder: seconds } = intDiv(timeSeconds, 60);
  const { quotient: hours, remainder: minutes } = intDiv(timeMinutes, 60);

  let result = "";

  if (hours) result = hours.toString();
  if (result.length)
    result = result + ":" + minutes.toString().padStart(2, "0");
  else result = minutes.toString();

  result += ":" + seconds.toString().padStart(2, "0");

  if (milliSeconds) result += "." + milliSeconds.toString().padStart(3, "0");

  return result;
}
