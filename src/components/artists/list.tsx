import { DataTable } from "mantine-datatable";
import useArtists from "./useArtists";
import { Switch } from "@mantine/core";
import { useState } from "react";
import { ArtistView } from "../../api/artists";
import { modals } from "@mantine/modals";
import ArtistForm from "./form";

const MODAL_ID = "rename-artist";

export default function ArtistsList() {
  const [includeArchived, setIncludeArchived] = useState(false);
  const { artists, loading, refresh } = useArtists(includeArchived);

  const updateName = (artist: ArtistView) =>
    modals.open({
      modalId: MODAL_ID,
      title: "Renombrar artista",
      children: (
        <ArtistForm
          id={artist.id}
          initialValues={{ name: artist.name }}
          onSuccess={() => {
            refresh();
            modals.close(MODAL_ID);
          }}
        />
      ),
    });

  return (
    <div className="pageContainer flex-col">
      <div className="flex justify-between">
        <h1>Registro de artistas</h1>
        <Switch
          checked={includeArchived}
          onChange={(e) => setIncludeArchived(e.currentTarget.checked)}
          label="Incluir sin canciones"
        />
      </div>
      <DataTable
        withTableBorder
        borderRadius="sm"
        withColumnBorders
        striped
        highlightOnHover
        fetching={loading}
        records={artists}
        columns={[
          {
            accessor: "id",
            title: "#",
            textAlign: "right",
          },
          { accessor: "name", title: "Nombre" },
          { accessor: "songsCount", title: "Cant. de canciones" },
        ]}
        emptyState="No hay artistas"
        onRowClick={(e) => updateName(e.record)}
      />
    </div>
  );
}
