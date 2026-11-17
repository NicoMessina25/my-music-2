import { invoke } from "@tauri-apps/api/core";

export type Artist = { id: number; name: string; archivedAt: string | null };
export type ArtistInput = { name: string };

export const listArtists = (includeArchived = false) =>
  invoke<Artist[]>("list_artists", { includeArchived });
export const createArtist = (input: ArtistInput) =>
  invoke<Artist>("create_artist", { input });
export const updateArtist = (id: number, input: ArtistInput) =>
  invoke<Artist>("update_artist", { id, input });
export const archiveArtist = (id: number) =>
  invoke<void>("archive_artist", { id });