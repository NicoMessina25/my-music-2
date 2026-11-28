import { useForm } from "@mantine/form";
import { Button, TextInput } from "@mantine/core";
import { showNotification } from "@mantine/notifications";
import { useState } from "react";
import { FormProps } from "../../utils/form";
import { errorMessage, isAppError } from "../../api/error";
import { Artist, ArtistInput, updateArtist } from "../../api/artists";

export default function ArtistForm({
  id,
  initialValues,
  onSuccess,
}: Readonly<FormProps<ArtistInput, Artist>>) {
  const [loading, setLoading] = useState(false);

  const form = useForm({
    mode: "uncontrolled",
    initialValues,

    validate: {
      name: (value) => (value.trim().length > 0 ? null : "Obligatorio"),
    },
  });

  function createOrUpdate(input: ArtistInput) {
    if (!id) return;

    setLoading(true);
    updateArtist(id, input)
      .then((res) => {
        showNotification({ message: "Guardado correctamente", color: "green" });
        onSuccess?.(res);
      })
      .catch((err) => {
        if (isAppError(err) && err.kind === "conflict")
          form.setFieldError("name", err.message);
        else showNotification({ message: errorMessage(err), color: "red" });
      })
      .finally(() => setLoading(false));
  }

  return (
    <form onSubmit={form.onSubmit(createOrUpdate)}>
      <TextInput
        withAsterisk
        label="Nombre"
        key={form.key("name")}
        {...form.getInputProps("name")}
      />
      <Button type="submit" loading={loading}>
        Guardar
      </Button>
    </form>
  );
}
