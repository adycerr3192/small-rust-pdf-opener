import React from "react";
import {
  Easing,
  Img,
  interpolate,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import {
  AccentWord,
  FadeUp,
  Headline,
  SceneShell,
  Subline,
} from "./components";
import { colors, displayFont, sansFont } from "./theme";

const ease = Easing.bezier(0.16, 1, 0.3, 1);

/** Fake continuous-scroll PDF strip */
export const PdfScrollMock: React.FC<{ scrollBoost?: number }> = ({
  scrollBoost = 1,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const y = interpolate(
    frame,
    [0, 3.2 * fps],
    [40, -220 * scrollBoost],
    {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: ease,
    },
  );

  const pages = [0, 1, 2];

  return (
    <div
      style={{
        width: 520,
        height: 420,
        borderRadius: 18,
        background: colors.ink,
        padding: 14,
        boxShadow: "0 28px 60px rgba(11,18,32,0.28)",
        overflow: "hidden",
        boxSizing: "border-box",
      }}
    >
      <div
        style={{
          display: "flex",
          gap: 8,
          marginBottom: 10,
          alignItems: "center",
        }}
      >
        {["#FF5F57", "#FEBC2E", "#28C840"].map((c) => (
          <div
            key={c}
            style={{ width: 10, height: 10, borderRadius: 99, background: c }}
          />
        ))}
        <div
          style={{
            marginLeft: 8,
            color: "#A8B3C0",
            fontSize: 14,
            fontFamily: sansFont,
          }}
        >
          invoice.pdf — local only
        </div>
      </div>
      <div
        style={{
          background: "#2A3340",
          borderRadius: 12,
          height: 360,
          overflow: "hidden",
          position: "relative",
        }}
      >
        <div style={{ translate: `0px ${y}px`, padding: 18 }}>
          {pages.map((p) => (
            <div
              key={p}
              style={{
                background: colors.paper,
                borderRadius: 8,
                height: 200,
                marginBottom: 14,
                padding: 22,
                boxSizing: "border-box",
              }}
            >
              <div
                style={{
                  height: 10,
                  width: "55%",
                  background: colors.ink,
                  opacity: 0.85,
                  marginBottom: 14,
                  borderRadius: 3,
                }}
              />
              {[0.92, 0.78, 0.86, 0.64, 0.8].map((w, i) => (
                <div
                  key={i}
                  style={{
                    height: 7,
                    width: `${w * 100}%`,
                    background: colors.line,
                    marginBottom: 10,
                    borderRadius: 2,
                  }}
                />
              ))}
              {p === 1 ? (
                <div
                  style={{
                    marginTop: 8,
                    alignSelf: "flex-end",
                    width: 120,
                    height: 36,
                    border: `2px solid ${colors.accent}`,
                    borderRadius: 4,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    color: colors.accent,
                    fontSize: 14,
                    fontFamily: displayFont,
                    fontStyle: "italic",
                  }}
                >
                  W. Signature
                </div>
              ) : null}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export const CloudBad: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const shake = Math.sin(frame / 3) * 2;
  const opacity = interpolate(frame, [0, 0.4 * fps], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        opacity,
        translate: `${shake}px 0px`,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 16,
      }}
    >
      <div
        style={{
          width: 340,
          borderRadius: 16,
          background: "#FFF5F0",
          border: `2px solid ${colors.cloudWarn}`,
          padding: 28,
          boxSizing: "border-box",
          textAlign: "center",
        }}
      >
        <div style={{ fontSize: 28, fontWeight: 700, color: colors.cloudWarn }}>
          Upload your PDF to edit
        </div>
        <div style={{ marginTop: 12, fontSize: 20, color: colors.muted }}>
          invoice.pdf → someone else's server
        </div>
      </div>
    </div>
  );
};

export const SceneHook: React.FC = () => (
  <SceneShell>
    <FadeUp>
      <Headline>
        Stop uploading <AccentWord>contracts</AccentWord>
        <br />
        to random PDF sites
      </Headline>
    </FadeUp>
    <FadeUp delay={12}>
      <CloudBad />
    </FadeUp>
  </SceneShell>
);

export const SceneBrand: React.FC = () => (
  <SceneShell>
    <FadeUp>
      <Img
        src={staticFile("app-icon-256.png")}
        style={{
          width: 128,
          height: 128,
          borderRadius: 28,
          boxShadow: "0 16px 40px rgba(11,18,32,0.22)",
        }}
      />
    </FadeUp>
    <FadeUp delay={8}>
      <Headline size={84}>Small Rust PDF Opener</Headline>
    </FadeUp>
    <FadeUp delay={16}>
      <Subline>
        Local-first viewer & light editor — documents stay on your machine
      </Subline>
    </FadeUp>
  </SceneShell>
);

export const SceneScroll: React.FC = () => (
  <SceneShell>
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        gap: 64,
        width: "100%",
        justifyContent: "center",
      }}
    >
      <div style={{ flex: "0 1 640px" }}>
        <FadeUp>
          <Headline size={78}>Fast continuous scroll</Headline>
        </FadeUp>
        <FadeUp delay={10}>
          <Subline>MuPDF rendering. Native. No browser tab.</Subline>
        </FadeUp>
      </div>
      <FadeUp delay={6}>
        <PdfScrollMock />
      </FadeUp>
    </div>
  </SceneShell>
);

const FeatureChip: React.FC<{ label: string; delay: number }> = ({
  label,
  delay,
}) => (
  <FadeUp delay={delay}>
    <div
      style={{
        padding: "18px 28px",
        borderRadius: 14,
        background: colors.paper,
        border: `1.5px solid ${colors.line}`,
        fontSize: 32,
        fontWeight: 600,
        boxShadow: "0 10px 28px rgba(11,18,32,0.08)",
      }}
    >
      {label}
    </div>
  </FadeUp>
);

export const SceneEdit: React.FC = () => (
  <SceneShell>
    <FadeUp>
      <Headline size={80}>
        Light edits. <AccentWord>No suite.</AccentWord>
      </Headline>
    </FadeUp>
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        gap: 18,
        justifyContent: "center",
        maxWidth: 1200,
      }}
    >
      {["Delete", "Rotate", "Reorder", "Crop", "Compress"].map((label, i) => (
        <FeatureChip key={label} label={label} delay={8 + i * 5} />
      ))}
    </div>
  </SceneShell>
);

