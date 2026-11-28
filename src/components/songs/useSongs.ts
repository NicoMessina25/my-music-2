import { useEffect, useState } from "react";
import { listSongs, SongView } from "../../api/song";
import { showNotification } from "@mantine/notifications";
import { errorMessage } from "../../api/error";

const keyToString = (count: number, includeArchived: boolean) =>
  `${includeArchived}-${count}`;

export default function useSongs(includeArchived = false) {
  const [request, setRequest] = useState<{
    key: string | null;
    songs: SongView[];
  }>({ key: null, songs: [] });
  const [count, setCount] = useState(0);

  const loading = request.key !== keyToString(count, includeArchived);

  const refresh = () => {
    setCount((c) => c + 1);
  };

  useEffect(() => {
    let ignore = false;

    listSongs(includeArchived)
      .then((res) => {
        if (ignore) return;

        setRequest({
          key: keyToString(count, includeArchived),
          songs: res,
        });
      })
      .catch((err) => {
        if (ignore) return;

        showNotification({
          message: errorMessage(err),
          color: "red",
        });
        setRequest({
          songs: [],
          key: keyToString(count, includeArchived),
        });
      });

    return () => {
      ignore = true;
    };
  }, [includeArchived, count]);

  return { songs: request.songs, loading, refresh };
}
