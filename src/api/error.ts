const KINDS = ["db", "validation", "conflict", "notFound", "lock"] as const;

type AppErrorKind = (typeof KINDS)[number];

type AppError = { kind: AppErrorKind; message: string };

export function isAppError(e: unknown): e is AppError {
  return (
    typeof e === "object" &&
    e !== null &&
    "kind" in e &&
    "message" in e &&
    typeof e.message === "string" &&
    (KINDS as readonly unknown[]).includes(e.kind)
  );
}

export function errorMessage(e: unknown): string {
  if (isAppError(e)) return e.message;
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;

  return "Error desconocido";
}
