import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/useSettings";
import { commands, type ScoreReport } from "@/bindings";
import { SettingsGroup, ToggleSwitch } from "@/components/ui";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";

export const TutorSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting } = useSettings();

  const [reference, setReference] = useState("");
  const [spoken, setSpoken] = useState("");
  const [report, setReport] = useState<ScoreReport | null>(null);
  const [isScoring, setIsScoring] = useState(false);

  const tutorEnabled = getSetting("tutor_enabled") ?? false;

  const handleScore = async () => {
    setIsScoring(true);
    try {
      const result = await commands.tutorScore(reference, spoken);
      setReport(result);
    } catch (err) {
      console.error("Failed to call tutor_score:", err);
    } finally {
      setIsScoring(false);
    }
  };

  return (
    <div className="flex flex-col animate-in fade-in duration-500">
      <SettingsGroup title={t("sidebar.tutor")}>
        <ToggleSwitch
          checked={tutorEnabled}
          onChange={(checked) => updateSetting("tutor_enabled", checked)}
          label={t("settings.tutor.enable.label")}
          description={t("settings.tutor.enable.description")}
        />
      </SettingsGroup>

      <SettingsGroup title={t("settings.tutor.practice")}>
        <div className="flex flex-col gap-4 p-6">
          <div className="flex flex-col gap-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-widest ml-1">
              {t("settings.tutor.reference")}
            </label>
            <Input
              value={reference}
              onChange={(e) => setReference(e.target.value)}
              placeholder="..."
              className="bg-slate-950/50 border-slate-800"
            />
          </div>

          <div className="flex flex-col gap-2">
            <label className="text-xs font-bold text-slate-500 uppercase tracking-widest ml-1">
              {t("settings.tutor.spoken")}
            </label>
            <Input
              value={spoken}
              onChange={(e) => setSpoken(e.target.value)}
              placeholder="..."
              className="bg-slate-950/50 border-slate-800"
            />
          </div>

          <Button
            onClick={handleScore}
            disabled={isScoring || !reference || !spoken}
            className="mt-2 py-3 bg-indigo-600 hover:bg-indigo-500 border-none shadow-lg shadow-indigo-500/20"
          >
            {isScoring ? t("common.loading") : t("settings.tutor.score")}
          </Button>

          {report && (
            <div className="mt-6 flex flex-col gap-5 p-5 bg-indigo-500/5 rounded-2xl border border-indigo-500/20 animate-in zoom-in-95 duration-300">
              <div className="flex items-center justify-between">
                <div className="flex flex-col">
                  <span className="text-4xl font-black text-white tracking-tighter">
                    {Math.round(report.overall)}%
                  </span>
                  <span className="text-[10px] text-indigo-400 font-bold uppercase tracking-widest">
                    {t("settings.tutor.overall")}
                  </span>
                </div>
                <div className="text-right max-w-[60%]">
                  <p className="text-sm text-slate-300 italic leading-relaxed">
                    {report.note}
                  </p>
                </div>
              </div>

              <div className="flex flex-wrap gap-2 pt-5 border-t border-slate-800/50">
                {report.words.map((word, i) => (
                  <div
                    key={i}
                    className={`px-3 py-1.5 rounded-lg text-sm font-bold transition-colors ${
                      word.matched
                        ? "text-green-400 bg-green-400/10 border border-green-400/20"
                        : "text-red-400 bg-red-400/10 border border-red-400/20"
                    }`}
                  >
                    {word.reference}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};
