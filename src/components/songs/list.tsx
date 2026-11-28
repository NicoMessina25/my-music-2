import { ActionIcon, Button, Switch } from "@mantine/core";
import { DataTable } from "mantine-datatable";
import { archiveSong, restoreSong, SongView } from "../../api/song";
import { showNotification } from "@mantine/notifications";
import { useNavigate } from "react-router";
import useSongs from "./useSongs";
import { ArchiveIcon, ArrowUDownLeftIcon } from "@phosphor-icons/react";
import { modals } from "@mantine/modals";
import { useState } from "react";
import { errorMessage } from "../../api/error";

export default function SongsList() {
  const navigate = useNavigate();
  const [includeArchived, setIncludeArchived] = useState(false);
  const { songs, refresh, loading } = useSongs(includeArchived);

  const confirmArchive = ({ id, title, artists }: SongView) =>
    modals.openConfirmModal({
      title: "Archivar canción",
      children: (
        <p>
          {title} de {artists.map((a) => a.name).join(", ")}
        </p>
      ),
      labels: { confirm: "Archivar", cancel: "Cancelar" },
      onConfirm: () =>
        archiveSong(id)
          .then(() => {
            showNotification({
              message: "Archivada correctamente",
              color: "green",
            });
            refresh();
          })
          .catch((err) =>
            showNotification({ message: errorMessage(err), color: "red" }),
          ),
      confirmProps: { color: "red" },
    });

  const confirmRestore = ({ id, title, artists }: SongView) =>
    modals.openConfirmModal({
      title: "Restaurar canción",
      children: (
        <p>
          {title} de {artists.map((a) => a.name).join(", ")}
        </p>
      ),
      labels: { confirm: "Restaurar", cancel: "Cancelar" },
      onConfirm: () =>
        restoreSong(id)
          .then(() => {
            showNotification({
              message: "Restaurada correctamente",
              color: "green",
            });
            refresh();
          })
          .catch((err) =>
            showNotification({ message: errorMessage(err), color: "red" }),
          ),
    });

  return (
    <div className="pageContainer flex-col">
      <div className="flex justify-between">
        <h1>Registro de canciones</h1>
        <Switch
          checked={includeArchived}
          onChange={(e) => setIncludeArchived(e.currentTarget.checked)}
          label="Incluir archivadas"
        />
        <Button onClick={() => navigate("../new")}>Crear</Button>
      </div>
      <DataTable
        withTableBorder
        borderRadius="sm"
        withColumnBorders
        striped
        highlightOnHover
        fetching={loading}
        rowColor={(record) => (record.archivedAt ? "gray.6" : undefined)}
        records={songs}
        columns={[
          {
            accessor: "id",
            title: "#",
            textAlign: "right",
          },
          { accessor: "title", title: "Título" },
          {
            accessor: "artists",
            title: "Artistas",
            render: (record) => record.artists.map((a) => a.name).join(", "),
          },
          {
            accessor: "actions",
            render: (record) => {
              if (record.archivedAt)
                return (
                  <ActionIcon
                    aria-label="Restaurar canción"
                    onClick={(e) => {
                      e.stopPropagation();
                      confirmRestore(record);
                    }}
                  >
                    <ArrowUDownLeftIcon size={20} />
                  </ActionIcon>
                );

              return (
                <ActionIcon
                  aria-label="Archivar canción"
                  color="red"
                  onClick={(e) => {
                    e.stopPropagation();
                    confirmArchive(record);
                  }}
                >
                  <ArchiveIcon size={20} />
                </ActionIcon>
              );
            },
          },
        ]}
        emptyState="No hay canciones"
        onRowClick={(e) =>
          e.record.archivedAt
            ? showNotification({
                message: "Restaurar canción para editarla",
                color: "yellow",
              })
            : navigate(`../${e.record.id}/edit`)
        }
      />
    </div>
  );
}
