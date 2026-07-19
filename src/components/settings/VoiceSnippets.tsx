import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { commands, type Snippet, type SnippetSort } from "../../bindings";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";
import { Textarea } from "../ui/Textarea";

interface VoiceSnippetsProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const VoiceSnippets: React.FC<VoiceSnippetsProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const fileInput = useRef<HTMLInputElement>(null);
    const [snippets, setSnippets] = useState<Snippet[]>([]);
    const [query, setQuery] = useState("");
    const [sort, setSort] = useState<SnippetSort>("updated");
    const [name, setName] = useState("");
    const [triggerPhrase, setTriggerPhrase] = useState("");
    const [expansion, setExpansion] = useState("");
    const [editingId, setEditingId] = useState<number | null>(null);
    const [busy, setBusy] = useState(false);

    const load = useCallback(async () => {
      const result = await commands.getSnippets(query || null, sort);
      if (result.status === "ok") setSnippets(result.data);
      else toast.error(result.error);
    }, [query, sort]);

    useEffect(() => {
      void load();
    }, [load]);

    const resetForm = () => {
      setName("");
      setTriggerPhrase("");
      setExpansion("");
      setEditingId(null);
    };

    const save = async () => {
      if (!name.trim() || !triggerPhrase.trim() || !expansion) return;
      setBusy(true);
      const result =
        editingId === null
          ? await commands.createSnippet(name, triggerPhrase, expansion)
          : await commands.updateSnippet(
              editingId,
              name,
              triggerPhrase,
              expansion,
            );
      setBusy(false);
      if (result.status === "error") {
        toast.error(result.error);
        return;
      }
      resetForm();
      await load();
    };

    const remove = async (snippet: Snippet) => {
      if (!window.confirm(t("settings.advanced.voiceSnippets.deleteConfirm")))
        return;
      const result = await commands.deleteSnippet(snippet.id);
      if (result.status === "error") toast.error(result.error);
      else await load();
    };

    const exportJson = async () => {
      const result = await commands.exportSnippetsJson();
      if (result.status === "error") {
        toast.error(result.error);
        return;
      }
      const url = URL.createObjectURL(
        new Blob([result.data], { type: "application/json;charset=utf-8" }),
      );
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = "freeflow-voice-snippets.json";
      anchor.click();
      URL.revokeObjectURL(url);
    };

    const importJson = async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      event.target.value = "";
      if (!file) return;
      const result = await commands.importSnippetsJson(await file.text());
      if (result.status === "error") toast.error(result.error);
      else {
        toast.success(
          t("settings.advanced.voiceSnippets.imported", { count: result.data }),
        );
        await load();
      }
    };

    return (
      <div className="space-y-3">
        <SettingContainer
          title={t("settings.advanced.voiceSnippets.title")}
          description={t("settings.advanced.voiceSnippets.description")}
          descriptionMode={descriptionMode}
          grouped={grouped}
        >
          <div className="grid w-full gap-2">
            <div className="flex flex-wrap gap-2">
              <Input
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder={t("settings.advanced.voiceSnippets.name")}
                aria-label={t("settings.advanced.voiceSnippets.name")}
                maxLength={100}
                variant="compact"
                disabled={busy}
              />
              <Input
                value={triggerPhrase}
                onChange={(event) => setTriggerPhrase(event.target.value)}
                placeholder={t("settings.advanced.voiceSnippets.trigger")}
                aria-label={t("settings.advanced.voiceSnippets.trigger")}
                maxLength={200}
                variant="compact"
                disabled={busy}
              />
            </div>
            <Textarea
              value={expansion}
              onChange={(event) => setExpansion(event.target.value)}
              placeholder={t("settings.advanced.voiceSnippets.expansion")}
              aria-label={t("settings.advanced.voiceSnippets.expansion")}
              maxLength={4000}
              variant="compact"
              disabled={busy}
              className="w-full"
            />
            <div className="flex gap-2">
              <Button onClick={() => void save()} disabled={busy} size="sm">
                {editingId === null
                  ? t("settings.advanced.voiceSnippets.add")
                  : t("settings.advanced.voiceSnippets.save")}
              </Button>
              {editingId !== null && (
                <Button onClick={resetForm} variant="secondary" size="sm">
                  {t("settings.advanced.voiceSnippets.cancel")}
                </Button>
              )}
            </div>
          </div>
        </SettingContainer>

        <div className="px-4 space-y-2">
          <div className="flex flex-wrap gap-2">
            <Input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t("settings.advanced.voiceSnippets.search")}
              aria-label={t("settings.advanced.voiceSnippets.search")}
              variant="compact"
            />
            <select
              value={sort}
              onChange={(event) => setSort(event.target.value as SnippetSort)}
              aria-label={t("settings.advanced.voiceSnippets.sort")}
              className="rounded-lg border border-mid-gray/20 bg-background px-2 text-sm"
            >
              <option value="updated">
                {t("settings.advanced.voiceSnippets.sortUpdated")}
              </option>
              <option value="name">
                {t("settings.advanced.voiceSnippets.sortName")}
              </option>
              <option value="trigger_phrase">
                {t("settings.advanced.voiceSnippets.sortTrigger")}
              </option>
            </select>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => void exportJson()}
            >
              {t("settings.advanced.voiceSnippets.exportJson")}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => fileInput.current?.click()}
            >
              {t("settings.advanced.voiceSnippets.importJson")}
            </Button>
            <input
              ref={fileInput}
              type="file"
              accept=".json,application/json"
              className="hidden"
              onChange={(event) => void importJson(event)}
            />
          </div>

          {snippets.length === 0 ? (
            <p className="text-sm text-text/60">
              {t("settings.advanced.voiceSnippets.empty")}
            </p>
          ) : (
            <div className="space-y-1">
              {snippets.map((snippet) => (
                <div
                  key={snippet.id}
                  className="flex items-start gap-2 rounded-lg border border-mid-gray/20 p-2 text-sm"
                >
                  <div className="min-w-0 flex-1">
                    <div className="font-medium">{snippet.name}</div>
                    <div className="text-text/70">{snippet.trigger_phrase}</div>
                    <pre className="mt-1 whitespace-pre-wrap break-words font-sans text-text/80">
                      {snippet.expansion}
                    </pre>
                  </div>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => {
                      setEditingId(snippet.id);
                      setName(snippet.name);
                      setTriggerPhrase(snippet.trigger_phrase);
                      setExpansion(snippet.expansion);
                    }}
                  >
                    {t("settings.advanced.voiceSnippets.edit")}
                  </Button>
                  <Button
                    variant="danger-ghost"
                    size="sm"
                    onClick={() => void remove(snippet)}
                  >
                    {t("settings.advanced.voiceSnippets.delete")}
                  </Button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    );
  },
);
