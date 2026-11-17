import { NavLink, Outlet } from "react-router";

const links = [
  { to: "/songs", label: "Canciones" },
  { to: "/artists", label: "Artistas" },
  // { to: "/playlists", label: "Playlists" },
  // { to: "/imports", label: "Importaciones" },
];

export default function Layout() {
  return (
    <div className="flex">
      <nav className="flex p-2 border-r-2 border-emerald-300 flex-col">
        <h1 className="font-bold">MyMusic</h1>
        {links.map((l) => (
          <NavLink key={l.to} to={l.to} className={({ isActive }) => (isActive ? "bg-emerald-300" : "")}>
            {l.label}
          </NavLink>
        ))}
      </nav>
      <main className="content">
        <Outlet />
      </main>
    </div>
  );
}