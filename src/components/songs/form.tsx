import { useForm } from "@mantine/form";
import { createSong, SongInput, SongView, updateSong } from "../../api/song";
import { Button, TagsInput, TextInput } from "@mantine/core";
import { string2time, time2string } from "../../utils/time";
import { showNotification } from "@mantine/notifications";
import { useState } from "react";
import { FormProps } from "../../utils/form";
import useArtists from "../artists/useArtists";
import useConfirmLeave from "../hooks/useConfirmLeave";
import { errorMessage, isAppError } from "../../api/error";

export default function SongForm({
  id,
  initialValues,
  onSuccess,
}: Readonly<FormProps<SongInput, SongView>>) {
  const [loading, setLoading] = useState(false);

  const form = useForm({
    mode: "uncontrolled",
    initialValues: {
      ...initialValues,
      durationMs: time2string(initialValues.durationMs),
    },

    validate: {
      title: (value) => (value?.trim().length > 0 ? null : "Obligatorio"),
      artists: (value) => (value?.length > 0 ? null : "Al menos un artista"),
      durationMs: (value: string | null) =>
        !value || /^(?:\d+:)?\d{1,2}:[0-5]\d(?:\.\d{1,3})?$/.test(value)
          ? null
          : "Formato inválido",
      releaseDate: (value: string | null) =>
        !value ||
        /^\d{4}(?:-(?:0[1-9]|1[0-2])(?:-(?:0[1-9]|[12]\d|3[01]))?)?$/.test(
          value,
        )
          ? null
          : "Formato inválido",
    },
    transformValues: (values) => ({
      ...values,
      durationMs: string2time(values.durationMs),
    }),
  });

  useConfirmLeave(() => form.isDirty());

  const { artists, loading: artistsLoading } = useArtists();

  function createOrUpdate(input: SongInput) {
    setLoading(true);
    (id ? updateSong(id, input) : createSong(input))
      .then((res) => {
        showNotification({ message: "Guardado correctamente", color: "green" });
        form.resetDirty();
        onSuccess?.(res);
      })
      .catch((err) => {
        showNotification({ message: errorMessage(err), color: "red" });

        if (isAppError(err) && err.kind === "conflict")
          form.setFieldError("title", err.message);
      })
      .finally(() => setLoading(false));
  }

  return (
    <form onSubmit={form.onSubmit(createOrUpdate)}>
      <TextInput
        withAsterisk
        label="Título"
        key={form.key("title")}
        {...form.getInputProps("title")}
      />
      <TextInput
        label="Album"
        key={form.key("album")}
        {...form.getInputProps("album")}
      />
      <TagsInput
        label="Artistas ('Enter' para agregar)"
        key={form.key("artists")}
        {...form.getInputProps("artists")}
        data={artists.map((a) => a.name)}
        loading={artistsLoading}
        withPillsReorder
      />
      <TextInput
        label="Duración (mm:ss)"
        key={form.key("durationMs")}
        {...form.getInputProps("durationMs")}
      />
      <TextInput
        label="Lanzamiento"
        description="YYYY / YYYY-MM / YYYY-MM-DD"
        key={form.key("releaseDate")}
        {...form.getInputProps("releaseDate")}
      />
      <TextInput
        label="Notas"
        key={form.key("notes")}
        {...form.getInputProps("notes")}
      />
      <Button type="submit" loading={loading}>
        Guardar
      </Button>
    </form>
  );
}
