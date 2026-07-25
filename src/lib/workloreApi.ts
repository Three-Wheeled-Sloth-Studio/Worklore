import { invoke } from "@tauri-apps/api/core";
import type {
  CloudIdentifierMode,
  ImportSourceResult,
  SourceSummary,
  SourceType,
  VaultSummary,
} from "../domain/types";

export async function createVault(path: string, name: string): Promise<VaultSummary> {
  return invoke<VaultSummary>("create_vault", { path, name });
}

export async function openVault(path: string): Promise<VaultSummary> {
  return invoke<VaultSummary>("open_vault", { path });
}

export async function importSource(
  vaultPath: string,
  sourcePath: string,
  sourceType: SourceType,
): Promise<ImportSourceResult> {
  return invoke<ImportSourceResult>("import_source", {
    vaultPath,
    sourcePath,
    sourceType,
  });
}

export async function listSources(vaultPath: string): Promise<SourceSummary[]> {
  return invoke<SourceSummary[]>("list_sources", { vaultPath });
}

export async function updateCloudIdentifierMode(
  vaultPath: string,
  mode: CloudIdentifierMode,
): Promise<VaultSummary> {
  return invoke<VaultSummary>("update_cloud_identifier_mode", {
    vaultPath,
    mode,
  });
}
