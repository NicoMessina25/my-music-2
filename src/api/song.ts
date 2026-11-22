import { invoke } from "@tauri-apps/api/core";
import { ArtistRef } from "./artists";

export type SongInput = { 
    title: string,
    artists: string[],
    album: string | null,
    releaseDate: string | null,
    durationMs: number | null,
    notes: string | null,
};

export type SongView = { 
    id: number,
    title: string,
    artists: ArtistRef[],
    album: string | null,
    releaseDate: string | null,
    durationMs: number | null,
    notes: string | null,
    createdAt: string | null,
    archivedAt: string | null
}

export const listSongs = (includeArchived = false) =>
  invoke<SongView[]>("list_songs", { includeArchived });
export const createSong = (input: SongInput) =>
  invoke<SongView>("create_song", { input });
export const getSong = (id: number) =>
  invoke<void>("get_song", { id });
export const updateSong = (id: number, input: SongInput) =>
  invoke<SongView>("update_song", { id, input });
export const archiveSong = (id: number) =>
  invoke<void>("archive_song", { id });