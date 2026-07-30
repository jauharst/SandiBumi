# chartdig — chartbook curve digitizer

Extracts chart curves from the Schlumberger Log Interpretation Charts 2013 PDF at
vector precision. Produces `src/ui/chartOverlays.ts` (GENERATED — 19 chart defs as of
2026-07-20: D-N Por-11..19, PEF Lith-3/4, sonic Por-20/22, Lith-1/2/6) and
`src-tauri/src/neutron_charts.rs` (GENERATED — Por-4/Por-5 neutron matrix equivalence
tables for the `nphimat` module). The PDF itself is copyrighted and NOT in the repo — point
the `CHARTBOOK_PDF` environment variable at your own copy. It is only needed to
digitize NEW charts. Page-extract JSONs (`p*.json`) are working files — regenerate
them from the PDF, don't commit them.

## Usage

```powershell
npm install                             # one-off, in this folder (local package.json —
                                        # keeps pdfjs-dist out of the app's dependencies)
node extract.mjs 237 por11.json         # PDF page -> stroked vector paths + text (with CTM applied)
node gen_dn.mjs                         # D-N family (Por-11..19) from page JSONs
node gen_por20.mjs / gen_por22.mjs / gen_lith1.mjs / gen_lith2.mjs / gen_lith6.mjs
node assemble.mjs                       # merge gen outputs -> src/ui/chartOverlays.ts
node gen_por45.mjs                      # Por-4/5 (needs p228.json/p229.json) -> neutron_charts.rs
```

(`analyze7.mjs` is the original single-chart Por-11/12 pipeline, kept as the
worked-through reference implementation of the method below.)

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
