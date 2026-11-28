import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { Button, LoadingOverlay } from "@mantine/core";
import { emptySong, getSong, SongInput, view2input } from "../../api/song";
import { showNotification } from "@mantine/notifications";
import SongForm from "./form";
import { errorMessage } from "../../api/error";

export default function SongCRUD() {
  const { id } = useParams<{ id: string }>();
  const isEditing = !!id;
  const [loading, setLoading] = useState(isEditing);
  const [song, setSong] = useState<SongInput>(emptySong);
  const [error, setError] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    if (id)
      getSong(Number(id))
        .then((view) => setSong(view2input(view)))
        .catch((err) => {
          showNotification({ message: errorMessage(err) });
          setError(true);
        })
        .finally(() => setLoading(false));
  }, [id]);

  return (
    <div className="pageContainer flex-col">
      <LoadingOverlay visible={loading && isEditing} />
      <Button variant="outline" onClick={() => navigate(-1)}>
        Volver
      </Button>
      <h1>{isEditing ? song.title : "Nueva canción"}</h1>
      {!loading && !error && (
        <SongForm
          id={id ? Number(id) : undefined}
          initialValues={song}
          onSuccess={() => navigate(-1)}
        />
      )}
      {!loading && error && <p>No se encontró la canción</p>}
    </div>
  );
}
