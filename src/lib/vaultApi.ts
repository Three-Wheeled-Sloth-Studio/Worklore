import { invoke } from "@tauri-apps/api/core";
import type { VaultSummary } from "../domain/types";

export async function createVaultInParent(
  parentPath: string,
  name: string,
): Promise<VaultSummary> {
  return invoke<VaultSummary>("create_vault_in_parent", { parentPath, name });
}
