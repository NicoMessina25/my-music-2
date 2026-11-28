import ReactDOM from "react-dom/client";
import { createHashRouter, Navigate, RouterProvider } from "react-router";
import Layout from "./components/layout";
import SongsLayout from "./components/songs/layout";
import ArtistsList from "./components/artists/list";
import "./main.css";
import { MantineProvider } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import SongsList from "./components/songs/list";
import SongCRUD from "./components/songs/crud";

const router = createHashRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      { index: true, element: <Navigate to="/songs" replace /> },
      {
        path: "songs",
        element: <SongsLayout />,
        children: [
          { index: true, element: <Navigate to="/songs/list" replace /> },
          { path: "list", element: <SongsList /> },
          { path: "new", element: <SongCRUD /> },
          { path: ":id/edit", element: <SongCRUD /> },
        ],
      },
      { path: "artists", element: <ArtistsList /> },
      // { path: "artists/:id", element: <ArtistDetailPage /> },
      // { path: "playlists", element: <PlaylistsPage /> },
      // { path: "imports", element: <ImportsPage /> },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <MantineProvider>
    <Notifications />
    <RouterProvider router={router} />
  </MantineProvider>,
);
