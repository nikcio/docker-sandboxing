import { invoke } from "@tauri-apps/api/core";
import type { KitOption, KitArguments } from "./catalog";

export interface SaveTemplateRequest {
  name: string;
  workspace: string;
  agent: KitOption;
  mixins: KitOption[];
  arguments: KitArguments;
  save_path?: string;
}

export interface SavedTemplate {
  id: string;
  name: string;
  path: string;
  agent: string;
  workspace: string;
  kit_version: number;
  content: string;
}

export interface LaunchTemplateRequest {
  id: string;
  name?: string;
  workspace?: string;
}

export const saveTemplate = (request: SaveTemplateRequest) =>
  invoke<SavedTemplate>("save_template", { request });
export const listTemplates = () => invoke<SavedTemplate[]>("list_templates");
export const templateWarnings = () => invoke<string[]>("template_warnings");
export const deleteTemplate = (id: string) => invoke<void>("delete_template", { id });
export const launchTemplate = (request: LaunchTemplateRequest) =>
  invoke<string>("launch_template", { ...request });
