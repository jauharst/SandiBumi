# chartdig — chartbook curve digitizer

Extracts chart curves from the Schlumberger Log Interpretation Charts 2013 PDF
(`D:\01. Work\00. Guidebook\chartbook.pdf`) at vector precision. Used to produce
`src/ui/dnChartData.ts` (Por-11 / Por-12 D-N porosity overlay). Reusable for
future overlays (neutron-sonic CP/Por-20, density-sonic Por-22, M-N, …).

## Usage

```powershell
npm install pdfjs-dist@4.10.38          # one-off, in this folder
node extract.mjs 237 por11.json         # PDF page -> stroked vector paths + text (with CTM applied)
node extract.mjs 238 por12.json
node analyze7.mjs                       # calibrate, digitize, validate, emit dnChartData.ts
```

PDF page = printed page + 12 (Por-11 is printed p.225 = PDF p.237).

## Method (what analyze7 does)

1. **Grid-index calibration**: gray gridlines (`155,156,159`) are exactly 1 pu /
   0.02 g/cc apart; least-squares fit of line index → device coordinate (rms
   ~0.003 pt). Anchor: width-centered axis labels vote for the integer offset
   (x base −4, y base 2.98 on both pages; the black frame is one step outside).
2. **Curves**: long blue (`3,70,145`) polylines merged by endpoint proximity;
   matched to quartz/calcite/dolomite by ρb at their low-porosity end.
3. **Graduation-dash tips are the data** — each ~3–6 pt dash slants up-left from
   its tip, and the tip sits at the exact graduation coordinate; the drawn
   connecting line sags up to ~0.5 pu left of the true graduations (artwork
   artifact), so never sample the line where a dash exists. Each curve's path
   *starts* with its φ=0 dash (tip = point of max ρb). Long dashes (≥4.4 pt)
   mark 5-pu multiples — used as the assignment check.
4. **Chart ρma**: the chartbook draws dolomite for ρma **2.85** (not 2.87) —
   with 2.87 the φ assignment flips around φ30. Quartz 2.65 / calcite 2.71 are
   faithful. ρb of a graduation is analytic: φ·ρf + (1−φ)·ρma_chart.
5. **Validation**: calcite must be the identity on Por-11 (apparent limestone
   porosity axis) — rms 0.13 pu; long dashes on all six curves land on
   5-multiples; both charts' worked examples ((16.5 pu, 2.38 g/cc) → 18 pu
   ~40 % qtz fresh / 20 pu ~55 % qtz salt) reproduce via the scale-line
   construction; independently spot-checked against the rendered page images.
