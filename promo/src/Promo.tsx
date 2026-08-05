import React from "react";
import { AbsoluteFill } from "remotion";
import { TransitionSeries, linearTiming } from "@remotion/transitions";
import { fade } from "@remotion/transitions/fade";
import {
  SceneBrand,
  SceneCta,
  SceneEdit,
  SceneHook,
  SceneOcr,
  SceneScroll,
  SceneSign,
} from "./scenes";

const t = linearTiming({ durationInFrames: 12 });

/**
 * Scene durations + fade overlaps:
 * 75+70+85+70+80+75+90 - 6*12 = 473 frames ≈ 15.8s @ 30fps
 */
export const TOTAL_DURATION =
  75 + 70 + 85 + 70 + 80 + 75 + 90 - 12 * 6;

export const Promo: React.FC = () => {
  return (
    <AbsoluteFill style={{ backgroundColor: "#E8EEF2" }}>
      <TransitionSeries>
        <TransitionSeries.Sequence durationInFrames={75}>
          <SceneHook />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={t} />
        <TransitionSeries.Sequence durationInFrames={70}>
          <SceneBrand />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={t} />
        <TransitionSeries.Sequence durationInFrames={85}>
          <SceneScroll />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={t} />
        <TransitionSeries.Sequence durationInFrames={70}>
          <SceneEdit />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={t} />
        <TransitionSeries.Sequence durationInFrames={80}>
          <SceneSign />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={t} />
        <TransitionSeries.Sequence durationInFrames={75}>
          <SceneOcr />
        </TransitionSeries.Sequence>
        <TransitionSeries.Transition presentation={fade()} timing={t} />
        <TransitionSeries.Sequence durationInFrames={90}>
          <SceneCta />
        </TransitionSeries.Sequence>
      </TransitionSeries>
    </AbsoluteFill>
  );
};
