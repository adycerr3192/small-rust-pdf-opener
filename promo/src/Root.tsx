import React from "react";
import { Composition } from "remotion";
import { Promo, TOTAL_DURATION } from "./Promo";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="Promo"
        component={Promo}
        durationInFrames={TOTAL_DURATION}
        fps={30}
        width={1920}
        height={1080}
      />
      <Composition
        id="PromoGif"
        component={Promo}
        durationInFrames={TOTAL_DURATION}
        fps={15}
        width={960}
        height={540}
      />
    </>
  );
};
