import { useEffect, useState } from "react";
import { ArtistView, listArtists } from "../../api/artists";
import { showNotification } from "@mantine/notifications";
import { errorMessage } from "../../api/error";

const keyToString = (count: number, includeArchived: boolean) =>
  `${includeArchived}-${count}`;

export default function useArtists(includeArchived: boolean = false) {
  const [request, setRequest] = useState<{
    key: string | null;
    artists: ArtistView[];
  }>({ key: null, artists: [] });
  const [count, setCount] = useState(0);

  const loading = request.key !== keyToString(count, includeArchived);

  const refresh = () => {
    setCount((c) => c + 1);
  };

  useEffect(() => {
    let ignore = false;

    listArtists(includeArchived)
      .then((res) => {
        if (ignore) return;

        setRequest({
          key: keyToString(count, includeArchived),
          artists: res,
        });
      })
      .catch((err) => {
        if (ignore) return;

        showNotification({ message: errorMessage(err), color: "red" });
        setRequest({
          artists: [],
          key: keyToString(count, includeArchived),
        });
      });

    return () => {
      ignore = true;
    };
  }, [includeArchived, count]);

  return { artists: request.artists, loading, refresh };
}
