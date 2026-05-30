import React, { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";

import ModelSelector from "../model-selector";
import UpdateChecker from "../update-checker";

const Footer: React.FC = () => {
  const [version, setVersion] = useState("");

  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const appVersion = await getVersion();
        setVersion(appVersion);
      } catch (error) {
        console.error("Failed to get app version:", error);
        setVersion("0.1.2");
      }
    };

    fetchVersion();
  }, []);

  return (
    <div className="w-full border-t border-slate-800/60 pt-4 bg-slate-950/20 backdrop-blur-sm">
      <div className="flex justify-between items-center text-[11px] px-6 pb-4 text-slate-400 font-medium">
        <div className="flex items-center gap-4">
          <ModelSelector />
        </div>

        {/* Update Status */}
        <div className="flex items-center gap-2">
          <UpdateChecker />
          <span className="text-slate-700">•</span>
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="font-bold text-slate-500 tracking-tight">
            v{version}
          </span>
        </div>
      </div>
    </div>
  );
};

export default Footer;
