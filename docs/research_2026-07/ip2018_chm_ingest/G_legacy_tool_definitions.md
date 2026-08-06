# IP2018 CHM Ingest — Target G: Legacy Borehole-Image, Dipmeter & Acoustic-Waveform Tool Definitions

**Source**: `C:\Program Files\IP2018` — live vendor install of Interactive Petrophysics 2018
(PGL / Lloyd's Register / Geoactive). **Read-only throughout: nothing in the install tree was
modified, renamed, or copied into this repo.**

**Why this target exists**: a file-level diff against `C:\Program Files\IP2025` shows 539 files
present only in IP2018. Among them are five whole families the newer release no longer ships —
legacy borehole-image, dipmeter and acoustic-waveform *tool definitions*, all plain text or XML.

**Tier**: **Tier A throughout** — schemas, unit conventions, curve-mnemonic conventions, the tool
inventory itself, and processing parameter defaults. The one Tier-A/Tier-C boundary in the target is
the `SpeedCorrected` flag, handled in §8. Per-tool button geometry is vendor *data*: catalogued here
by **shape and representative value with its source filename only**, never bulk-copied.

**Redistribution discipline**: these files ship under the vendor EULA. No file is reproduced whole.
Nothing here is to be copied into the SandiBumi repo as data. The deliverable is a catalogue and a
schema description.

**Numeric discipline**: every value below is one actually present in a file, with its source
filename. Nothing is inferred from outside knowledge. Where a file does not state something, this
report says `not present in file` and §9 records it as a gap. Tool geometry, standoff and button
counts are **never** supplied from general knowledge.

---

## 1. Scope and parse result

| ext | count | what it is |
|---|---|---|
| `.itt` | 72 | XML imaging-tool definitions (`InteractivePetrophysicsImagingTool` namespace) |
| `.att` | 60 | XML acoustic-waveform tool definitions with processing parameter defaults |
| `.itp` | 58 | XML pad definitions with button geometry in inches |
| `.bor` | 18 | plain-text button coordinate tables |
| `.eli` | 16 | plain-text electrical-image (dipmeter) parameter files |
| **total** | **224** | catalogued |
| `.t83` | 94 | **binary — excluded by instruction; not decoded, nothing inferred** |

**Parse result: 224/224 files parsed with zero errors.** Referential integrity is clean —
all 47 `.itt` files that use `ReferencedPad`/`ExternalPadID` resolve to a shipped `.itp` `Pad ID`,
with **zero orphans**. That is a meaningful validation: the two XML families are a genuinely
coherent linked pair, not a pile of loose files.

---

## 2. Inventory

### 2.1 `.itt` imaging tools — 72 files

The tool *class* is encoded in the **element name**, not an attribute. Eight classes ship:

| tool element | files | what it models |
|---|---|---|
| `PadBasedTool` | 36 | pad/arm microresistivity imagers (FMI, STAR, EMI, XRMI, CMI, OBMI …) |
| `LWDTool` | 12 | logging-while-drilling azimuthal imagers (ADN, geoVISION, AFR, StarTrak …) |
| `AcousticTool` | 10 | ultrasonic borehole televiewers, amplitude & travel-time (UBI, CBIL, CAST) |
| `DipmeterTool` | 8 | classic dipmeters (HDT, SHDT, OBDT, SED, HDIP) |
| `CaliperTool` | 3 | multi-arm caliper imagers (WGI, FIAC, ICT) |
| `Image360Tool` | 1 | generic 360° image container |
| `Scan360Tool` | 1 | generic 360° scan container |
| `MITTool` | 1 | multi-finger casing-inspection tool |

By declared `Company` attribute:

| vendor | `.itt` files |
|---|---|
| Schlumberger | 26 |
| Halliburton | 13 |
| Baker | 11 |
| Weatherford | 10 |
| (none declared) | 9 |
| Pathfinder | 2 |
| GOWell | 1 |

Full per-file inventory (`ID` and `Name` are verbatim attribute values; `Diam` is `Tool/Diameter`
in inches; `arms`/`pads`/`curves` are counted from the parsed tree):

| file | element | ID | Name | Company | Diam (in) | arms | pads | src curves |
|---|---|---|---|---|---|---|---|---|
| `Baker CBIL AMP.itt` | AcousticTool | Baker CBIL AMP v1.0 | Baker CBIL AMP | Baker | 4 | 0 | 0 | 1 |
| `Baker CBIL TT.itt` | AcousticTool | Baker CBIL TT v1.0 | Baker CBIL TT | Baker | 4 | 0 | 0 | 1 |
| `Baker EARTH.itt` | PadBasedTool | Baker EARTH v1.1 | Baker EARTH | Baker | 3.625 | 6 | 6 | 6 |
| `Baker GeoXplorer  - Techlog Export.itt` | PadBasedTool | Baker GeoXplorer - Techlog Export v1.0 | Baker GeoXplorer - Techlog Export | Baker | 3.625 | 6 | 6 | 6 |
| `Baker GeoXplorer.itt` | PadBasedTool | Baker GeoXplorer v1.1 | Baker GeoXplorer | Baker | 3.625 | 6 | 6 | 6 |
| `Baker HDIP.itt` | DipmeterTool | Baker HDIP v1.0 | Baker HDIP | Baker | 4 | 6 | 6 | 6 |
| `Baker STAR - Recall Export.itt` | PadBasedTool | Baker STAR - Recall Export v1.0 | Baker STAR - Recall Export | Baker | 3.625 | 6 | 6 | 6 |
| `Baker STAR Wide.itt` | PadBasedTool | Baker STAR Wide v1.1 | Baker STAR Wide | Baker | 3.625 | 6 | 6 | 6 |
| `Baker STAR.itt` | PadBasedTool | Baker STAR v1.1 | Baker STAR | Baker | 3.625 | 6 | 6 | 6 |
| `Baker StarTrak.itt` | LWDTool | Baker StarTrak v1.1 | Baker StarTrak | Baker | 4.75 | 0 | 0 | 1 |
| `Baker WGI.itt` | CaliperTool | Baker WGI v1.0 | Baker WGI | Baker | 4 | 6 | 6 | 6 |
| `GOWell MCI.itt` | PadBasedTool | GOWell MCI v1.1 | GOWell MCI | GOWell | 3.625 | 6 | 6 | 6 |
| `Halliburton AFR.itt` | LWDTool | Halliburton AFR v1.0 | Halliburton AFR | Halliburton | 4.75 | 0 | 0 | 1 |
| `Halliburton ALD.itt` | LWDTool | Halliburton ALD v1.0 | Halliburton ALD | Halliburton | 4.75 | 0 | 0 | 1 |
| `Halliburton CAST-F AMP.itt` | AcousticTool | Halliburton CAST-F AMP v1.0 | Halliburton CAST-F AMP | Halliburton | 4 | 0 | 0 | 1 |
| `Halliburton CAST-F TT.itt` | AcousticTool | Halliburton CAST-F TT v1.0 | Halliburton CAST-F TT | Halliburton | 4 | 0 | 0 | 1 |
| `Halliburton CAST-V AMP.itt` | AcousticTool | Halliburton CAST-V AMP v1.0 | Halliburton CAST-V AMP | Halliburton | 4 | 0 | 0 | 1 |
| `Halliburton CAST-V TT.itt` | AcousticTool | Halliburton CAST-V TT v1.0 | Halliburton CAST-V TT | Halliburton | 4 | 0 | 0 | 1 |
| `Halliburton EMI.itt` | PadBasedTool | Halliburton EMI v1.2 | Halliburton EMI | Halliburton | 3.625 | 6 | 6 | 6 |
| `Halliburton FIAC.itt` | CaliperTool | Halliburton FIAC v1.0 | Halliburton FIAC | Halliburton | 4 | 4 | 4 | 4 |
| `Halliburton ICT.itt` | CaliperTool | Halliburton ICT v1.0 | Halliburton ICT | Halliburton | 4 | 6 | 6 | 6 |
| `Halliburton OMRI.itt` | PadBasedTool | Halliburton OMRI v1.1 | Halliburton OMRI | Halliburton | 5.0 | 6 | 6 | 6 |
| `Halliburton SED.itt` | DipmeterTool | Halliburton SED v1.0 | Halliburton SED | Halliburton | 4 | 6 | 6 | 6 |
| `Halliburton XRMI - RECALL Export.itt` | PadBasedTool | Halliburton XRMI - Recall Export v1.0 | Halliburton XRMI - Recall Export | Halliburton | 3.625 | 6 | 6 | 6 |
| `Halliburton XRMI.itt` | PadBasedTool | Halliburton XRMI v1.3 | Halliburton XRMI | Halliburton | 3.625 | 6 | 6 | 6 |
| `Pathfinder iFinder.itt` | LWDTool | Pathfinder iFinder v1.0 | Pathfinder iFinder | Pathfinder | 4.75 | 0 | 0 | 1 |
| `Pathfinder iPZIG.itt` | LWDTool | Pathfinder iPZIG v1.0 | Pathfinder iPZIG | Pathfinder | 4.75 | 0 | 0 | 1 |
| `Schlumberger ADN.itt` | LWDTool | Schlumberger ADN v1.0 | Schlumberger ADN | Schlumberger | 4.75 | 0 | 0 | 1 |
| `Schlumberger Dual OBMI, Lower.itt` | PadBasedTool | Schlumberger Lower OBMI v1.1 | Schlumberger Dual OBMI, Lower | Schlumberger | 5.75 | 4 | 4 | 4 |
| `Schlumberger Dual OBMI, Upper.itt` | PadBasedTool | Schlumberger Lower OBMI v1.1 | Schlumberger Dual OBMI, Upper | Schlumberger | 5.75 | 4 | 4 | 4 |
| `Schlumberger Dual OBMI.itt` | PadBasedTool | Schlumberger Dual OBMI v1.1 | Schlumberger Dual OBMI | Schlumberger | 5.75 | 8 | 8 | 8 |
| `Schlumberger FMI - Geoframe Export (16x12 buttons).itt` | PadBasedTool | Schlumberger FMI Geoframe (16x12 buttons) v1.0 | Schlumberger FMI - Geoframe Export (16x12 buttons) | Schlumberger | 4 | 4 | 8 | 16 |
| `Schlumberger FMI - Geoframe Export (8x24 buttons).itt` | PadBasedTool | Schlumberger FMI Geoframe (8x24 buttons) v1.0 | Schlumberger FMI - Geoframe Export (8x24 buttons)  | Schlumberger | 4 | 4 | 8 | 8 |
| `Schlumberger FMI - Recall Export.itt` | PadBasedTool | Schlumberger FMI Recall v1.1 | Schlumberger FMI - Recall Export | Schlumberger | 4 | 4 | 8 | 8 |
| `Schlumberger FMI Slimhole (no flaps).itt` | PadBasedTool | Schlumberger FMI v1.0 | Schlumberger FMI Slimhole (no flaps) | Schlumberger | 4 | 4 | 4 | 8 |
| `Schlumberger FMI Slimhole - Geoframe Export (8x24 buttons).itt` | PadBasedTool | Schlumberger FMI Geoframe (8x24 buttons) v1.0 | Schlumberger FMI Slimhole - Geoframe Export (8x24 buttons)  | Schlumberger | 4 | 4 | 4 | 4 |
| `Schlumberger FMI-HD.itt` | PadBasedTool | Schlumberger FMI v1.2 | Schlumberger FMI-HD | Schlumberger | 4 | 4 | 8 | 16 |
| `Schlumberger FMI.itt` | PadBasedTool | Schlumberger FMI v1.2 | Schlumberger FMI | Schlumberger | 4 | 4 | 8 | 16 |
| `Schlumberger FMS-A (2arm).itt` | PadBasedTool | Schlumberger FMS-A (2arm) v1.1 | Schlumberger FMS-A (2arm) | Schlumberger | 5 | 2 | 2 | 8 |
| `Schlumberger FMS-B (4arm Slimhole).itt` | PadBasedTool | Schlumberger FMS-B (4arm Slimhole) v1.1 | Schlumberger FMS-B (4arm Slimhole) | Schlumberger | 3.625 | 4 | 4 | 8 |
| `Schlumberger FMS-C (4arm).itt` | PadBasedTool | Schlumberger FMS-C (4arm) v1.1 | Schlumberger FMS-C (4arm) | Schlumberger | 5 | 4 | 4 | 8 |
| `Schlumberger HDT.itt` | DipmeterTool | Schlumberger HDT v1.0 | Schlumberger HDT | Schlumberger | 4 | 4 | 4 | 5 |
| `Schlumberger MicroScope HD - Techlog Export.itt` | LWDTool | Schlumberger MicroScope HD - Techlog Export v1.0 | Schlumberger MicroScope HD - Techlog Export | Schlumberger | 4.75 | 0 | 0 | 1 |
| `Schlumberger MicroScope HD.itt` | LWDTool | Schlumberger MicroScope HD v1.0 | Schlumberger MicroScope HD | Schlumberger | 4.75 | 0 | 0 | 1 |
| `Schlumberger NGI Lower.itt` | PadBasedTool | Schlumberger NGI Lower v1.1 | Schlumberger NGI Lower | Schlumberger | 4.0 | 4 | 4 | 4 |
| `Schlumberger NGI Upper.itt` | PadBasedTool | Schlumberger NGI Upper v1.1 | Schlumberger NGI Upper | Schlumberger | 4.0 | 4 | 4 | 4 |
| `Schlumberger NGI.itt` | PadBasedTool | Schlumberger NGI v1.1 | Schlumberger NGI | Schlumberger | 4.0 | 8 | 8 | 8 |
| `Schlumberger OBDT.itt` | DipmeterTool | Schlumberger OBDT v1.0 | Schlumberger OBDT | Schlumberger | 4 | 4 | 4 | 4 |
| `Schlumberger OBMI.itt` | PadBasedTool | Schlumberger OBMI v1.1 | Schlumberger OBMI | Schlumberger | 5.75 | 4 | 4 | 4 |
| `Schlumberger SHDT.itt` | DipmeterTool | Schlumberger SHDT v1.0 | Schlumberger SHDT | Schlumberger | 4 | 4 | 4 | 10 |
| `Schlumberger UBI AMP.itt` | AcousticTool | Schlumberger UBI AMP v1.0 | Schlumberger UBI AMP | Schlumberger | 4 | 0 | 0 | 1 |
| `Schlumberger UBI TT.itt` | AcousticTool | Schlumberger UBI TT v1.0 | Schlumberger UBI TT | Schlumberger | 4 | 0 | 0 | 1 |
| `Schlumberger geoVISION.itt` | LWDTool | Schlumberger geoVISION (RAB) v1.0 | Schlumberger geoVISION (RAB) | Schlumberger | 4.75 | 0 | 0 | 1 |
| `Weatherford AZD.itt` | LWDTool | Weatherford AZD v1.0 | Weatherford AZD | Weatherford | 4.75 | 0 | 0 | 1 |
| `Weatherford CMI 2.4.itt` | PadBasedTool | Weatherford CMI 2.4 v1.1 | Weatherford CMI 2.4 | Weatherford | 4 | 8 | 8 | 16 |
| `Weatherford CMI 4.1.itt` | PadBasedTool | Weatherford CMI 4.1 v1.1 | Weatherford CMI 4.1 | Weatherford | 4 | 8 | 8 | 16 |
| `Weatherford CMI 5.0.itt` | PadBasedTool | Weatherford CMI 5.0 v1.1 | Weatherford CMI 5.0 | Weatherford | 4 | 8 | 8 | 16 |
| `Weatherford CMI.itt` | PadBasedTool | Weatherford CMI v1.1 | Weatherford CMI | Weatherford | 4 | 8 | 8 | 16 |
| `Weatherford COI.itt` | PadBasedTool | Weatherford COI v1.1 | Weatherford COI | Weatherford | 4 | 8 | 8 | 8 |
| `Weatherford HMI.itt` | PadBasedTool | Weatherford HMI v1.1 | Weatherford HMI | Weatherford | 4 | 6 | 6 | 6 |
| `Weatherford OMI.itt` | PadBasedTool | Weatherford OMI v1.1 | Weatherford OMI | Weatherford | 4 | 6 | 6 | 6 |
| `Weatherford SCMI.itt` | PadBasedTool | Weatherford SCMI v1.1 | Weatherford SCMI | Weatherford | 4 | 8 | 8 | 16 |
| `Weatherford SineWave.itt` | LWDTool | Weatherford SineWave v1.1 | Weatherford SineWave | Weatherford | 4.75 | 0 | 0 | 1 |
| `Acoustic (Generic) AMP.itt` | AcousticTool | Acoustic (Generic) AMP v1.0 | Acoustic (Generic) AMP | *(none)* | 4 | 0 | 0 | 1 |
| `Acoustic (Generic) TT.itt` | AcousticTool | Acoustic (Generic) TT v1.0 | Acoustic (Generic) TT | *(none)* | 4 | 0 | 0 | 1 |
| `Dipmeter 3 Arm (Generic).itt` | DipmeterTool | Dipmeter 3 Arm (Generic) v1.0 | Dipmeter 3 Arm (Generic) | *(none)* | 4 | 3 | 3 | 6 |
| `Dipmeter 4 Arm (Generic).itt` | DipmeterTool | Dipmeter 4 Arm (Generic) v1.0 | Dipmeter 4 Arm (Generic) | *(none)* | 4 | 4 | 4 | 8 |
| `Dipmeter 6 Arm (Generic).itt` | DipmeterTool | Dipmeter 6 Arm (Generic) v1.0 | Dipmeter 6 Arm (Generic) | *(none)* | 4 | 6 | 6 | 12 |
| `Image360.itt` | Image360Tool | Image360 v1.0 | Image360 | *(none)* | 4.75 | 0 | 0 | 1 |
| `LWD (Generic).itt` | LWDTool | LWD (Generic) v1.0 | LWD (Generic) | *(none)* | 4.75 | 0 | 0 | 1 |
| `Scan360.itt` | Scan360Tool | Scan360 v1.0 | Scan360 | *(none)* | 4.75 | 0 | 0 | 1 |
| `Sondex MIT.itt` | MITTool | Sondex MIT v1.0 | Sondex MIT | *(none)* | 4.75 | 0 | 0 | 1 |

### 2.2 `.att` acoustic-waveform tools — 60 files

**Two schema generations ship side by side in the same install** — this is the single most important
structural fact about this family:

| location | files | schema generation |
|---|---|---|
| install root `C:\Program Files\IP2018\*.att` | 23 | **older / flatter** — `ReceiverCurves/<string>`, `DistanceFromTransmitterToReceiver/<double>`, carries `CoreMode`, `VerticalResolution`, `FrequencyVDL`, `FrequencySemblanceVDL`; **no** `RxTxOrientation`, **no** real transmitter–receiver spacings (all zeros) |
| `AcousticWaveforms\Tools\*.att` | 37 | **newer** — `DistanceFromTransmitterToReceivers/DistanceFromTransmitterToReceiver`, `TimeShifts/TimeShift`, `GainShifts/GainShift`; adds `RxTxOrientation`, `TheReceiverGroup`, `GainAction`, `TimeUnits`, `AnisoMethod`, `SembMethod`, `DispersionCorrectionValue`, `NptsToSmooth`, `PlotFileName*`; **carries real TR spacings** |

The newer set is the substantive one — it is where the tool geometry actually lives. The root set
looks like a stripped legacy carry-over whose per-mode geometry fields are all zero.

`.att` declares vendor with an **abbreviation**, unlike `.itt` (see gap G-6):

| vendor code | `.att` files |
|---|---|
| (none declared) | 23 |
| SLB | 9 |
| WFT | 8 |
| HAL | 7 |
| BHI | 6 |
| GE | 2 |
| Pathfinder | 2 |
| Gowell | 2 |
| APS | 1 |

Full per-file inventory (`ToolTemplate/@Name` and `@Company` verbatim; modes are the `ModeName`
values of the `ModeParameter` blocks):

| file | generation | Name | Company | modes | mode list |
|---|---|---|---|---|---|
| `AcousticWaveforms\Tools\AHW-MAS.att` | Tools | MAS | GE | 8 | `MP;ST;XX;XY;YX;YY;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\APS-SureLog_FWS.att` | Tools | SureLog_FWS | APS | 1 | `MP` |
| `AcousticWaveforms\Tools\AZIBAT_BAT_QBAT.att` | Tools | BAT/QBAT/AZIBAT | HAL | 4 | `HF;HB;LF;LB` |
| `AcousticWaveforms\Tools\BSAT.att` | Tools | BSAT | HAL | 3 | `MPup;MPdown;MPall` |
| `AcousticWaveforms\Tools\CLSS.att` | Tools | CLSS | Pathfinder | 1 | `MP` |
| `AcousticWaveforms\Tools\CXD.att` | Tools | CXD | WFT | 10 | `MP;XX;YY;XY;YX;FastShearWaves;SlowShearWaves;STM;STX;STY` |
| `AcousticWaveforms\Tools\CXDSingleSided.att` | Tools | CXD-SingleSided | WFT | 24 | `MP;MA;MB;MC;MD;XX;XA;XB;XC;XD;YY;YA;YB;YC;YD;XY;YX;STM;STX;STY;XApXC;YBpYD;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\CrossWave.att` | Tools | CrossWave | WFT | 17 | `MP;MP0;MP1;MP2;MP3;MP4;MP5;MP6;MP7;MP8;MP9;MP10;MP11;MP12;MP13;MP14;MP15` |
| `AcousticWaveforms\Tools\DAC.att` | Tools | DAC | BHI | 3 | `Short_MP;MP;ST` |
| `AcousticWaveforms\Tools\DAL.att` | Tools | DAL | BHI | 1 | `MP` |
| `AcousticWaveforms\Tools\DSI.att` | Tools | DSI | SLB | 8 | `MP;XX;YY;XY;YX;ST;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\DSLT.att` | Tools | DSLT | SLB | 1 | `MP` |
| `AcousticWaveforms\Tools\Esonic.att` | Tools | eSonic | SLB | 1 | `MP` |
| `AcousticWaveforms\Tools\FWS.att` | Tools | FWS | HAL | 1 | `MP` |
| `AcousticWaveforms\Tools\GE_CDS.att` | Tools | CDS | GE | 8 | `HM;LM;XX;YY;XY;YX;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\GXDT.att` | Tools | XDLT/GXDT | Gowell | 9 | `Short_MP;MP;ST;XX;YY;XY;YX;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\Generic.att` | Tools | Generic | *(none)* | 6 | `MP;XX;YY;XY;YX;ST` |
| `AcousticWaveforms\Tools\GoWell.att` | Tools | HDSL | Gowell | 2 | `MP;ST` |
| `AcousticWaveforms\Tools\HSFWS.att` | Tools | HFWS | HAL | 8 | `MP;ST;XX;YY;XY;YX;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\Isonic.att` | Tools | Isonic/SonicVision | SLB | 2 | `MP;ST` |
| `AcousticWaveforms\Tools\MAC.att` | Tools | MAC | BHI | 4 | `Short_MP;MP;ST;XX` |
| `AcousticWaveforms\Tools\MDA.att` | Tools | MDA | WFT | 2 | `MP;XX` |
| `AcousticWaveforms\Tools\MDX.att` | Tools | MDX | WFT | 8 | `MP;XX;YY;XY;YX;FastShearWaves;SlowShearWaves;STM` |
| `AcousticWaveforms\Tools\SCLSS.att` | Tools | SCLSS | Pathfinder | 1 | `MP` |
| `AcousticWaveforms\Tools\SDTC.att` | Tools | SDTC | SLB | 1 | `MP` |
| `AcousticWaveforms\Tools\SSLT.att` | Tools | SSLT | SLB | 1 | `MP` |
| `AcousticWaveforms\Tools\Scanner.att` | Tools | SonicScanner | SLB | 10 | `MP;XX;YY;XY;YX;ST;NearMPUp;NearMPLo;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\Scope.att` | Tools | SonicScope | SLB | 3 | `MP;DP;QP` |
| `AcousticWaveforms\Tools\ShockWave.att` | Tools | ShockWave | WFT | 2 | `CSG;MP` |
| `AcousticWaveforms\Tools\SoundTrak.att` | Tools | SoundTrak | BHI | 4 | `MH;ML;QH;QL` |
| `AcousticWaveforms\Tools\ThruBit.att` | Tools | ThruBit | SLB | 8 | `MP;ST;XX;YY;XY;YX;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\WFT BHC.att` | Tools | BHC | WFT | 1 | `MP` |
| `AcousticWaveforms\Tools\WFT_MSS.att` | Tools | MSS | WFT | 1 | `MP` |
| `AcousticWaveforms\Tools\WaveSonic.att` | Tools | WaveSonic | HAL | 8 | `MP;ST;XX;YY;XY;YX;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\XBAT.att` | Tools | XBAT | HAL | 12 | `HF;HB;MP;MPA;MPB;MPC;MPD;XX;XXA;XXB;XXC;XXD` |
| `AcousticWaveforms\Tools\XMAC.att` | Tools | XMAC | BHI | 9 | `Short_MP;MP;ST;XX;YY;XY;YX;FastShearWaves;SlowShearWaves` |
| `AcousticWaveforms\Tools\Xaminer.att` | Tools | Xaminer | HAL | 11 | `MP;FarMP;XX;YY;XY;YX;ST;NearMPUp;NearMPLo;FastShearWaves;SlowShearWaves` |
| `AZIBAT.att` | root | AZIBAT | *(none)* | 4 | `HF;HB;LF;LB` |
| `BAT.att` | root | BAT | *(none)* | 4 | `HF;HB;LF;LB` |
| `Big Guns DTS.att` | root | Big Guns DTS | *(none)* | 1 | `MP` |
| `CLSS.att` | root | CLSS | *(none)* | 1 | `MP` |
| `CXD.att` | root | CXD | *(none)* | 1 | `MP` |
| `CrossWave.att` | root | CrossWave | *(none)* | 1 | `Unknown` |
| `DAL.att` | root | DAL | *(none)* | 1 | `MP` |
| `DSI.att` | root | DSI | *(none)* | 6 | `MP;XX;YY;XY;YX;ST` |
| `DSLT.att` | root | DSLT | *(none)* | 1 | `MP` |
| `Esonic.att` | root | Esonic | *(none)* | 1 | `MP` |
| `FWS.att` | root | FWS | *(none)* | 1 | `MP` |
| `Generic.att` | root | Generic | *(none)* | 6 | `Short_MP;MP;XX;YY;XY;YX` |
| `GoWell.att` | root | GoWell | *(none)* | 6 | `MP;XX;YY;XY;MP;ST` |
| `HSFWS.att` | root | HSFWS | *(none)* | 1 | `MP` |
| `Isonic.att` | root | Isonic | *(none)* | 1 | `MP` |
| `QBAT.att` | root | QBAT | *(none)* | 4 | `HF;HB;LF;LB` |
| `SCLSS.att` | root | SCLSS | *(none)* | 1 | `MP` |
| `Scanner.att` | root | Scanner | *(none)* | 6 | `MP;XX;YY;XY;YX;ST` |
| `Scope.att` | root | Scope | *(none)* | 3 | `MP;DP;QP` |
| `ShockWave.att` | root | ShockWave | *(none)* | 1 | `Unknown` |
| `WFT BHC.att` | root | WFT BHC | *(none)* | 1 | `MP` |
| `XBAT.att` | root | XBAT | *(none)* | 4 | `HF;HB;LF;LB` |
| `XMAC.att` | root | XMAC | BHI | 6 | `Short_MP;MP;XX;YY;XY;YX` |

### 2.3 `.itp` pad definitions — 58 files

`.itp` files carry **no `Company` attribute** — vendor is only inferable from the filename and from
which `.itt` references them. `total buttons` is the sum of `Count` over the file's `ButtonRow`
collections.

| file | Pad ID | Name | rows | total buttons |
|---|---|---|---|---|
| `Baker EARTH Lower.itp` | Baker EARTH Lower v1.0 | Baker EARTH Lower | 1 | 8 |
| `Baker EARTH Upper.itp` | Baker EARTH Upper v1.0 | Baker EARTH Upper | 1 | 8 |
| `Baker GeoXplorer Lower - Techlog Export.itp` | Baker GeoXplorer Lower - Techlog Export v1.0 | Baker GeoXplorer Lower - Techlog Export | 1 | 10 |
| `Baker GeoXplorer Lower.itp` | Baker GeoXplorer Lower v1.0 | Baker GeoXplorer Lower | 1 | 10 |
| `Baker GeoXplorer Upper  - Techlog Export.itp` | Baker GeoXplorer Upper - Techlog Export v1.0 | Baker GeoXplorer Upper - Techlog Export | 1 | 10 |
| `Baker GeoXplorer Upper.itp` | Baker GeoXplorer Upper v1.0 | Baker GeoXplorer Upper | 1 | 10 |
| `Baker HDIP.itp` | Baker HDIP v1.0 | Baker HDIP | 1 | 1 |
| `Baker STAR Lower - Recall Export.itp` | Baker STAR Lower - Recall Export v1.0 | Baker STAR Lower - Recall Export | 2 | 24 |
| `Baker STAR Lower.itp` | Baker STAR Lower v1.0 | Baker STAR Lower | 2 | 24 |
| `Baker STAR Upper - Recall Export.itp` | Baker STAR Upper - Recall Export v1.0 | Baker STAR Upper - Recall Export | 2 | 24 |
| `Baker STAR Upper.itp` | Baker STAR Upper v1.0 | Baker STAR Upper | 2 | 24 |
| `Baker STAR Wide Lower.itp` | Baker STAR Wide Lower v1.0 | Baker STAR Wide Lower | 2 | 24 |
| `Baker STAR Wide Upper.itp` | Baker STAR Wide Upper v1.0 | Baker STAR Wide Upper | 2 | 24 |
| `Baker WGI.itp` | Baker WGI v1.0 | Baker WGI | 1 | 1 |
| `Dipmeter Generic Pad.itp` | Dipmeter Generic Pad v1.0 | Dipmeter Generic Pad | 2 | 2 |
| `GOWell MCI Lower.itp` | GOWell MCI Lower v1.0 | GOWell MCI Lower | 2 | 24 |
| `GOWell MCI Upper.itp` | GOWell MCI Upper v1.0 | GOWell MCI Upper | 2 | 24 |
| `Halliburton EMI Lower.itp` | Halliburton EMI Lower v1.1 | Halliburton EMI Lower | 2 | 25 |
| `Halliburton EMI Upper.itp` | Halliburton EMI Upper v1.1 | Halliburton EMI Upper | 2 | 25 |
| `Halliburton FIAC.itp` | Halliburton FIAC v1.0 | Halliburton FIAC | 1 | 1 |
| `Halliburton ICT.itp` | Halliburton ICT v1.0 | Halliburton ICT | 1 | 1 |
| `Halliburton OMRI.itp` | Halliburton OMRI v1.0 | Halliburton OMRI | 1 | 6 |
| `Halliburton SED.itp` | Halliburton SED v1.0 | Halliburton SED | 1 | 1 |
| `Halliburton XRMI Lower - Recall Export.itp` | Halliburton XRMI Lower - Recall Export v1.0 | Halliburton XRMI Lower - Recall Export | 2 | 25 |
| `Halliburton XRMI Lower.itp` | Halliburton XRMI Lower v1.1 | Halliburton XRMI Lower | 2 | 25 |
| `Halliburton XRMI Upper - Recall Export.itp` | Halliburton XRMI Upper - Recall Export v1.0 | Halliburton XRMI Upper - Recall Export | 2 | 25 |
| `Halliburton XRMI Upper.itp` | Halliburton XRMI Upper v1.1 | Halliburton XRMI Upper | 2 | 25 |
| `Schlumberger FMI - Geoframe Export (16x12 buttons).itp` | Schlumberger FMI Geoframe (16x12 buttons) v1.0 | Schlumberger FMI - Geoframe Export (16x12 buttons) | 2 | 24 |
| `Schlumberger FMI - Geoframe Export (8x24 buttons).itp` | Schlumberger FMI Geoframe (8x24 buttons) v1.0 | Schlumberger FMI - Geoframe Export (8x24 buttons) | 1 | 24 |
| `Schlumberger FMI - Recall Export.itp` | Schlumberger FMI Recall v1.0 | Schlumberger FMI - Recall Export | 1 | 24 |
| `Schlumberger FMI.itp` | Schlumberger FMI v1.0 | Schlumberger FMI | 2 | 24 |
| `Schlumberger FMS-A (2arm).itp` | Schlumberger FMS-A (2arm) v1.0 | Schlumberger FMS-A (2arm) | 4 | 27 |
| `Schlumberger FMS-B (4arm Slimhole).itp` | Schlumberger FMS-B (4arm Slimhole) v1.0 | Schlumberger FMS-B (4arm Slimhole) | 2 | 16 |
| `Schlumberger FMS-C (4arm).itp` | Schlumberger FMS-C (4arm) v1.0 | Schlumberger FMS-C (4arm) | 2 | 16 |
| `Schlumberger HDT SB.itp` | Schlumberger HDT v1.0 SB | Schlumberger HDT | 2 | 2 |
| `Schlumberger HDT.itp` | Schlumberger HDT v1.0 | Schlumberger HDT | 1 | 1 |
| `Schlumberger NGI Lower.itp` | Schlumberger NGI Lower v1.0 | Schlumberger NGI Lower | 1 | 24 |
| `Schlumberger NGI Upper.itp` | Schlumberger NGI Upper v1.0 | Schlumberger NGI Upper | 1 | 24 |
| `Schlumberger NGI.itp` | Schlumberger NGI v1.0 | Schlumberger NGI | 1 | 24 |
| `Schlumberger OBDT.itp` | Schlumberger OBDT v1.0 | Schlumberger OBDT | 1 | 1 |
| `Schlumberger OBMI.itp` | Schlumberger OBMI v1.0 | Schlumberger OBMI | 1 | 5 |
| `Schlumberger SHDT SB.itp` | Schlumberger SHDT v1.0 SB | Schlumberger SHDT | 3 | 3 |
| `Schlumberger SHDT.itp` | Schlumberger SHDT v1.0 | Schlumberger SHDT | 2 | 2 |
| `Weatherford CMI 2.4 Lower.itp` | Weatherford CMI 2.4 Lower v1.0 | Weatherford CMI 2.4 Lower | 2 | 8 |
| `Weatherford CMI 2.4 Upper.itp` | Weatherford CMI 2.4 Upper v1.0 | Weatherford CMI 2.4 Upper | 2 | 8 |
| `Weatherford CMI 4.1 Lower.itp` | Weatherford CMI 4.1 Lower v1.0 | Weatherford CMI 4.1 Lower | 2 | 24 |
| `Weatherford CMI 4.1 Upper.itp` | Weatherford CMI 4.1 Upper v1.0 | Weatherford CMI 4.1 Upper | 2 | 20 |
| `Weatherford CMI 5.0 Lower.itp` | Weatherford CMI 5.0 Lower v1.0 | Weatherford CMI 5.0 Lower | 2 | 24 |
| `Weatherford CMI 5.0 Upper.itp` | Weatherford CMI 5.0 Upper v1.0 | Weatherford CMI 5.0 Upper | 2 | 20 |
| `Weatherford CMI Lower.itp` | Weatherford CMI Lower v1.0 | Weatherford CMI Lower | 2 | 24 |
| `Weatherford CMI Upper.itp` | Weatherford CMI Upper v1.0 | Weatherford CMI Upper | 2 | 20 |
| `Weatherford COI Lower.itp` | Weatherford COI Lower v1.0 | Weatherford COI Lower | 1 | 10 |
| `Weatherford COI Upper.itp` | Weatherford COI Upper v1.0 | Weatherford COI Upper | 1 | 8 |
| `Weatherford HMI Lower.itp` | Standard HMI Lower v1.0 | Standard HMI Lower | 2 | 25 |
| `Weatherford HMI Upper.itp` | Standard HMI Upper v1.0 | Standard HMI Upper | 2 | 25 |
| `Weatherford OMI.itp` | Weatherford OMI v1.0 | Weatherford OMI | 1 | 8 |
| `Weatherford SCMI Lower.itp` | Weatherford SCMI Lower v1.0 | Weatherford SCMI Lower | 2 | 8 |
| `Weatherford SCMI Upper.itp` | Weatherford SCMI Upper v1.0 | Weatherford SCMI Upper | 2 | 8 |

### 2.4 `.bor` button-order tables — 18 files

These carry **no machine-readable identity at all** — tool name, pad numbers, rotation sense and
depth reference exist only in `$` comments (gap G-3). `rows` = one row per button.

| file | rows | cols | tool identity — *from the `$` comment only* |
|---|---|---|---|
| `EI_Pad135.bor` | 8 | 2 | Button order for Baker EI tool |
| `EI_Pad246.bor` | 8 | 2 | Button order for Baker EI tool |
| `HAL_EMIpads_1_3_5.bor` | 25 | 2 | Button order for Halliburtom EMI tool. Pads 1, 3 and 4 |
| `HAL_EMIpads_2_4_6.bor` | 25 | 2 | Button order for Halliburtom EMI tool. Pads 2, 4 and 6 |
| `HAL_OMRIpads_1_3_5.bor` | 6 | 2 | Button order for Halliburtom OMRI tool. Pads 1, 3 and 5 |
| `HAL_OMRIpads_2_4_6.bor` | 6 | 2 | Button order for Halliburtom OMRI tool. Pads 2, 4 and 6 |
| `OBMI_button_order.bor` | 5 | 2 | Button order for Schlumberger OBMI tool |
| `STAR_Pad135.bor` | 24 | 2 | Button order for Baker STAR tool pads 1, 3 and 5 |
| `STAR_Pad246.bor` | 24 | 2 | Button order for Baker STAR tool pads 2,4 and 6 |
| `WSTAR_Pad135.bor` | 24 | 2 | Button order for Baker Wide STAR tool pads 1, 3 and 5 |
| `WSTAR_Pad246.bor` | 24 | 2 | Button order for Baker Wide STAR tool pads 2,4 and 6 |
| `Weatherford HMI lower.bor` | 25 | 2 | Button order for Weatherford Precision Drilling HMI tool lower pads |
| `Weatherford HMI upper.bor` | 25 | 2 | Button order for Weatherford Precision Drilling HMI tool upper pads |
| `Weatherford OMI.bor` | 8 | 1 | Button order for Weatherford Precision Drilling HMI tool upper pads |
| `Weatherford_HMI_lower.bor` | 25 | 2 | Button order for Weatherford Precision Drilling HMI tool lower pads |
| `Weatherford_HMI_upper.bor` | 25 | 2 | Button order for Weatherford Precision Drilling HMI tool upper pads |
| `XRMI_lower.bor` | 25 | 2 | Button order for Halliburton XRMI pads 2, 4 and 6 |
| `XRMI_upper.bor` | 25 | 2 | Button order for Halliburton XRMI pads 1, 3 and 5 |

### 2.5 `.eli` electrical-image parameter files — 16 files

The `.eli` is the **legacy predecessor** of the `.itt`/`.itp` pair, and it is the format that
references `.bor` files. Tool identity lives in a free-text `$Tool :` line.

| file | `$Tool :` line (verbatim) | pad rows | banner date |
|---|---|---|---|
| `4Arm Dipmeter Image.eli` | Create image from 4 arm dipmeter curves | 4 | 26/11/02 10:02:20 |
| `ADNRosi.eli` | Anadrill rotation density image ROSI | 1 | 03/09/2007 09:49:29 |
| `Baker EI.eli` | Baker Atlas Earth Imager | 6 | 03/27/2009 15:33:39 |
| `Baker STAR.eli` | Baker Atlas STAR (II and III) | 6 | 03/27/2009 15:33:39 |
| `Baker Wide STAR.eli` | Baker Atlas Wide STAR | 6 | 03/27/2009 15:33:39 |
| `FMI processed.eli` | FMI processed data | 8 | 09/01/2008 17:37:26 |
| `FMI.eli` | FMI image creation | 16 | 25/11/02 15:39:09 |
| `FMS 1 crv per btn.eli` | FMS-C 1 curve per button | 64 | 1/6/2003 4:27:56 PM |
| `Halliburton_EMI_Image.eli` | Halliburton EMI | 6 | for Halliburton EMI 6 arm tool |
| `Halliburton_OMRI.eli` | Halliburton OMRI tool | 6 | 08/11/2013 08:41:00 |
| `OBMI.eli` | Schlumberger OBMI | 4 | 28/04/2004 10:47:08 |
| `Weatherford HMI.eli` | Weatherford HMI tool | 6 | 22/06/2007 10:17:37 |
| `Weatherford OMI.eli` | Weatherford OMI tool | 6 | 19/08/2009 08:41:00 |
| `Weatherford_CMI.eli` | *(blank)* | 16 | 05/06/2009 16:53:43 |
| `Weatherford_HMI.eli` | Weatherford HMI tool | 6 | 22/06/2007 10:17:37 |
| `XRMI.eli` | XRMI image creation | 6 | 10/11/11 15:53:00 |

---

## 3. Schema descriptions

### 3.1 `.itt` — imaging tool definition (XML)

Root `<InteractivePetrophysicsImagingTool>` in namespace
`http://www.InteractivePetrophysics.com/ImageTool`, preceded by an **XML comment that documents the
units and enumerations**. Those comments are the single most useful thing in the whole target —
seven variants ship, keyed to tool class:

| comment variant | files | what it adds over the common core |
|---|---|---|
| 1 | 15 | `AzimuthReference` still live (TrueNorth/MagneticNorth); `MagneticDeclinationValue/Units` = Degrees, Gradians or Radians; **no** `Stage` — arms hang directly off the tool<br>*e.g.* `Baker HDIP.itt`, `Baker STAR - Recall Export.itt`, `Baker WGI.itt` |
| 2 | 3 | as variant 1 but adds `Stage/Angle` (degrees) and `Stage/VerticalOffset` (inches); documents `MagneticInclinationValue/Units`<br>*e.g.* `Schlumberger Dual OBMI, Lower.itt`, `Schlumberger Dual OBMI, Upper.itt`, `Schlumberger Dual OBMI.itt` |
| 3 | 4 | `AzimuthReference` + all magnetic declination/inclination fields declared **obsolete — 'if specified their values will be ignored'**; no `Stage`<br>*e.g.* `Schlumberger FMI - Geoframe Export (16x12 buttons).itt`, `Schlumberger FMS-A (2arm).itt`, `Schlumberger FMS-B (4arm Slimhole).itt` |
| 4 | 25 | as variant 3, obsolete magnetics, **plus** `Stage/Angle` and `Stage/VerticalOffset` — this is the modern pad-tool form<br>*e.g.* `Baker EARTH.itt`, `Baker GeoXplorer  - Techlog Export.itt`, `Baker GeoXplorer.itt` |
| 5 | 3 | short form for image containers: adds `Tool/RenderStyle` = Normal or Interpolated, `Tool/HoleSizeMethod` = BitSize or CaliperOrRadius<br>*e.g.* `Image360.itt`, `Scan360.itt`, `Sondex MIT.itt` |
| 6 | 10 | short form for acoustic tools: adds `Tool/Sector1Alignment` = Left or Center<br>*e.g.* `Acoustic (Generic) AMP.itt`, `Acoustic (Generic) TT.itt`, `Baker CBIL AMP.itt` |
| 7 | 12 | short form for LWD tools: `Sector1Alignment` + `RenderStyle` + `HoleSizeMethod`<br>*e.g.* `Baker StarTrak.itt`, `Halliburton AFR.itt`, `Halliburton ALD.itt` |

**Units and enumerations the comments state verbatim** (quoted short, attributed):

| field | unit / enumeration | stated in |
|---|---|---|
| `Tool/Diameter` | inches | all 7 comment variants |
| `Tool/DepthOfInvestigationValue/Units` | Centimetres, Decimetres, Feet, Inches, Metres, Millimetres or Yards | all 7 |
| `Stage/Angle`, `Arm/Angle` | degrees | variants 1–4 |
| `Stage/VerticalOffset`, `Arm/VerticalOffset` | inches | variants 1–4 |
| `DefinedPad/Type` | Arm or Flapper (default Arm) | variants 1–4 |
| `DefinedPad/VerticalOffset`, `/HorizontalOffset` | inches | variants 1–4 |
| `ButtonRow/ButtonWidth`, `/HorizontalOffset`, `/HorizontalStep`, `/VerticalOffset`, `/VerticalStep` | inches | variants 1–4 and the `.itp` header |
| `SourceDataType` | Resistivity or Conductivity (default Resistivity) — "Controls how EMEX corrections are applied" | variants 1–4 |
| `Tool/RenderStyle` | Normal or Interpolated | variants 5, 7 |
| `Tool/HoleSizeMethod` | BitSize or CaliperOrRadius | variants 5, 7 |
| `Tool/Sector1Alignment` | Left or Center | variants 6, 7 |
| `MagneticDeclinationValue/Units` | Degrees, Gradians or Radians | variant 1 |
| `DelayValue` | `Units="Microseconds"` (attribute) | AcousticTool files |
| `FluidSlownessValue` | `Units="MicrosecondsPerFoot"` (attribute) | AcousticTool files |
| `AccelerometerOffset` | `Units="Inches"` (attribute) | 24 files |
| `SectorOffset` | `Units="Degrees"` (attribute) | 2 files |

**Element tree.** Tool element attributes: `ID`, `Name`, `TemplateName`, `Company`, `Diameter`.
Children fall in three groups:

1. **Scalar settings** — `DefaultPaletteFileName`, `ReversePalette`, `ButtonsAligned`, `PadsAligned`,
   `MagneticDeclinationApplied`, `CenterReProjectionNotRequired`, `RenderStyle`, `HoleSizeMethod`,
   `ToolAlignment`, `ToolOrientation`, `StagesAligned`, `NavigationAligned`, `ReferenceStage`,
   `DivideTravelTimeByConstant`, `HoleDeviationRequired`, `Description`.
2. **`*Curve` elements** — each names the LAS mnemonic(s) that fill a role. Catalogued in §5.
3. **Geometry tree** (pad/dipmeter/caliper classes only):
   `Stages > Stage > Arms > Arm > Pads > Pad`. `Arm` carries `Angle` (degrees) and `VerticalOffset`
   (inches) and **must have either a `CaliperCurve` or a `RadiusCurve`** (stated in the comment).
   `Pad` is `xsi:type="ReferencedPad"` (points at an external `.itp` via `ExternalPadID`) or an
   inline `DefinedPad`. `Arm` also carries `<DynamicOffsets A..F>`.

Element frequency across all 72 files:

| element | files | | element | files |
|---|---|---|---|---|
| `HoleDeviationAngleCurve` | 72 | | `CenterReProjectionNotRequired` | 17 |
| `HoleDeviationAzimuthCurve` | 72 | | `MagneticDeclinationValue` | 16 |
| `DepthOfInvestigationValue` | 72 | | `MagneticDeclinationCurve` | 16 |
| `DepthOfInvestigationCurve` | 68 | | `RenderStyle` | 15 |
| `Pad1AzimuthCurve` | 57 | | `CaliperCurve` | 15 |
| `RelativeBearingCurve` | 57 | | `HoleSizeMethod` | 15 |
| `ZAccelerationCurve` | 49 | | `ToolAlignment` | 15 |
| `XAccelerometerCurve` | 45 | | `TravelTimeCurve` | 10 |
| `YAccelerometerCurve` | 45 | | `DelayValue` | 10 |
| `ZAccelerometerCurve` | 45 | | `FluidSlownessValue` | 10 |
| `XMagnetometerCurve` | 45 | | `FluidSlownessCurve` | 10 |
| `YMagnetometerCurve` | 45 | | `DivideTravelTimeByConstant` | 6 |
| `ZMagnetometerCurve` | 45 | | `Description` | 6 |
| `GammaRayCurve` | 45 | | `ToolOrientation` | 4 |
| `SurfaceVelocityCurve` | 42 | | `RadiusCurve` | 4 |
| `CumulativeTimeCurve` | 39 | | `StagesAligned` | 3 |
| `ReversePalette` | 37 | | `ReferenceStage` | 3 |
| `IntervalTimeCurve` | 36 | | `NavigationAligned` | 3 |
| `PadsAligned` | 35 | | `AdditionalNavigationOffset` | 2 |
| `ButtonsAligned` | 34 | | `SpeedButtonCurve` | 2 |
| `SourceCurve` | 25 | | `SectorOffset` | 2 |
| `EmexCurrentCurve` | 25 | | `HoleDeviationRequired` | 1 |
| `AccelerometerOffset` | 24 | | `BitSizeValue` | 1 |
| `DefaultPaletteFileName` | 24 | | `BitSizeCurve` | 1 |
| `MagneticDeclinationApplied` | 22 | |  |  |

### 3.2 `.att` — acoustic-waveform tool definition (XML)

Root `<AcousticWaveformToolDefinition>`. Body is three block types:

| block | cardinality | purpose |
|---|---|---|
| `<ToolTemplate>` | 1 | tool identity, output-curve lists, anisotropy/dispersion switches, plot templates |
| `<ModeParameter>` | N | **per-mode slowness-processing defaults** — filter band, slowness search, time gate, semblance cutoff |
| `<ModeSettings>` | N | **per-mode acquisition geometry** — receiver count, transmitter→receiver spacings, depth shift, sample rate, input-curve wildcard mask |

`ModeParameter` and `ModeSettings` are joined on `ModeName`. A *mode* is an acquisition/processing
channel: `MP` monopole, `ST` Stoneley, `XX`/`YY`/`XY`/`YX` cross-dipole tensor components,
`FastShearWaves`/`SlowShearWaves` the rotated shear pair, plus tool-specific ones
(`HF`/`HB`/`LF`/`LB` on the BAT family, `MP0`…`MP15` on CrossWave).

`<ToolTemplate>` attributes (60 files):

| attribute | present in | note |
|---|---|---|
| `Name` | 60/60 | tool name |
| `Company` | 60/60 | vendor abbreviation |
| `PrimaryOutputCurves` | 60/60 | `;`-separated output mnemonics |
| `SecondaryOutputCurves` | 60/60 | `;`-separated derived/QC mnemonics |
| `AzimuthCurve` | 60/60 | empty in 59/60 |
| `AzimuthOffset` | 60/60 | degrees |
| `Comments` | 60/60 | free text |
| `ModeOutputCurveMask` | 60/60 | wildcard |
| `AnisoDiagnostics` | 60/60 | boolean |
| `AnisoLog` | 60/60 | boolean |
| `SaveAniso` | 60/60 | boolean |
| `PreviewLog` | 60/60 | boolean |
| `SaveResults` | 60/60 | boolean |
| `FillGaps` | 60/60 | boolean |
| `DispersionCorrection` | 60/60 | boolean |
| `GainAction` | 37/60 | **newer schema only** |
| `TimeUnits` | 37/60 | **newer schema only** — literal `µs` |
| `AnisoRotation` | 37/60 | **newer only** — +1/-1 rotation sense |
| `PreviewPane` | 37/60 | **newer only** |
| `DispersionCorrectionValue` | 37/60 | **newer only** — fraction |
| `NptsToSmooth` | 37/60 | **newer only** |
| `PlotFileNameBasic` | 37/60 | **newer only** — plot template |
| `PlotFileNameVendor` | 37/60 | **newer only** |
| `PlotFileNameAnisotropy` | 37/60 | **newer only** |
| `AnisoMethod` | 36/60 | **newer only** |
| `SemblanceDescription` | 36/60 | **newer only** |
| `SembMethod` | 36/60 | **newer only** |
| `Diagnostics` | 23/60 | **older schema only** |
| `DispersionCorrectionPercentage` | 2/60 | only 2 files |

`<ModeParameter>` attributes (266 mode blocks across all 60 files):

| attribute | present in blocks | meaning |
|---|---|---|
| `ModeName` | 266 | join key to `ModeSettings` |
| `CalculateMode` | 266 | run this mode by default? |
| `MaxReceivers` | 266 | upper bound of receivers used in array processing |
| `NumberOfDepthsToStack` | 266 | depth stacking |
| `TheReceiverMode` | 266 | acquisition mode enum |
| `FilterLow` | 266 | bandpass low corner — **unit not stated, see gap G-2** |
| `FilterHigh` | 266 | bandpass high corner — **unit not stated, see gap G-2** |
| `Npts` | 266 | semblance scan points |
| `MinSlow` | 266 | slowness search lower bound |
| `SlowStep` | 266 | slowness increment |
| `MaxSlow` | 266 | slowness search upper bound |
| `Units` | 266 | **declares the slowness unit** |
| `Early` | 266 | processing time-gate start |
| `Late` | 266 | processing time-gate end |
| `TimeStep` | 266 | time-axis increment |
| `Window` | 266 | semblance/correlation window length |
| `FindPeaks` | 266 | peak picking on/off |
| `VDLMode` | 266 | VDL display mode |
| `SemblanceCutoff` | 266 | coherence threshold — **scale ambiguous, see gap G-1** |
| `TheReceiverGroup` | 204 | newer schema only |
| `VerticalResolution` | 62 | older schema only |
| `FrequencyVDL` | 62 | older schema only |
| `FrequencySemblanceVDL` | 62 | older schema only |

`<ModeSettings>` attributes and child lists:

| attribute / list | present | meaning |
|---|---|---|
| `ModeName` | 266 | join key |
| `ReceiverCount` | 266 | receivers in the array for this mode |
| `DepthShift` | 266 | tool-to-measure-point depth offset |
| `SampleRate` | 266 | waveform sample rate |
| `InputCurveMask` | 266 | **wildcard pattern** binding vendor waveform arrays to this mode |
| `RxTxOrientation` | 198 | `Rx above Tx` (197) or `Tx above Rx` (1) — **newer schema only** |
| `CoreMode` | 62 | **older schema only** |
| `DistanceFromTransmitterToReceiver` (list) | per mode | transmitter→receiver spacings, one per receiver |
| `TimeShift` (list) | per mode | per-receiver time shift |
| `GainShift` (list) | per mode | per-receiver gain curve mnemonic (or a literal) |

*Example of the geometry shape* — `AcousticWaveforms\Tools\DSI.att`, mode `MP`: `ReceiverCount="8"`,
`RxTxOrientation="Rx above Tx"`, `DepthShift="52"`, `SampleRate="10"`, `InputCurveMask="PWF4_*"`,
`GainShift` = `PWN4`, and eight `DistanceFromTransmitterToReceiver` values running 9 → 12.5 in
0.5 steps. **This spacing table is vendor tool geometry — recorded here as a shape example only, not
adopted, and not to be copied into SandiBumi.**

### 3.3 `.itp` — pad definition (XML)

Root `<Pad xsi:type="DefinedPad" ID=… Name=…>` in the same ImageTool namespace. Comment header
states verbatim that `ButtonWidth`, `HorizontalOffset`, `HorizontalStep`, `VerticalOffset` and
`VerticalStep` **are all in inches**, and that `ButtonRow/SourceCurve` **"is an index (e.g. [1]) to
the list of SourceCurves in the tool definition's ReferencedPad."**

Body is `ButtonCollections > ButtonCollection`. Every one of the 96 collections observed is
`xsi:type="ButtonRow"` — no other collection type ships.

A row is generated **procedurally**, not listed: `Count` buttons starting at
(`HorizontalOffset`, `VerticalOffset`), each stepping by (`HorizontalStep`, `VerticalStep`), with the
curve-sample index starting at `StartIndex` and advancing by `IndexStep`. An interlaced two-row pad
is therefore two `ButtonCollection` elements with the same `Count` and a `VerticalOffset` difference
— typically `0` and `0.3`.

| attribute | distinct values | most common | note |
|---|---|---|---|
| `type` | 1 | `ButtonRow` (96 of 96) | always ButtonRow |
| `Count` | 10 | `12` (34 of 96) | buttons in the row |
| `ButtonWidth` | 15 | `0.1` (58 of 96) | **inches** |
| `StartIndex` | 3 | `1` (78 of 96) | first curve-sample index |
| `IndexStep` | 2 | `1` (68 of 96) | index increment — `2` means interlaced |
| `HorizontalOffset` | 35 | `0` (14 of 96) | **inches** |
| `HorizontalStep` | 17 | `-0.2` (35 of 96) | **inches**, sign = rotation sense |
| `VerticalOffset` | 8 | `0` (62 of 96) | **inches** |
| `VerticalStep` | 2 | `0` (94 of 96) | **inches** |
| `DataNormalisationOffset` | 1 | `0` (96 of 96) | linear rescale offset |
| `DataNormalisationScale` | 1 | `1` (96 of 96) | linear rescale gain |
| `Hidden` | 1 | `true` (2 of 2) | only 2 collections |

`SourceCurve` index-reference forms observed: `[1]` (×74), `[2]` (×19), `[3]` (×2), `[4]` (×1).

### 3.4 `.bor` — button coordinate table (plain text)

Whitespace-delimited, **no fixed column widths**, no header keywords.

- Lines beginning with `$` are free-text comments and carry the **only** documentation of
  sense-of-rotation and depth reference.
- Data lines are **one row per button**, in the order the buttons appear in the source curve array.
- **17 of 18 files are 2-column**: col 1 = horizontal/circumferential offset, col 2 = vertical
  (depth) offset. `Weatherford OMI.bor` is **1-column** (horizontal only).
- **Units are inches**, stated only inside the `$` comments — e.g. `"25 buttons spacing is 0.2 inch"`
  (`Weatherford HMI upper.bor`), `"5 buttons spacing is 0.4 inch"` (`OBMI_button_order.bor`).
- Row count equals the button count declared for that pad in the companion `.eli`.

The `$` comments are also where the **depth-reference convention** is stated. `STAR_Pad135.bor`
records that button 2 is the depth reference and that alternate buttons sit 0.3 inch deeper —
an interlaced-row convention that the `.itp` format later encodes structurally as two `ButtonRow`
collections with a `VerticalOffset` difference. Same physical fact, two encodings.

### 3.5 `.eli` — electrical-image parameter file (plain text)

Three parts, identical in all 16 files:

1. **Banner** — line 1 is the literal `Electrical Image Parameter file` plus a free-form timestamp.
   Timestamp formats are inconsistent (`25/11/02 15:39:09`, `03/27/2009 15:33:39`,
   `1/6/2003 4:27:56 PM`, and one file with no date at all).
2. **Pad table** — a `$`-commented two-line legend, then **one row per pad/curve**, 7 whitespace-
   delimited fields:

   | col | legend (verbatim) | meaning | units |
   |---|---|---|---|
   | 1 | `Name Curve` | LAS mnemonic of the pad's button array | — |
   | 2 | `Number Buttons` | button count on that pad | count |
   | 3 | `Button Spacing` | inter-button spacing | inches *(per `.bor` comments — not stated in `.eli`)* |
   | 4 | `Orientation to Azimuth` | pad angular position from pad 1 | degrees |
   | 5 | `Depth Shift` | pad depth offset | inches |
   | 6 | `Rotation Shift` | azimuthal rotation correction | degrees |
   | 7 | `Button order file name` | companion `.bor` filename — **optional**, blank in 6 files | — |

3. **Trailer** — nine fixed `$`-keyed directives, each followed by its value line, present in
   **16/16 files in exactly the same order**:

   | # | directive (verbatim) |
   |---|---|
   | 1 | `$Tool Resolution arround borehole` *(sic — vendor's typo)* |
   | 2 | `$Default Output image curve name` |
   | 3 | `$Default Input Low High / Output Low high Low Values` |
   | 4 | `$Default Clip output to input low high value` |
   | 5 | `$Default Normalize image / normalization window` |
   | 6 | `$Default Hole size` |
   | 7 | `$Default Azimuth input curve` |
   | 8 | `$Default Correct azimuth for magnetic deviation / deviation value` |
   | 9 | `$Default top / bottom depths` |

`-` is the null/unset token (used in the input/output range and the top/bottom depth lines).

---

## 4. Harvested parameter defaults

### 4.1 Truly universal — identical in every file that carries the attribute

These are real defaults, not tool characteristics. Safe to treat as *the vendor's shipped default*,
**with this source cited**.

| parameter | value | units | scope | source |
|---|---|---|---|---|
| `Units` | `us/ft` | (enumeration) | UNIVERSAL - 266/266 mode blocks, across all 60/60 .att files carrying this attribute | 60 files, e.g. AZIBAT.att |
| `TheReceiverMode` | `rmSingleAcq` | (enumeration) | UNIVERSAL - 266/266 mode blocks, across all 60/60 .att files carrying this attribute | 60 files, e.g. AZIBAT.att |
| `TheReceiverGroup` | `rgFirst` | (enumeration) | UNIVERSAL - 204/204 mode blocks, across all 37/60 .att files carrying this attribute | 37 files, e.g. AcousticWaveforms\Tools\AHW-MAS.att |
| `NumberOfDepthsToStack` | `0` | count | UNIVERSAL - 266/266 mode blocks, across all 60/60 .att files carrying this attribute | 60 files, e.g. AZIBAT.att |
| `FindPeaks` | `false` | boolean | UNIVERSAL - 266/266 mode blocks, across all 60/60 .att files carrying this attribute | 60 files, e.g. AZIBAT.att |
| `TimeUnits` | `µs` | (literal) | UNIVERSAL across the 37 files carrying this attribute | 37 files, e.g. AcousticWaveforms\Tools\AHW-MAS.att |
| `AnisoMethod` | `Alford (Time)` | (enumeration) | UNIVERSAL across the 36 files carrying this attribute | 36 files, e.g. AcousticWaveforms\Tools\AHW-MAS.att |
| `SembMethod` | `Semblance` | (enumeration) | UNIVERSAL across the 36 files carrying this attribute | 36 files, e.g. AcousticWaveforms\Tools\AHW-MAS.att |
| `NptsToSmooth` | `3` | count | UNIVERSAL across the 37 files carrying this attribute | 37 files, e.g. AcousticWaveforms\Tools\AHW-MAS.att |
| `DataNormalisationOffset` | `0` | dimensionless | UNIVERSAL - 96/96 ButtonRow collections in all 58 .itp files | all .itp files, e.g. Schlumberger FMI.itp |
| `DataNormalisationScale` | `1` | dimensionless | UNIVERSAL - 96/96 ButtonRow collections in all 58 .itp files | all .itp files, e.g. Schlumberger FMI.itp |

### 4.2 Near-universal — one single outlier

| parameter | dominant value | outlier | outlier file |
|---|---|---|---|
| `MinSlow` | `40` µs/ft — 265/266 mode blocks | `30` | `AcousticWaveforms\Tools\DSLT.att` |
| `SlowStep` | `2` µs/ft — 265/266 mode blocks | `1` | `AcousticWaveforms\Tools\DSLT.att` |

**A single file — `DSLT.att` — is the only deviation in the entire set** for both. It also carries the
lowest `MaxSlow` (`140`). So the effective shipped default slowness search is
**40 → (240 or 440) µs/ft in 2 µs/ft steps**, and `DSLT` is the one tool given a tighter, finer scan.

### 4.3 Tool-specific — genuine per-tool values, NOT defaults

`MaxSlow` is the clearest case of a parameter that *looks* like a default but is not:

| `MaxSlow` (µs/ft) | mode blocks | example source |
|---|---|---|
| `240` | 132 | `54 files, e.g. AZIBAT.att` |
| `440` | 125 | `21 files, e.g. AcousticWaveforms\Tools\AZIBAT_BAT_QBAT.att` |
| `340` | 8 | `7 files, e.g. AcousticWaveforms\Tools\AHW-MAS.att` |
| `140` | 1 | `AcousticWaveforms\Tools\DSLT.att` |

`240` dominates the older root-schema files; `440` dominates the newer `Tools\` files — i.e. the
newer generation widened the shear/Stoneley search. **Do not adopt a single number here.**

Other genuinely tool-specific parameters, with their observed ranges — reported as ranges precisely
because *no* value in these rows is a defensible default:

| parameter | distinct values | range / dominant | units |
|---|---|---|---|
| `FilterLow` | 13 | 1 → 15 (most common `1`, 141 blocks) | **not stated — see gap G-2** |
| `FilterHigh` | 16 | 2.5 → 25 (most common `20`, 61 blocks) | **not stated — see gap G-2** |
| `Window` | 8 | 100 → 1000 (most common `250`, 124 blocks) | µs |
| `Early` | 11 | -1500 → 2000 (most common `0`, 74 blocks) | µs |
| `Late` | 13 | 100 → 10500 (most common `1000`, 143 blocks) | µs |
| `TimeStep` | 7 | 10 → 200 (most common `100`, 177 blocks) | µs |
| `Npts` | 4 | 30 → 200 (most common `35`, 136 blocks) | count |
| `MaxReceivers` | 9 | 4 → 13 (most common `8`, 112 blocks) | count |

### 4.4 `.itt` imaging-side values

`DepthOfInvestigationValue` is the only imaging parameter shipped with a real spread. Reported in
full because each is a *tool characteristic* with its filename — none is a default:

| value | Units attribute | files | example source |
|---|---|---|---|
| `0` | Inches | 18 | `Baker HDIP.itt` |
| `0.5` | Inches | 13 | `Baker EARTH.itt` |
| `*(empty — not present in file)*` | Feet | 10 | `Acoustic (Generic) AMP.itt` |
| `*(empty — not present in file)*` | Inches | 7 | `Image360.itt` |
| `0.25` | Inches | 5 | `Weatherford CMI 4.1.itt` |
| `0.9` | Inches | 4 | `Halliburton EMI.itt` |
| `0.2` | Inches | 3 | `Schlumberger NGI Lower.itt` |
| `0.35` | Inches | 2 | `Baker GeoXplorer  - Techlog Export.itt` |
| `0.0` | Inches | 2 | `Halliburton FIAC.itt` |
| `1.5` | Inches | 2 | `Schlumberger MicroScope HD - Techlog Export.itt` |
| `1.75` | Inches | 1 | `Halliburton ALD.itt` |
| `3.0` | Inches | 1 | `Halliburton OMRI.itt` |
| `0.39` | Inches | 1 | `Schlumberger FMI-HD.itt` |
| `1.8` | Inches | 1 | `Weatherford AZD.itt` |
| `1.0` | Inches | 1 | `Weatherford CMI 2.4.itt` |
| `0.8` | Inches | 1 | `Weatherford SineWave.itt` |

`DepthOfInvestigationCurve` ships in 68/72 files and is **empty in all 68** — the curve-driven
alternative is declared but never populated.

Other `.itt` scalars:

| parameter | values observed | units | note |
|---|---|---|---|
| `AccelerometerOffset` | `16` (14 files), `0`/`0.0` (9), `37.8` (1) | Inches | distance accelerometer→measure point, used for speed correction |
| `DelayValue` | `0` (8), `22.7` (2) | Microseconds | acoustic electronic delay; `22.7` is both Baker CBIL files |
| `FluidSlownessValue` | `0` (10) | MicrosecondsPerFoot | **always 0 — a placeholder, not a default**; the paired `FluidSlownessCurve` carries the real input |
| `SectorOffset` | `-135` (2) | Degrees | both Schlumberger MicroScope HD files |
| `BitSizeValue` | `8.5` (1) | Inches | `LWD (Generic).itt` only |
| `RenderStyle` | `Interpolated` (14), `Normal` (1) | enum | `Normal` only on `Image360.itt` |
| `HoleSizeMethod` | `CaliperOrRadius` (8), `BitSize` (7) | enum | |
| `ToolAlignment` | `Highside` (15) | enum | **universal across every LWD/360/MIT file** |
| `ToolOrientation` | `North` (4) | enum | all four Halliburton CAST files |
| `DivideTravelTimeByConstant` | `true` (6) | boolean | Baker CBIL ×2, Halliburton CAST ×4 |

---

## 5. Curve-mnemonic conventions

This is the section most directly reusable as an alias-catalogue contribution.

### 5.1 The mechanism — a priority-ordered alias list

Every `*Curve` element holds **either a single mnemonic or a `;`-separated priority-ordered list**.
IP tries each candidate in turn. Examples verbatim from `Schlumberger FMI.itt`:
`<Pad1AzimuthCurve>P1NO;P1AZ</Pad1AzimuthCurve>`, and from the Baker GeoXplorer family:
`<HoleDeviationAngleCurve>DEVGX;DEVGX.H</HoleDeviationAngleCurve>`.

Two suffix conventions appear inside those lists:

- **`.H`** — a vendor/export variant of the same channel (`DEVCB;DEVCB.H`, `ETGX;ETGX.H`,
  `RMX;RMXGCB.H`). Seen across the Baker CBIL / GeoXplorer / StarTrak export definitions.
- **`_S`** — a second export variant (`DEVGX;DEVGX_S`, `AZGX;AZGX_S`).

**This is the single most transferable idea in the target**: a role resolves to an *ordered list*,
not to one name.

### 5.2 Role → mnemonic map (`.itt`)

Every value below is verbatim from a file; `files` is how many `.itt` files carry it.

| role | meaning | mnemonics observed (`value` ×files) |
|---|---|---|
| `HoleDeviationAngleCurve` | borehole inclination | `DEVI` ×37, `DEV` ×8, `ITLT` ×6, `DEVI;DEVI2` ×4, `DEVIM` ×3, `DEVCB;DEVCB.H` ×2, `DEVEI` ×1, `DEVGX;DEVGX_S` ×1, `DEVGX;DEVGX.H` ×1, `DEV;DEVST.H` ×1, `SINCL` ×1, `DEVW` ×1, `HDEVI` ×1, `SDEV;DEVI` ×1, `SDEV` ×1, `MITDEV` ×1, `INC` ×1, `4DVI` ×1 |
| `HoleDeviationAzimuthCurve` | borehole azimuth | `HAZI` ×47, `IAZI` ×6, `DAZ` ×4, `AZI1` ×3, `DAZCB;DAZCB.H` ×2, `DAZEI` ×1, `DAZGX;DAZGX_S` ×1, `DAZGX;DAZGX.H` ×1, `DAZ;DAZST.H` ×1, `SAZIMC` ×1, `DAZW` ×1, `MITROT` ×1, `AZM` ×1, `UHAZ` ×1, `4UHA` ×1 |
| `Pad1AzimuthCurve` | azimuth of pad 1 | `P1NO;P1AZ` ×13, `AZI1` ×8, `P1AZ` ×7, `IAP1` ×5, `AZ` ×4, `HAZI` ×3, `P1NO_NGI_UPPER` ×3, `AZCB;AZCB.H` ×2, `P1NO_OBMT_2;P1AZ_OBMT_2` ×2, `AZEI` ×1, `AZGX;AZGX_S` ×1, `AZGX;AZGX.H` ×1, `AZ;AZST.H` ×1, `AZW` ×1, `P1NO_OBMT;P1AZ_OBMT` ×1, `P1NO_FBST;P1NO;P1AZ` ×1, `IAAZ` ×1, `UAZI` ×1, `4UAZ` ×1 |
| `RelativeBearingCurve` | relative bearing | `RB` ×36, `IRHS` ×4, `RB_NGI_UPPER` ×3, `RBCB;RBCB.H` ×2, `RB_OBMT_2` ×2, `RB_OBMT` ×2, `IRBR` ×2, `RBEI` ×1, `RBGX;RBGX_S` ×1, `RBGX;RBGX.H` ×1, `RB;RBST.H` ×1, `RBW` ×1, `4RB` ×1 |
| `GammaRayCurve` | gamma ray | `GR` ×38, `GRGC` ×7 |
| `SurfaceVelocityCurve` | cable / logging speed | `CVEL` ×19, `SPD` ×8, `MSPD` ×7, `LPSD;CS` ×6, `LSPD;CS` ×1, `CS` ×1 |
| `XAccelerometerCurve` | accelerometer X | `AX` ×19, `ACCX` ×10, `IMAX` ×7, `GAX` ×5, `GAX;RAXGCB.H` ×2, `GAX;RAXGGX.H` ×1, `GAX;RAXGST.H` ×1 |
| `YAccelerometerCurve` | accelerometer Y | `AY` ×19, `ACCY` ×10, `IMAY` ×7, `GAY` ×5, `GAY;RAYGCB.H` ×2, `GAY;RAYGGX.H` ×1, `GAY;RAYGST.H` ×1 |
| `ZAccelerometerCurve` | accelerometer Z | `AZ` ×19, `ACCZ` ×10, `IMZA` ×7, `GAZ` ×5, `GAZ;RAZGCB.H` ×2, `GAZ;RAZGGX.H` ×1, `GAZ;RAZGST.H` ×1 |
| `ZAccelerationCurve` | Z acceleration (speed correction input) | `FCAZ` ×21, `ZACC` ×11, `IMZA` ×7, `GAZ` ×4, `GAZ;RAZGCB.H` ×2, `GAZ;RAZGGX.H` ×1, `GAZ;RAZGST.H` ×1, `sec` ×1, `RAZ` ×1 |
| `XMagnetometerCurve` | magnetometer X | `FX` ×19, `MAGX` ×10, `IEMX` ×7, `RMX` ×5, `RMX;RMXGCB.H` ×2, `RMX;RMXGGX.H` ×1, `RMX;RMXGST.H` ×1 |
| `YMagnetometerCurve` | magnetometer Y | `FY` ×19, `MAGY` ×10, `IEMY` ×7, `RMY` ×5, `RMY;RMYGCB.H` ×2, `RMY;RMYGGX.H` ×1, `RMY;RMYGST.H` ×1 |
| `ZMagnetometerCurve` | magnetometer Z | `FZ` ×19, `MagZ` ×9, `IEMZ` ×7, `RMZ` ×5, `RMZ;RMZGCB.H` ×2, `RMZ;RMZGGX.H` ×1, `RMZ;RMZGST.H` ×1, `MAGZ` ×1 |
| `IntervalTimeCurve` | per-sample interval time | `FTIM` ×21, `IEDZ` ×7, `ETCB;ETCB.H` ×2, `ETEI` ×1, `ETGX` ×1, `ETGX;ETGX.H` ×1, `ETST;ETST.H` ×1, `TEN` ×1, `ETIMD` ×1 |
| `CumulativeTimeCurve` | elapsed time | `DXTM` ×11, `ETIM` ×5, `ETIMD` ×1, `ETMD` ×1 |
| `EmexCurrentCurve` | EMEX current (button current normalisation) | `EI` ×10, `CGVT` ×7, `PADG` ×3, `PD6G` ×1 |
| `CaliperCurve` | caliper | `CALI` ×3, `SCAL` ×2, `CALCM5` ×1, `UCAV` ×1, `VERD` ×1, `BS` ×1 |
| `RadiusCurve` | radius (alternative to caliper) | `PRAD;RADI` ×2, `RADI` ×2 |
| `TravelTimeCurve` | ultrasonic travel time | `TTBK` ×4, `TT` ×4, `BHTT;BHTT.H;BHTT5.H` ×2 |
| `FluidSlownessCurve` | borehole fluid slowness | `CFVL` ×4, `FTT` ×4, `SFLD;SFLD.H` ×2 |
| `SpeedButtonCurve` | dipmeter speed button | `SB1` ×1, `SB2` ×1 |
| `SourceCurve` | the image data channel itself | `AWBK` ×2, `TTBK` ×2, `TT` ×2, `BHTA;BHTA.H;BHTA5.H` ×1, `BHTT;BHTT.H;BHTT5.H` ×1, `SRAMDF` ×1, `@AFRImages` ×1, `HD` ×1, `PAMP;AMP` ×1, `AMP` ×1, `IRHOB` ×1, `INBGR` ×1, `ROSI` ×1, `RES_BS_IMG` ×1, `UHRI_IMG` ×1, `RES_BS_IMG;RES_BM_IMG;RES_BD_IMG` ×1, `RHOZC` ×1, `IB0Z` ×1 |

**Declared but always empty** (no default mnemonic exists for these roles): `DepthOfInvestigationCurve` (in 68 files), `MagneticDeclinationCurve` (in 16 files), `BitSizeCurve` (in 1 files).

### 5.3 Acoustic output-curve conventions (`.att`)

`PrimaryOutputCurves` and `SecondaryOutputCurves` are `;`-separated lists. The naming rule is
**`DT` + mode token**:

| mnemonic | files | role |
|---|---|---|
| `DTC` | 44 | compressional slowness |
| `DTS_MP` | 36 | shear from monopole |
| `DTST` | 33 | Stoneley slowness |
| `DTXX` | 21 | cross-dipole XX component |
| `DTXY` | 18 | cross-dipole XY |
| `DTYX` | 18 | cross-dipole YX |
| `DTYY` | 18 | cross-dipole YY |
| `DTFast` | 14 | fast shear slowness |
| `DTSlow` | 14 | slow shear slowness |
| `AnisoAngle` | 14 | anisotropy angle |
| `Aniso` | 13 | anisotropy magnitude |
| `FastAzi` | 13 | fast-shear azimuth |
| `DTRS` | 8 | refracted shear |
| `DTSTM` | 3 | Stoneley, monopole |
| `DTQuad` | 3 | quadrupole slowness |
| `DTSTX` | 2 | Stoneley X |
| `DTSTY` | 2 | Stoneley Y |
| `DTFlex` | 2 | flexural slowness |

Secondary (derived / QC) outputs follow two suffix rules — **`_Smooth`** = smoothed twin of a
primary, **`Aniso*Map`** = anisotropy diagnostic raster:

| mnemonic | files | role |
|---|---|---|
| `VPVS` | 42 | Vp/Vs ratio |
| `DTS` | 38 | shear slowness |
| `TT` | 37 | travel time |
| `AnisoMap` | 18 | anisotropy raster |
| `DTS_XX` | 17 | shear from XX |
| `DTS_ST` | 16 | shear from Stoneley |
| `DTS_YY` | 14 |  |
| `DTS_XY` | 14 |  |
| `DTS_YX` | 14 |  |
| `DTS_Fast` | 14 |  |
| `DTS_Slow` | 14 |  |
| `AnisoDeltaMap` | 14 | anisotropy delta raster |
| `AnisoInlineMap` | 14 | inline energy raster |
| `AnisoCrosslineMap` | 14 | crossline energy raster |
| `AnisoEnergyRatioMap` | 14 | energy-ratio raster |
| `AnisoEnergyMin` | 14 | minimum energy |

### 5.4 Input binding is by wildcard, not by name (`.att`)

`ModeSettings/@InputCurveMask` is a **pattern**, not a mnemonic: `*` matches the receiver index, `;`
separates alternative patterns tried in order, and `,` (rare) lists explicit mnemonics. Examples
verbatim: `PWF4_*` (`Tools\DSI.att`), `PWF2_*;PWFX_1_*` (same file), `FastWaveform_*` /
`SlowWaveform_*` (13 files each), `M*AP;MP*` (4 files),
`WF11,WF12,WF13,WF14,WF15,WF21,WF22,WF23,WF24,WF25` (one file, the explicit-list form).

This is how IP binds an arbitrary vendor waveform array to a processing mode without knowing the
receiver count in advance. **The pattern mechanism is worth adopting; the individual masks are
vendor channel names.**

### 5.5 Legacy pad-array names (`.eli` column 1)

| mnemonic | files | tool |
|---|---|---|
| `PAD1` | 4 | Halliburton EMI |
| `PAD2` | 4 | Halliburton EMI |
| `PAD3` | 4 | Halliburton EMI |
| `PAD4` | 4 | Halliburton EMI |
| `PAD5` | 4 | Halliburton EMI |
| `PAD6` | 4 | Halliburton EMI |
| `BA15` | 2 | FMS-C 1 curve per button |
| `BB15` | 2 | FMS-C 1 curve per button |
| `BC15` | 2 | FMS-C 1 curve per button |
| `BD15` | 2 | FMS-C 1 curve per button |
| `Pad1` | 1 | Create image from 4 arm dipmeter curves |
| `Pad2` | 1 | Create image from 4 arm dipmeter curves |
| `Pad3` | 1 | Create image from 4 arm dipmeter curves |
| `Pad4` | 1 | Create image from 4 arm dipmeter curves |
| `ROSI` | 1 | Anadrill rotation density image ROSI |
| `P1OBI` | 1 | Baker Atlas Earth Imager |
| `P2OBI` | 1 | Baker Atlas Earth Imager |
| `P3OBI` | 1 | Baker Atlas Earth Imager |
| `P4OBI` | 1 | Baker Atlas Earth Imager |
| `P5OBI` | 1 | Baker Atlas Earth Imager |
| `P6OBI` | 1 | Baker Atlas Earth Imager |
| `P1BTN` | 1 | Baker Atlas STAR (II and III) |
| `P2BTN` | 1 | Baker Atlas STAR (II and III) |
| `P3BTN` | 1 | Baker Atlas STAR (II and III) |

Convention: a **pad token plus an index** — `PAD1`…`PAD6`, `P1BTN`/`P2BTN`, `FCA1`…`FCD4`,
`OBRA`/`OBRB`, `XPAD1`…`XPAD6`, `BT1U`/`BT1L` (upper/lower row).

---

## 6. What SandiBumi can legitimately reuse

Stated plainly, and conservatively. The distinction that matters is **adopt the convention**
(a design idea, a unit rule, a taxonomy — free) versus **copy the data** (the vendor's compiled
per-tool geometry — not free).

### 6.1 Tool-class taxonomy (PadBasedTool / LWDTool / AcousticTool / DipmeterTool / CaliperTool / MITTool / Image360Tool / Scan360Tool)

✅ **ADOPT THE CONVENTION**

Model SandiBumi's borehole-image tool registry on this 8-class split. It is a genuinely good decomposition: the class determines which fields are even meaningful (pad tools need arms+pads, LWD tools need SectorOffset+ToolAlignment, acoustic tools need DelayValue+FluidSlowness). Copy the IDEA and the field-set-per-class, write our own class names and our own schema.

*IP risk*: none - a taxonomy is not expression

### 6.2 Unit declarations carried in the comment headers (Diameter/offsets/ButtonWidth = inches; angles = degrees; DelayValue = Microseconds; FluidSlowness = MicrosecondsPerFoot; DepthOfInvestigation Units enum)

✅ **ADOPT THE CONVENTION**

Make units EXPLICIT-PER-FIELD in SandiBumi's tool schema, exactly as IP does with the Units attribute on DepthOfInvestigationValue. IP's own inconsistency (units in a free-text comment for .bor, in an attribute for .itt) is the anti-pattern to avoid: always an attribute.

*IP risk*: none - unit conventions are facts

### 6.3 The ';'-separated priority-ordered alias list on every *Curve element

✅ **ADOPT THE CONVENTION**

SandiBumi's alias catalogue should resolve a ROLE (hole-deviation angle, pad-1 azimuth, relative bearing, Z accelerometer...) to an ORDERED list of candidate mnemonics, not a single name. This is the single most directly transferable idea in the whole target.

*IP risk*: none - the mechanism is a convention

### 6.4 The observed role->mnemonic mapping itself (DEVI/DEV/INC for deviation, HAZI/DAZ/AZI1 for hole azimuth, P1AZ/P1NO for pad-1 azimuth, RB for relative bearing, GR, CVEL/SPD/MSPD for cable speed, AX/AY/AZ + FX/FY/FZ for accelerometer/magnetometer triads)

⚠️ **ADOPT AS EVIDENCE, RE-DERIVE**

These mnemonics are INDUSTRY names (they originate with the service companies, not with PGL). Use this target as EVIDENCE that a mnemonic is real and what role it fills, then cross-check each one against the Techlog family/alias catalogs and the IP2025 register already ingested, and enter it in SandiBumi's own catalogue with that cross-source citation. Do NOT bulk-import IP's lists as a file.

*IP risk*: low - individual mnemonics are industry facts; the curated LIST as a compiled work is not ours

### 6.5 Acoustic slowness-processing parameter SET (MinSlow/MaxSlow/SlowStep/Units/FilterLow/FilterHigh/Window/Early/Late/TimeStep/Npts/MaxReceivers/SemblanceCutoff)

⚠️ **ADOPT THE SET, RE-DERIVE THE VALUES**

The parameter set is the correct decomposition of an STC (slowness-time-coherence) job and SandiBumi should expose the same knobs. The universal values (MinSlow 40, SlowStep 2, Units us/ft) are defensible as an initial UI default WITH THIS FILE CITED. The per-tool values (TR spacings, DepthShift, SampleRate) are vendor tool geometry and must come from the tool's own datasheet, not from here.

*IP risk*: low for the parameter set; the per-tool geometry table is vendor data - do not redistribute

### 6.6 Per-tool button geometry (.itp ButtonRow offsets, .bor coordinate tables, .eli pad tables)

🛑 **DO NOT COPY**

This is the vendor's measured tool geometry, compiled by PGL, shipped under EULA. SandiBumi must let a USER import or enter their own tool geometry, and should ship at most a GENERIC parametric pad (count / spacing / row offset) that a user populates. Record here that the procedural ButtonRow encoding (Count + Offset + Step + StartIndex + IndexStep) is an elegant way to express an interlaced two-row pad in 2 lines - adopt THAT encoding, empty.

*IP risk*: HIGH - this is the redistributable-data core of the target

### 6.7 The ButtonRow procedural encoding and the SourceCurve '[n]' indirection

✅ **ADOPT THE CONVENTION**

Expressing a pad as (Count, ButtonWidth, StartIndex, IndexStep, H/V Offset, H/V Step) rather than an explicit coordinate list, plus an index reference into the parent tool's curve list, makes a pad definition ~2 lines and lets one pad file serve many tools. Worth copying as a design decision for SandiBumi's own pad schema.

*IP risk*: none - a data-modelling idea

### 6.8 The 'is this already corrected?' boolean family (ButtonsAligned / PadsAligned / SpeedCorrected / SwingArmCorrected / MagneticDeclinationApplied / CenterReProjectionNotRequired / StagesAligned / NavigationAligned)

✅ **ADOPT — HIGH VALUE**

This is a provenance contract: the tool definition declares WHICH corrections the incoming data has ALREADY had applied, so the software does not double-correct. Double-applying a speed correction or a pad-offset alignment is exactly the silent-wrongness failure SandiBumi's data-integrity discipline exists to prevent. Adopt this flag family into SandiBumi's image ingest contract.

*IP risk*: none - a design pattern

### 6.9 Two-generation .att schema coexistence

⚠️ **ADOPT AS A WARNING**

IP2018 ships 23 old-schema and 37 new-schema .att files in different directories with the same extension and the same root element, differing in element names and in the MEANING of SemblanceCutoff. SandiBumi should version its tool-definition schema explicitly (a schemaVersion attribute) and refuse to load an unversioned file.

*IP risk*: none

### 6.10 The two-sentence recommendation

**Adopt the *conventions* wholesale and the *data* not at all**: the eight-class tool taxonomy, the
explicit per-field unit declarations, the `;`-separated priority-ordered mnemonic alias list, the
procedural `ButtonRow` pad encoding, and above all the *"which corrections has this data already
had?"* boolean family (`ButtonsAligned` / `PadsAligned` / `SpeedCorrected` / `SwingArmCorrected` /
`MagneticDeclinationApplied`) are design ideas that cost nothing to reuse and directly serve
SandiBumi's data-integrity discipline. The per-tool button geometry, transmitter–receiver spacing
tables and curated mnemonic lists are PGL's compiled vendor data under EULA — SandiBumi should ship
a *generic parametric* pad and acoustic-tool definition that a user populates from their own tool
datasheet, and should re-derive any mnemonic it adopts against the Techlog family/alias catalogs and
the already-ingested IP2025 register rather than importing these lists.

---

## 7. Traps worth carrying into the SandiBumi importer

Three findings here are silent-wrongness risks, not just curiosities.

1. **`SemblanceCutoff` ships in two scales, 100× apart, under one attribute name** (gap G-1). A
   value of `25` and a value of `.25` both appear, and one file contains both. Any importer that
   reads this attribute without deciding the scale will either disable the coherence gate or reject
   everything — and it will *compute and plot* either way.
2. **`FilterLow`/`FilterHigh` carry no unit anywhere in the schema** (gap G-2). kHz is the only
   physically sensible reading at these magnitudes, but that is inference. Recorded as inference.
3. **The vendor's own reference data contains defects** (gap G-5): a unit string sitting in a curve-
   mnemonic field, two different tools sharing one ID, and a pad file with no vendor prefix. This is
   the concrete argument for validating any imported tool definition rather than trusting it.

---

## 8. Tier-C flags

- **NO Tier-C functionality is referenced by any of the 224 files in this target.**
  - *Evidence*: Case-insensitive grep across all .itt/.att/.itp/.bor/.eli for 'entropy', 'SonicSaturation', 'Domain Transfer', 'Experienced Eye', 'neural', 'patent', 'proprietary' returned ZERO hits.

- **Entropy-based borehole-image speed correction (registered Tier-C) is NOT named here, but the schema reserves a slot adjacent to it.**
  - *Evidence*: The .itt comment header states verbatim: 'SpeedCorrected - when true implies that data for this tool has already been speed corrected.' The <SpeedCorrected> ELEMENT is documented in 15+25+3+4 = 47 comment headers but is SET IN ZERO SHIPPED FILES (grep '<SpeedCorrected>' over all 72 .itt returns nothing). Likewise <SwingArmCorrected>: documented, never set. So the speed-correction ALGORITHM lives in the binary, not in these files; only the already-corrected FLAG is part of the data contract.
  - *Tier*: Tier-A (the boolean flag/provenance contract) is free to adopt; the entropy speed-correction ALGORITHM remains Tier-C name-only and is not disclosed by this target.
  - *Action*: Adopt the flag, do not pursue the algorithm.

- **Anisotropy processing names an in-file method, but it is a published-literature method, not a PGL brand.**
  - *Evidence*: AnisoMethod="Alford (Time)" in 36/37 newer .att files; SembMethod="Semblance" in 36/37. Alford rotation is standard published cross-dipole processing - Tier A/B, not Tier C.
  - *Tier*: Tier A (method name is public)

**Summary: no Tier-C functionality is disclosed by this target.** A case-insensitive sweep of all
224 files for `entropy`, `SonicSaturation`, `Domain Transfer`, `Experienced Eye`, `neural`, `patent`
and `proprietary` returned **zero hits**.

The one boundary worth stating precisely: the registered Tier-C item *entropy-based borehole-image
speed correction* is **not named here**, but the `.itt` comment header documents a neighbouring
flag — `"SpeedCorrected - when true implies that data for this tool has already been speed
corrected."` The `<SpeedCorrected>` **element is documented in 47 comment headers and set in zero
shipped files** (`grep '<SpeedCorrected>'` over all 72 `.itt` returns nothing); the same is true of
`<SwingArmCorrected>`. So the speed-correction *algorithm* lives in the binary, not in these files.
**The boolean provenance flag is Tier A and free to adopt; the algorithm remains Tier-C name-only
and is not pursued.**

`AnisoMethod="Alford (Time)"` (36/37 newer files) and `SembMethod="Semblance"` (36/37) name
published-literature methods, not PGL brands — Tier A.

---

## 9. Gaps and unresolved items

### G-1 — SemblanceCutoff is SHIPPED IN TWO INCOMPATIBLE SCALES and the file never declares which.

Value '.25' or '0.25' appears in 198 mode blocks; value '25' appears in 56. Same attribute, same schema, 100x apart. The split correlates with schema generation (newer AcousticWaveforms\Tools\ files mostly '.25'; older root files mostly '25') but is NOT clean - Tools\CLSS.att, Tools\Esonic.att, Tools\FWS.att use '25' while root\Generic.att and root\XMAC.att use '.25', and Tools\Scope.att contains BOTH in one file.

- *Impact*: SandiBumi must NOT adopt a numeric semblance-cutoff default from this source without deciding the scale itself. Adopting '25' as a fraction would disable the cutoff; adopting '.25' as a percent would reject almost everything.
- *Resolution*: not resolvable from these files - needs the IP manual or a run-time test

### G-2 — FilterLow / FilterHigh carry NO unit anywhere in the file.

Values run 1 to 30 (FilterLow) and 3 to 30 (FilterHigh). The ToolTemplate declares TimeUnits for the time axis but there is no FrequencyUnits attribute anywhere in the schema.

- *Impact*: kHz is the only physically sensible reading for a sonic bandpass at these magnitudes, but that is INFERENCE, not a file statement. Recorded as inference, not as fact.
- *Resolution*: not resolvable from these files

### G-3 — .bor files carry no machine-readable tool identity.

Tool name, pad numbers, sense of rotation and depth reference exist ONLY in free-text '$' comments (e.g. 'Button order for Baker STAR tool pads 1, 3 and 5' / '24 buttons order is anti-clockwise around hole. Button 2 is the depth reference at 3.48"'). Two files (HAL_EMIpads_1_3_5.bor, HAL_EMIpads_2_4_6.bor) even misspell the vendor as 'Halliburtom'.

- *Impact*: Any importer must treat the .bor comment block as documentation for a human, not as metadata.
- *Resolution*: inherent to the format

### G-4 — .eli column units are never declared in the .eli itself.

The two-line '$' legend names the columns (Number Buttons / Button Spacing / Orientation to Azimuth / Depth Shift / Rotation Shift) but states no units. Inches for spacing and degrees for orientation are corroborated only by the companion .bor comments and by the equivalent .itt/.itp fields which DO declare units.

- *Impact*: Cross-format corroboration is required; do not read a .eli standalone.
- *Resolution*: corroborated via .itp comment header (inches) and .itt comment header (degrees)

### G-5 — Data-entry defects exist in the shipped set - the vendor's own data is not clean.

Baker WGI.itt has <ZAccelerationCurve>sec</ZAccelerationCurve> - 'sec' is a unit string in a field that must hold a curve mnemonic. Two .itt files (Schlumberger Dual OBMI, Lower.itt and Schlumberger Dual OBMI, Upper.itt) share the SAME ID 'Schlumberger Lower OBMI v1.1' despite being different tools. Weatherford HMI Lower.itp / Upper.itp declare ID/Name 'Standard HMI Lower/Upper v1.0' - no vendor prefix, unlike every sibling.

- *Impact*: Confirms the rule: do not treat vendor reference data as authoritative without validation. SandiBumi's importer needs a uniqueness check on tool ID and a mnemonic-shape check.
- *Resolution*: flagged; these are defects in the source, not parse errors

### G-6 — Vendor naming is inconsistent BETWEEN formats.

.itt Company uses full names (Schlumberger 26, Halliburton 13, Baker 11, Weatherford 10, Pathfinder 2, GOWell 1, empty 9). .att Company uses abbreviations (SLB 9, WFT 8, HAL 7, BHI 6, GE 2, Pathfinder 2, Gowell 2, APS 1, empty 23). 'Baker' vs 'BHI' and 'GOWell' vs 'Gowell' are the same vendor spelled differently in the same install.

- *Impact*: A vendor lookup keyed on this string breaks across formats. SandiBumi needs a canonical vendor id with an alias table - the same discipline as the mnemonic alias catalogue.
- *Resolution*: resolved by observation; SandiBumi should canonicalise

### G-7 — .t83 (94 files) deliberately NOT decoded.

Binary; excluded from this target by instruction. Their content and relationship to the text formats is unknown and is NOT inferred here.

- *Impact*: The catalogue may be incomplete with respect to whatever .t83 holds.
- *Resolution*: out of scope

### G-8 — DepthOfInvestigationValue is EMPTY in 17 of 72 .itt files.

10 files declare Units='Feet' with an empty value, 7 declare Units='Inches' with an empty value. DepthOfInvestigationCurve is present in 68/72 files and EMPTY in all 68.

- *Impact*: Depth of investigation is genuinely absent for those tools - recorded as 'not present in file', not filled with a textbook value.
- *Resolution*: recorded as absent

---

## 10. Companion machine-readable output

`G_legacy_tool_definitions.json` in this directory carries the complete structured catalogue:
`target`, `formats` (5), `tools` (224 — every file, with its declared name/ID/vendor/type and
key parameters), `defaults` (161 rows with parameter/value/units/scope/source_file),
`mnemonic_conventions` (29 roles), `reuse_assessment`, `tierC_flags`, and `gaps`.

Both files were generated by parsing the install tree read-only; the JSON's every value is derived
from file content rather than transcribed, and the markdown tables above are generated from the
same parse.
