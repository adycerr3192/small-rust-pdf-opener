import React from "react";
import {
  AbsoluteFill,
  Easing,
  interpolate,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { colors, displayFont, sansFont } from "./theme";

const ease = Easing.bezier(0.16, 1, 0.3, 1);

export const FadeUp: React.FC<{
  children: React.ReactNode;
  delay?: number;
  style?: React.CSSProperties;
}> = ({ children, delay = 0, style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = frame - delay;
  const opacity = interpolate(t, [0, 0.55 * fps], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: ease,
  });
  const translateY = interpolate(t, [0, 0.55 * fps], [28, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: ease,
  });

  return (
    <div
      style={{
        opacity,
        translate: `0px ${translateY}px`,
        ...style,
      }}
    >
      {children}
    </div>
  );
};

export const SceneShell: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  return (
    <AbsoluteFill
      style={{
        background: `radial-gradient(120% 80% at 50% 0%, ${colors.bg} 0%, ${colors.bgDeep} 70%, #C9D5E0 100%)`,
        fontFamily: sansFont,
        color: colors.ink,
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <AbsoluteFill
        style={{
          backgroundImage:
            "radial-gradient(rgba(11,18,32,0.04) 1px, transparent 1px)",
          backgroundSize: "22px 22px",
          opacity: 0.7,
        }}
      />
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
          padding: "100px 120px",
          gap: 36,
          boxSizing: "border-box",
          position: "relative",
          zIndex: 1,
        }}
      >
        {children}
      </div>
    </AbsoluteFill>
  );
};

export const Headline: React.FC<{
  children: React.ReactNode;
  size?: number;
}> = ({ children, size = 92 }) => (
  <div
    style={{
      fontFamily: displayFont,
      fontSize: size,
      fontWeight: 700,
      lineHeight: 1.05,
      letterSpacing: "-0.02em",
      textAlign: "center",
      maxWidth: 1500,
    }}
  >
    {children}
  </div>
);

export const Subline: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => (
  <div
    style={{
      fontSize: 40,
      fontWeight: 500,
      color: colors.muted,
      textAlign: "center",
      maxWidth: 1100,
      lineHeight: 1.35,
    }}
  >
    {children}
  </div>
);

export const AccentWord: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => <span style={{ color: colors.accent }}>{children}</span>;
