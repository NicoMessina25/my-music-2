import { invoke } from "@tauri-apps/api/core";
import { ArtistRef } from "./artists";

export type SongInput = {
  title: string;
  artists: string[];
  album: string | null;
  releaseDate: string | null;
  durationMs: number | null;
  notes: string | null;
};

export type SongView = {
  id: number;
  title: string;
  artists: ArtistRef[];
  album: string | null;
  releaseDate: string | null;
  durationMs: number | null;
  notes: string | null;
  createdAt: string;
  archivedAt: string | null;
};

export const listSongs = (includeArchived = false) =>
  invoke<SongView[]>("list_songs", { includeArchived });
export const createSong = (input: SongInput) =>
  invoke<SongView>("create_song", { input });
export const getSong = (id: number) => invoke<SongView>("get_song", { id });
export const updateSong = (id: number, input: SongInput) =>
  invoke<SongView>("update_song", { id, input });
export const archiveSong = (id: number) => invoke<void>("archive_song", { id });
export const restoreSong = (id: number) => invoke<void>("restore_song", { id });

export function view2input({
  title,
  album,
  artists,
  notes,
  releaseDate,
  durationMs,
}: SongView): SongInput {
  return {
    title,
    album,
    artists: artists.map((a) => a.name),
    durationMs,
    notes,
    releaseDate,
  };
}

export const emptySong: SongInput = {
  title: "",
  artists: [],
  album: null,
  durationMs: null,
  notes: null,
  releaseDate: null,
};
