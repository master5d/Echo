import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  MicrophoneIcon,
  TranscriptionIcon,
  CancelIcon,
} from "../components/icons";
import "./RecordingOverlay.css";
import { commands } from "@/bindings";
import i18n, { syncLanguageFromSettings } from "@/i18n";
import { getLanguageDirection } from "@/lib/utils/rtl";

type OverlayState = "preparing" | "recording" | "transcribing" | "processing";

const RecordingOverlay: React.FC = () => {
  const { t } = useTranslation();
  const [isVisible, setIsVisible] = useState(false);
  const [state, setState] = useState<OverlayState>("recording");
  const [levels, setLevels] = useState<number[]>(Array(16).fill(0));
  const [subtitles, setSubtitles] = useState("");
  const smoothedLevelsRef = useRef<number[]>(Array(16).fill(0));
  const direction = getLanguageDirection(i18n.language);

  useEffect(() => {
    const setupEventListeners = async () => {
      // Listen for show-overlay event from Rust
      const unlistenShow = await listen("show-overlay", async (event) => {
        // Sync language from settings each time overlay is shown
        await syncLanguageFromSettings();
        const overlayState = event.payload as OverlayState;
        setState(overlayState);
        setIsVisible(true);
        if (overlayState === "recording") {
          setSubtitles("");
        }
      });

      // Listen for hide-overlay event from Rust
      const unlistenHide = await listen("hide-overlay", () => {
        setIsVisible(false);
      });

      // Listen for mic-level updates
      const unlistenLevel = await listen<number[]>("mic-level", (event) => {
        const newLevels = event.payload as number[];

        // Apply smoothing to reduce jitter
        const smoothed = smoothedLevelsRef.current.map((prev, i) => {
          const target = newLevels[i] || 0;
          return prev * 0.7 + target * 0.3; // Smooth transition
        });

        smoothedLevelsRef.current = smoothed;
        setLevels(smoothed.slice(0, 9));
      });

      // Listen for subtitle updates
      const unlistenSubtitles = await listen<string>("subtitle-update", (event) => {
        setSubtitles(event.payload);
      });

      // Cleanup function
      return () => {
        unlistenShow();
        unlistenHide();
        unlistenLevel();
        unlistenSubtitles();
      };
    };

    setupEventListeners();
  }, []);

  const getIcon = () => {
    if (state === "preparing" || state === "recording") {
      return <MicrophoneIcon />;
    } else {
      return <TranscriptionIcon />;
    }
  };

  return (
    <div className="overlay-container">
      <div
        dir={direction}
        className={`recording-overlay state-${state} ${isVisible ? "fade-in" : ""}`}
      >
        <div className="overlay-left">{getIcon()}</div>

        <div className="overlay-middle">
          {state === "preparing" && (
            <div className="preparing-row">
              <span className="preparing-dots" aria-hidden="true">
                <span />
                <span />
                <span />
              </span>
              <span className="preparing-text">{t("overlay.preparing")}</span>
            </div>
          )}
          {state === "recording" && (
            <div className="bars-container">
              {levels.map((v, i) => (
                <div
                  key={i}
                  className="bar"
                  style={{
                    // Animate transform (compositor-only) instead of height to
                    // avoid layout thrash; bar is full-height, scaled from base.
                    transform: `scaleY(${Math.min(20, 4 + Math.pow(v, 0.7) * 16) / 20})`,
                    transition: "transform 60ms ease-out, opacity 120ms ease-out",
                    opacity: Math.max(0.2, v * 1.7), // Minimum opacity for visibility
                  }}
                />
              ))}
            </div>
          )}
          {state === "transcribing" && (
            <div className="transcribing-text">{t("overlay.transcribing")}</div>
          )}
          {state === "processing" && (
            <div className="transcribing-text">{t("overlay.processing")}</div>
          )}
        </div>

        <div className="overlay-right">
          {state === "recording" && (
            <button
              type="button"
              aria-label={t("overlay.cancel")}
              className="cancel-button"
              onClick={() => {
                commands.cancelOperation();
              }}
            >
              <CancelIcon />
            </button>
          )}
        </div>
      </div>
      {isVisible && subtitles && (
        <div className="subtitle-area fade-in">
          {subtitles}
        </div>
      )}
    </div>
  );
};

export default RecordingOverlay;
