# LRLC saturation methods — RtC excess-conductivity correction + IMTS model

Source: `D:\01. Work\2025\36. LRLC - Pertamina Upstream Innovation\Draft Final Report_Study
of LRLC caused by High Clay Volume (VCL) and Microporosity in Pertamina Fields compiled.docx`
(PHE UI + LAPI ITB study). Jauhar's own research on low-resistivity/low-contrast pay.
Ported into `lrlc.rs` (modules `sw_rtc`, `sw_imts`).

## RtC method (Resistivity correction by Clay & Capillary Water)

Idea: total measured conductivity = clean Archie conductivity + excess conductivity from
clay chemistry (Qv path) AND capillary/micropore water (pore-geometry path). PHIT−PHIFF
(irreducible water) defines the pore-geometry conductivity path.

- Normalized Rt for baseline work: Rt_norm = Rt·(φt/0.35)^m; Ct_norm = 1/Rt_norm.
- Qv = CEC·ρg·(1−φt)/(100·φt)  (CEC in meq/100g, ρg g/cc → Qv meq/cm³).
- Clean baseline in water zones: Co_archie = Swirr²·φt²/Rw; measured excess:
  Cex_log = 1/Rt_log − Co_archie.
- Multiple regression of averaged excess conductivity vs CAPbw (capillary bound-water
  volume, from triple combo — e.g. SSC's CWSH) and Qv gave:
  **Cexcess = (0.45·CAPbw + 0.0057·Qv − 0.0071)·φt·RSF**, RSF≈2.25 (their calibration).
  General form (a·CAPbw + b·Qv + c)·φt·RSF; a = water-proportion path, b = clay-chemistry
  mineral path (small, slightly negative buffer possible), c intercept.
- Corrected resistivity: Rt_corr = 1/(1/Rt_log − Cexcess); saturation:
  **Sw = [Rw·(1/Rt_log − Cexcess)/φt^m]^(1/n)**.

## IMTS model (Integrated Mineral-Textural Scaling)

Calibrates XRD mineralogy to lab CEC and focuses charge on the ACTIVE pore water
(capillary), not total:

- Scaling factor **S = CEC_measured / Σ(Vmin_i,weight × CEC_lit_i)**; literature
  constants: kaolinite 8, illite 25 meq/100g (measured lab CEC ≪ XRD-theoretical → S < 1).
- Qv_bulk = CEC_bulk·ρg/100·(1−φt)/φt; **Qv_eff = Qv_bulk/(1−Swirr)** (charge referenced
  to the conductive fraction; ion concentration per unit available water rises when HC
  displaces free water).
- Corrected resistivities for exponent fitting: Rt_corr = Rt·(1+Rw·B·Qv_eff),
  Ro_corr = Ro·(1+Rw·B·Qv_eff); m* from slope of log(Ro_corr) vs log(φt).
- Full saturation equation (iterative in Sw, Waxman-Smits family with F*=a/φt^m*):
  **Ct = Sw^n*/F* · [Cw + B·S·(ΣVmin_i·CEC_lit_i)·ρg·(1−φt)/(100·φt·(1−Swirr))/Sw]**
  where B = counterion mobility (Waxman-Smits/Juhasz temperature form). Iterate SwT to
  stability; SwE from the CBW split. m*, n* trend high when dispersed kaolinite present.
- Result on their LRLC samples: SwE lower than Archie everywhere; slightly lower than
  Waxman-Smits on most samples → unlocks hidden pay.

Context: Waxman-Smits Co=Cw/F*+B·Qv/F*; dual-water (Clavier); Juhasz normalized Qvn.
IMTS separates Qv into CBW vs CAPBW roles. Inputs pair naturally with SSC/SSPW outputs
(CWSH/CBW) — see `method_ssc_sspw.md`.