export const SceneSign: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const draw = interpolate(frame, [0.4 * fps, 1.6 * fps], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: ease,
  });

  return (
    <SceneShell>
      <FadeUp>
        <Headline size={78}>
          Sign <AccentWord>locally</AccentWord>
        </Headline>
      </FadeUp>
      <FadeUp delay={8}>
        <Subline>Draw a stamp or certify with PKCS#12 — keys never leave.</Subline>
      </FadeUp>
      <FadeUp delay={14}>
        <div
          style={{
            width: 520,
            height: 200,
            background: colors.paper,
            borderRadius: 16,
            border: `2px dashed ${colors.accent}`,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            position: "relative",
            overflow: "hidden",
          }}
        >
          <svg width="360" height="100" viewBox="0 0 360 100">
            <path
              d="M20 70 C 60 20, 100 90, 140 50 S 220 20, 260 55 S 320 80, 340 40"
              fill="none"
              stroke={colors.accent}
              strokeWidth="4"
              strokeLinecap="round"
              strokeDasharray={400}
              strokeDashoffset={400 * (1 - draw)}
            />
          </svg>
        </div>
      </FadeUp>
    </SceneShell>
  );
};

export const SceneOcr: React.FC = () => (
  <SceneShell>
    <FadeUp>
      <Headline size={78}>
        OCR <AccentWord>offline</AccentWord>
      </Headline>
    </FadeUp>
    <FadeUp delay={10}>
      <Subline>
        Opt-in local models. Searchable text — without uploading scans.
      </Subline>
    </FadeUp>
    <FadeUp delay={16}>
      <div
        style={{
          display: "flex",
          gap: 24,
          alignItems: "stretch",
        }}
      >
        <div
          style={{
            width: 280,
            height: 180,
            background: colors.paper,
            borderRadius: 12,
            border: `1px solid ${colors.line}`,
            padding: 20,
            boxSizing: "border-box",
            opacity: 0.55,
          }}
        >
          <div style={{ fontSize: 18, marginBottom: 12, color: colors.muted }}>
            Scanned page
          </div>
          {[1, 2, 3, 4].map((i) => (
            <div
              key={i}
              style={{
                height: 10,
                background: colors.line,
                marginBottom: 10,
                borderRadius: 2,
                width: `${70 + i * 5}%`,
              }}
            />
          ))}
        </div>
        <div
          style={{
            alignSelf: "center",
            fontSize: 40,
            color: colors.accent,
            fontWeight: 700,
          }}
        >
          →
        </div>
        <div
          style={{
            width: 280,
            height: 180,
            background: colors.accentSoft,
            borderRadius: 12,
            border: `2px solid ${colors.accent}`,
            padding: 20,
            boxSizing: "border-box",
          }}
        >
          <div
            style={{
              fontSize: 18,
              marginBottom: 12,
              color: colors.accent,
              fontWeight: 700,
            }}
          >
            Selectable text
          </div>
          <div style={{ fontSize: 22, lineHeight: 1.4, color: colors.ink }}>
            Invoice #4821
            <br />
            Due: Aug 2026
            <br />
            Total: $1,240
          </div>
        </div>
      </div>
    </FadeUp>
  </SceneShell>
);

export const SceneCta: React.FC = () => (
  <SceneShell>
    <FadeUp>
      <Headline size={76}>Open source. AGPL-3.0.</Headline>
    </FadeUp>
    <FadeUp delay={10}>
      <div
        style={{
          fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
          fontSize: 36,
          background: colors.ink,
          color: "#E8EEF2",
          padding: "28px 40px",
          borderRadius: 14,
          letterSpacing: "-0.01em",
        }}
      >
        brew install --cask pdf-opener
      </div>
    </FadeUp>
    <FadeUp delay={18}>
      <Subline>github.com/will702/small-rust-pdf-opener</Subline>
    </FadeUp>
  </SceneShell>
);
