import ReactDOM from "react-dom/client";
import { createHashRouter, Navigate, RouterProvider } from "react-router";
import Layout from "./components/layout/Layout";
import SongsPage from "./components/songs/SongsPage";
import ArtistsPage from "./components/artists/ArtistsPage";
import "./main.css"

const router = createHashRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      { index: true, element: <Navigate to="/songs" replace /> },
      { path: "songs", element: <SongsPage /> },
      { path: "artists", element: <ArtistsPage /> },
      // { path: "artists/:id", element: <ArtistDetailPage /> },
      // { path: "playlists", element: <PlaylistsPage /> },
      // { path: "imports", element: <ImportsPage /> },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root")!).render(
 
  <RouterProvider router={router} />

);