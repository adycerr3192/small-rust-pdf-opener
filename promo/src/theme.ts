import { loadFont as loadDisplay } from "@remotion/google-fonts/Fraunces";
import { loadFont as loadSans } from "@remotion/google-fonts/DMSans";

export const { fontFamily: displayFont } = loadDisplay("normal", {
  weights: ["600", "700"],
  subsets: ["latin"],
});

export const { fontFamily: sansFont } = loadSans("normal", {
  weights: ["400", "500", "700"],
  subsets: ["latin"],
});

export const colors = {
  bg: "#E8EEF2",
  bgDeep: "#D5DEE6",
  ink: "#0B1220",
  muted: "#4A5568",
  accent: "#0D6B5A",
  accentSoft: "#B8E0D6",
  paper: "#FBFCFE",
  cloudWarn: "#C45C26",
  line: "#C5D0DA",
};
