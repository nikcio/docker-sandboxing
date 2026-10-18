import type { Provider } from "../../lib/catalog";
import { isHubSearchProvider } from "../../lib/catalog";

export function normalizeRepository(value: string, kind: Provider["kind"]): string {
  let repository = value.trim().replace(/\/+$/, "");
  if (kind === "github") {
    repository = repository.replace(/^https?:\/\/github\.com\//i, "").replace(/\.git$/, "");
  } else {
    repository = repository
      .replace(/^https?:\/\/hub\.docker\.com\/r\//i, "")
      .replace(/^docker\.io\//i, "");
  }
  return repository;
}

export function repositoryIsValid(value: string, kind: Provider["kind"]) {
  const repository = normalizeRepository(value, kind);
  if (isHubSearchProvider({ repository, kind } as Provider)) return true;
  return kind === "github"
    ? /^[\w.-]+\/[\w.-]+$/.test(repository)
    : /^[a-z0-9][a-z0-9_.-]*\/[a-z0-9][a-z0-9_.-]*$/.test(repository);
}
