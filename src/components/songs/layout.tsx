import { useState } from "react";
import { Outlet } from "react-router";

export default function SongsLayout() {
  const [state, setState] = useState();

  return <Outlet context={[state, setState]} />;
}
