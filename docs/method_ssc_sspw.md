# SSC (Sand-Silt-Clay) + SSPW bound-water method — Jauhar's LQR Balam South workflow

Source: Jauhar's Loglan `ssc_lqr_gap_edit_jau.lls` (+ its `.info` defaults) and
`porosity_sspw.lls` (spec only; exec body not on disk), from his LQR study archive.
Ported into `ssc.rs` / `modules.rs` (modules `ssc`, `sspw`).

## SSC model (Kuttan Malay Basin, GAP 2023 modification)

N-D crossplot with 6 framework points: fluid (RHOB_FL 1, NPHI_FL 1), matrix (2.65, 0),
wet clay (RHOB_WCL 2.3, NPHI_WCL 0.6), dry clay (RHOB_DCL 2.71), wet silt (NPHI_WSI 0.3),
DCLF_SI 0.1 (dry-clay fraction at silt point). GR_MA 10, GR_SH 150, GAS_C 1.6 (defaults).

Algorithm (all lines y=RHOB on x=NPHI):

1. **Gas conditioning** (DEC-086): PHIDI=(RHOB_MA−RHOB)/(RHOB_MA−RHOB_FL); if NPHI≤1.05·PHIDI,
   with Δ=|PHIDI²−NPHI²| and the **user parameter `GAS_C`** (written *c* below, range 0–2):
   PHID=sqrt(PHIDI²−c·Δ/2), RHOB_COR=RHOB_MA−(RHOB_MA−RHOB_FL)·PHID,
   NPHI_COR=sqrt(NPHI²+c·Δ/2); else pass-through.

   *c* is where the answer lands between the two logs: **c=0** leaves the density alone,
   **c=1** is the even split — the RMS midpoint sqrt((PHIDI²+NPHI²)/2), where both corrected
   legs meet — and **c=2** hands the answer to the neutron outright. Above c=1 the two legs
   *cross*: at 1.6 the corrected density is 0.2·PHIDI²+0.8·NPHI² and the corrected neutron its
   mirror, so the D-N crossover is reversed rather than closed.

   **The two source files disagree and Jauhar ruled between them** — the 2022 spec-only
   `porosity_sspw.lls` writes c=1.6, the 2025 exec body `sspw.lls` writes the even split.
   **Both modules now ship c=1.6**: `sspw` under DEC-086 (what it has always run) and `ssc`
   under DEC-088, because in his rock the even split still reads optimistic and that
   observation is about the rock, not about which module is reading it. DEC-088 therefore
   MOVED SSC's gas numbers — a re-run of a gas well reads lower PHIT than it did before
   2026-08-20. Neither is a constant any more; a well whose rock disagrees dials it back.
2. **Derived points**: NPHI_DCL from dry-clay density on the clay-water line
   (M1=(1−RHOB_WCL)/(1−NPHI_WCL)); RHOB_WSI on the matrix–wet-clay line at NPHI_WSI;
   dry silt = intersection of the (fluid→wet silt) line with the dry line (matrix→dry clay).
3. **Projection**: project (NPHI_COR,RHOB_COR) from the fluid point onto the dry line →
   NPHI_PROJ; clamp to [NPHI_MA, NPHI_DCL].
4. **Fractions**: if NPHI_PROJ<NPHI_DSI (sand side): DCLF=M6·NPHI_PROJ with
   M6=DCLF_SI/(NPHI_DSI−NPHI_MA); DSAF=−M7·NPHI_PROJ+1−DCLF with
   M7=(1−DCLF_SI)/(NPHI_DSI−NPHI_MA); DSIF=rest. Clay side: DCLF=M6·NPHI_PROJ+C6 with
   M6=(1−DCLF_SI)/(NPHI_DCL−NPHI_DSI), C6=1−M6·NPHI_DCL; DSAF=0.
5. **Porosity**: RHOMA=Σfrac·ρ (sand RHOB_MA, silt RHOB_DSI, clay RHOB_DCL);
   PHIT=(RHOMA−RHOB_COR)/(RHOMA−RHOB_FL), limit 0.001–0.75.
6. **Volumes** (bulk): VDCL=DCLF·(1−PHIT), VSAND=DSAF·(1−PHIT), VSILT=DSIF·(1−PHIT);
   VWCL=VDCL/(1−PHIT_CL) with PHIT_CL = total porosity of clay (interval param);
   VSH=VWCL+VSILT.
7. **Bound-water split**: PHIE=PHIT−VWCL·PHIT_CL; CBW=PHIT−PHIE (clay-bound);
   PHIT_SH=(RHOB_DSI−RHOB_WSI)/(RHOB_DSI−RHOB_FL); VWSH=VSH/(1−PHIT_SH);
   PHIFF=PHIE−VWSH·PHIT_SH; CWSH=VWSH−VDCL−CBW−VSILT (capillary-bound in silt/shale);
   BW=CBW+CWSH.
8. **SWIRR**: SWIRR_T=BW/PHIT; SWIRR_EFF=1−PHIT·(1−SWIRR_T)/PHIE. Conditioning:
   PHIE≤0.002→CWSH=PHIT−CBW; BW/PHIT<SWIRR_MIN→CWSH=RANNORMAL(SWIRR_MIN·PHIT,0.005)−CBW
   (the SandiBumi port uses deterministic SWIRR_MIN·PHIT); PHIFF recomputed
   = PHIT−CBW−CWSH.
9. **GR-equivalent volumes**: each SSC volume rescaled by VSHGR/VWSH (shale side) or
   (1−VSHGR)/(1−VWSH) (sand side) so track sums honour the chosen VSHGR
   (LINEAR/STIEBER/LARINOV/CLAVIER options).

## SSPW

Key message: **PHIE = PHIT − clay-bound water only; capillary-bound water is inside PHIE;
PHIFF = PHIT − CBW − CAPBW is what flows.**

Spec params: NPHI must be sandstone units; NPHI_MAT 0, RHOB_MAT 2.65, NPHI_SH 0.55,
RHOB_SH 2.4, RHOB_DSH 2.71 (dry shale grain density), VOL_CBW_SH 0.1, SWIRR_MIN 0,
GAS_C 1.6 (see step 1 above — SSPW corrects the density leg only, so the leg-crossing
c>1 produces in SSC has no counterpart here; the symptom is simply a lower PHIT).
Outputs PHIT/PHIE/PHIFF_SSPW, VOL_CLYBNDWAT, VOL_CAPBNDWAT, VOL_BOUNDWAT, SWIRR_SSPW.

**SSPW was reconstructed from spec, not source — needs validation against his the reference suite LAS
exports** (open REVIEW item).

Related: `method_lrlc_rtc_imts.md` (uses CAPBW from these models), `workflow_standards.md`.
