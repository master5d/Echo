import { type FC, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { commands, type VoiceInfo } from "@/bindings";
import { useSettings } from "@/hooks/useSettings";

export const TtsSettings: FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const [voices, setVoices] = useState<VoiceInfo[]>([]);

  useEffect(() => {
    commands
      .ttsListVoices()
      .then((res) => setVoices(res.status === "ok" ? res.data : []))
      .catch(() => setVoices([]));
  }, []);

  if (!settings) return null;

  const enabled = settings.tts_enabled;
  const voiceId = settings.tts_voice_id ?? "";
  const rate = settings.tts_rate ?? 1.0;

  const test = () => {
    void commands.ttsSpeak(
      "Echo speech engine online. Эхо на связи.",
      voiceId || null,
      rate,
    );
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6 p-2">
      <div>
        <h2 className="text-lg font-semibold">{t("settings.tts.title")}</h2>
        <p className="text-sm text-text/50">{t("settings.tts.description")}</p>
      </div>

      {/* Enable */}
      <label className="flex items-center justify-between rounded-xl border border-slate-800 p-4">
        <span className="text-sm">{t("settings.tts.speakAnswers")}</span>
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => void updateSetting("tts_enabled", e.target.checked)}
        />
      </label>

      {/* Voice */}
      <div className="rounded-xl border border-slate-800 p-4 space-y-2">
        <label className="text-sm block">{t("settings.tts.voice")}</label>
        <select
          className="w-full bg-transparent border border-slate-700 rounded p-2 text-sm"
          value={voiceId}
          onChange={(e) =>
            void updateSetting("tts_voice_id", e.target.value || null)
          }
        >
          <option value="">{t("settings.tts.voiceAuto")}</option>
          {voices.map((v) => (
            <option key={v.id} value={v.id}>
              {v.display_name} ({v.language})
            </option>
          ))}
        </select>
      </div>

      {/* Rate */}
      <div className="rounded-xl border border-slate-800 p-4 space-y-2">
        <label className="text-sm flex items-center justify-between">
          <span>{t("settings.tts.rate")}</span>
          <span className="text-text/50">{rate.toFixed(1)}×</span>
        </label>
        <input
          type="range"
          min={0.5}
          max={2.0}
          step={0.1}
          value={rate}
          onChange={(e) =>
            void updateSetting("tts_rate", parseFloat(e.target.value))
          }
          className="w-full"
        />
      </div>

      <button
        type="button"
        onClick={test}
        className="px-4 py-2 rounded bg-indigo-600 text-white text-sm"
      >
        {t("settings.tts.testVoice")}
      </button>
    </div>
  );
};
