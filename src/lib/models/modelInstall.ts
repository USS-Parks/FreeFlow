import { ask } from "@tauri-apps/plugin-dialog";
import type { TFunction } from "i18next";
import { commands, type ModelInstallPlan } from "@/bindings";

const formatBytes = (bytes: number): string =>
  new Intl.NumberFormat(undefined).format(bytes);

export async function requestModelInstallConfirmation(
  modelId: string,
  t: TFunction,
): Promise<ModelInstallPlan | null> {
  const result = await commands.getModelInstallPlan(modelId);
  if (result.status !== "ok") {
    throw new Error(result.error);
  }

  const plan = result.data;
  const licenses = plan.licenses
    .map((license) =>
      t("modelInstall.licenseLine", {
        scope: license.scope,
        name: license.name,
        identifier: license.identifier,
        url: license.url,
        attribution: license.attribution,
      }),
    )
    .join("\n\n");
  const accepted = await ask(
    t("modelInstall.confirmMessage", {
      modelName: plan.display_name,
      sizeBytes: formatBytes(plan.size_bytes),
      sourceRepository: plan.artifact_repository,
      sourceRevision: plan.artifact_revision,
      sourceUrl: plan.source_url,
      baseRepository: plan.base_repository,
      baseRevision: plan.base_revision,
      sha256: plan.sha256,
      destination: plan.destination,
      licenses,
      redistributionStatus: plan.redistribution_status,
    }),
    {
      title: t("modelInstall.confirmTitle"),
      kind: "warning",
    },
  );

  return accepted ? plan : null;
}
