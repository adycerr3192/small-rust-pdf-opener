# Promo video (Remotion)

Motion promo for Small Rust PDF Opener (~16s).

## Preview

```bash
cd promo
npm install
npm run dev
```

## Render

```bash
npm run render      # out/promo.mp4  (1920×1080, 30fps)
npm run render:gif  # out/promo.gif  (960×540, 15fps, large)
```

README-sized GIF (via ffmpeg + gifski):

```bash
ffmpeg -y -i out/promo.mp4 -vf "fps=12,scale=720:-1:flags=lanczos" -f yuv4mpegpipe - \
  | gifski --quality=80 --fps 12 -o out/promo-readme.gif -
cp out/promo-readme.gif ../docs/images/promo.gif
cp out/promo.mp4 ../docs/images/promo.mp4
```
