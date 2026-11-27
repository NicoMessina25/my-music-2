import { modals } from "@mantine/modals";
import { useEffect } from "react";
import { useBlocker } from "react-router";

export default function useConfirmLeave(shouldBlock: () => boolean) {
  const blocker = useBlocker(() => shouldBlock());

  useEffect(() => {
    if (blocker.state === "blocked")
      modals.openConfirmModal({
        title: "Estás por salir sin guardar",
        labels: { confirm: "Confirmar", cancel: "Cancelar" },
        onConfirm: () => blocker.proceed(),
        onCancel: () => blocker.reset(),
        confirmProps: { color: "red" },
        closeOnEscape: false,
        closeOnClickOutside: false,
        withCloseButton: false,
      });
  }, [blocker]);
}
