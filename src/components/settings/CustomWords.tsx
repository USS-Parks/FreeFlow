import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  commands,
  type DictionaryEngineSupport,
  type DictionaryEntry,
  type DictionarySort,
} from "../../bindings";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";

interface CustomWordsProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

const EMPTY_SUPPORT: DictionaryEngineSupport = {
  whisper_initial_prompt_only: true,
  deterministic_replacement: true,
};

export const CustomWords: React.FC<CustomWordsProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const fileInput = useRef<HTMLInputElement>(null);
    const [entries, setEntries] = useState<DictionaryEntry[]>([]);
    const [query, setQuery] = useState("");
    const [sort, setSort] = useState<DictionarySort>("starred");
    const [spokenForm, setSpokenForm] = useState("");
    const [replacement, setReplacement] = useState("");
    const [editingId, setEditingId] = useState<number | null>(null);
    const [busy, setBusy] = useState(false);
    const [support, setSupport] =
      useState<DictionaryEngineSupport>(EMPTY_SUPPORT);

    const load = useCallback(async () => {
      const result = await commands.getDictionaryEntries(query || null, sort);
      if (result.status === "ok") setEntries(result.data);
      else toast.error(result.error);
    }, [query, sort]);

    useEffect(() => {
      void load();
    }, [load]);

    useEffect(() => {
      void commands.getDictionaryEngineSupport().then(setSupport);
    }, []);

    const resetForm = () => {
      setSpokenForm("");
      setReplacement("");
      setEditingId(null);
    };

    const save = async () => {
      if (!spokenForm.trim() || !replacement.trim()) return;
      setBusy(true);
      const result =
        editingId === null
          ? await commands.createDictionaryEntry(spokenForm, replacement, false)
          : await commands.updateDictionaryEntry(
              editingId,
              spokenForm,
              replacement,
              entries.find((entry) => entry.id === editingId)?.starred ?? false,
            );
      setBusy(false);
      if (result.status === "error") {
        toast.error(result.error);
        return;
      }
      resetForm();
      await load();
    };

    const toggleStar = async (entry: DictionaryEntry) => {
      const result = await commands.updateDictionaryEntry(
        entry.id,
        entry.spoken_form,
        entry.replacement,
        !entry.starred,
      );
      if (result.status === "error") toast.error(result.error);
      else await load();
    };

    const remove = async (entry: DictionaryEntry) => {
      if (!window.confirm(t("settings.advanced.customWords.deleteConfirm")))
        return;
      const result = await commands.deleteDictionaryEntry(entry.id);
      if (result.status === "error") toast.error(result.error);
      else await load();
    };

    const exportCsv = async () => {
      const result = await commands.exportDictionaryCsv();
      if (result.status === "error") {
        toast.error(result.error);
        return;
      }
      const url = URL.createObjectURL(
        new Blob([result.data], { type: "text/csv;charset=utf-8" }),
      );
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = "freeflow-dictionary.csv";
      anchor.click();
      URL.revokeObjectURL(url);
    };

    const importCsv = async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      event.target.value = "";
      if (!file) return;
      const result = await commands.importDictionaryCsv(await file.text());
      if (result.status === "error") toast.error(result.error);
      else {
        toast.success(
          t("settings.advanced.customWords.imported", { count: result.data }),
        );
        await load();
      }
    };

    return (
      <div className="space-y-3">
        <SettingContainer
          title={t("settings.advanced.customWords.title")}
          description={t("settings.advanced.customWords.description")}
          descriptionMode={descriptionMode}
          grouped={grouped}
        >
          <div className="flex flex-wrap items-center gap-2">
            <Input
              value={spokenForm}
              onChange={(event) => setSpokenForm(event.target.value)}
              placeholder={t("settings.advanced.customWords.spokenPlaceholder")}
              aria-label={t("settings.advanced.customWords.spokenLabel")}
              maxLength={200}
              variant="compact"
              disabled={busy}
            />
            <Input
              value={replacement}
              onChange={(event) => setReplacement(event.target.value)}
              placeholder={t(
                "settings.advanced.customWords.replacementPlaceholder",
              )}
              aria-label={t("settings.advanced.customWords.replacementLabel")}
              maxLength={4000}
              variant="compact"
              disabled={busy}
            />
            <Button onClick={() => void save()} disabled={busy} size="sm">
              {editingId === null
                ? t("settings.advanced.customWords.add")
                : t("settings.advanced.customWords.save")}
            </Button>
            {editingId !== null && (
              <Button onClick={resetForm} variant="secondary" size="sm">
                {t("settings.advanced.customWords.cancel")}
              </Button>
            )}
          </div>
        </SettingContainer>

        <div className="px-4 space-y-2">
          <div className="flex flex-wrap gap-2">
            <Input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t("settings.advanced.customWords.search")}
              aria-label={t("settings.advanced.customWords.search")}
              variant="compact"
            />
            <select
              value={sort}
              onChange={(event) =>
                setSort(event.target.value as DictionarySort)
              }
              aria-label={t("settings.advanced.customWords.sort")}
              className="rounded-lg border border-mid-gray/20 bg-background px-2 text-sm"
            >
              <option value="starred">
                {t("settings.advanced.customWords.sortStarred")}
              </option>
              <option value="updated">
                {t("settings.advanced.customWords.sortUpdated")}
              </option>
              <option value="spoken_form">
                {t("settings.advanced.customWords.sortAlphabetical")}
              </option>
            </select>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => void exportCsv()}
            >
              {t("settings.advanced.customWords.exportCsv")}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => fileInput.current?.click()}
            >
              {t("settings.advanced.customWords.importCsv")}
            </Button>
            <input
              ref={fileInput}
              type="file"
              accept=".csv,text/csv"
              className="hidden"
              onChange={(event) => void importCsv(event)}
            />
          </div>

          <p className="text-xs text-text/70">
            {support.whisper_initial_prompt_only
              ? t("settings.advanced.customWords.engineSupport")
              : t("settings.advanced.customWords.localReplacement")}
          </p>

          {entries.length === 0 ? (
            <p className="text-sm text-text/60">
              {t("settings.advanced.customWords.empty")}
            </p>
          ) : (
            <div className="space-y-1">
              {entries.map((entry) => (
                <div
                  key={entry.id}
                  className="flex items-center gap-2 rounded-lg border border-mid-gray/20 p-2 text-sm"
                >
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => void toggleStar(entry)}
                    aria-label={t("settings.advanced.customWords.star")}
                  >
                    {entry.starred ? "★" : "☆"}
                  </Button>
                  <div className="min-w-0 flex-1">
                    <span className="font-medium">{entry.spoken_form}</span>
                    <span className="mx-2 text-text/50">→</span>
                    <span className="break-words">{entry.replacement}</span>
                  </div>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => {
                      setEditingId(entry.id);
                      setSpokenForm(entry.spoken_form);
                      setReplacement(entry.replacement);
                    }}
                  >
                    {t("settings.advanced.customWords.edit")}
                  </Button>
                  <Button
                    variant="danger-ghost"
                    size="sm"
                    onClick={() => void remove(entry)}
                  >
                    {t("settings.advanced.customWords.delete")}
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
