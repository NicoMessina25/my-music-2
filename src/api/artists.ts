import { invoke } from "@tauri-apps/api/core";

export type Artist = { id: number; name: string; archivedAt: string | null };
export type ArtistInput = { name: string };
export type ArtistView = Artist & { songsCount: number };
export type ArtistRef = { id: number; name: string };

export const listArtists = (includeArchived = false) =>
  invoke<ArtistView[]>("list_artists", { includeArchived });
export const updateArtist = (id: number, input: ArtistInput) =>
  invoke<Artist>("update_artist", { id, input });
