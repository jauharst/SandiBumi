// Extract stroked vector paths (with color + CTM applied) from a chartbook page.
// Usage: node extract.mjs <pageNum> <out.json>
import { getDocument, OPS } from "pdfjs-dist/legacy/build/pdf.mjs";
import { readFileSync, writeFileSync } from "fs";

const PDF = "D:\\01. Work\\00. Guidebook\\chartbook.pdf";
const pageNum = parseInt(process.argv[2] || "237", 10);
const outFile = process.argv[3] || "page.json";

const data = new Uint8Array(readFileSync(PDF));
const doc = await getDocument({ data, useSystemFonts: true }).promise;
const page = await doc.getPage(pageNum);
const opList = await page.getOperatorList();

// --- CTM tracking ---------------------------------------------------------
const mul = (a, b) => [
  a[0] * b[0] + a[2] * b[1],
  a[1] * b[0] + a[3] * b[1],
  a[0] * b[2] + a[2] * b[3],
  a[1] * b[2] + a[3] * b[3],
  a[0] * b[4] + a[2] * b[5] + a[4],
  a[1] * b[4] + a[3] * b[5] + a[5],
];
const apply = (m, x, y) => [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];

let ctm = [1, 0, 0, 1, 0, 0];
const stack = [];
let strokeColor = [0, 0, 0];
let fillColor = [0, 0, 0];
let lineWidth = 1;
let dashed = false; // true while a non-empty dash array is set (setDash stroke property)

// current path being constructed (in device space after CTM)
let path = [];      // array of polylines: [[x,y],[x,y],...]
const out = [];     // emitted strokes: {color, width, polys}

function flatBezier(p0, p1, p2, p3, n = 16) {
  const pts = [];
  for (let i = 1; i <= n; i++) {
    const t = i / n, u = 1 - t;
    pts.push([
      u * u * u * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t * t * t * p3[0],
      u * u * u * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t * t * t * p3[1],
    ]);
  }
  return pts;
}

function buildPath(opsArr, coords) {
  let i = 0;
  let cur = null;
  let curPoly = null;
  for (const op of opsArr) {
    if (op === OPS.moveTo) {
      const p = apply(ctm, coords[i], coords[i + 1]); i += 2;
      curPoly = [p]; path.push(curPoly); cur = p;
    } else if (op === OPS.lineTo) {
      const p = apply(ctm, coords[i], coords[i + 1]); i += 2;
      if (!curPoly) { curPoly = [cur || p]; path.push(curPoly); }
      curPoly.push(p); cur = p;
    } else if (op === OPS.curveTo) {
      const c1 = apply(ctm, coords[i], coords[i + 1]);
      const c2 = apply(ctm, coords[i + 2], coords[i + 3]);
      const p = apply(ctm, coords[i + 4], coords[i + 5]); i += 6;
      if (!curPoly) { curPoly = [cur || c1]; path.push(curPoly); }
      curPoly.push(...flatBezier(cur || c1, c1, c2, p)); cur = p;
    } else if (op === OPS.curveTo2) {
      const c2 = apply(ctm, coords[i], coords[i + 1]);
      const p = apply(ctm, coords[i + 2], coords[i + 3]); i += 4;
      if (!curPoly) { curPoly = [cur || c2]; path.push(curPoly); }
      curPoly.push(...flatBezier(cur || c2, cur || c2, c2, p)); cur = p;
    } else if (op === OPS.curveTo3) {
      const c1 = apply(ctm, coords[i], coords[i + 1]);
      const p = apply(ctm, coords[i + 2], coords[i + 3]); i += 4;
      if (!curPoly) { curPoly = [cur || c1]; path.push(curPoly); }
      curPoly.push(...flatBezier(cur || c1, c1, p, p)); cur = p;
    } else if (op === OPS.closePath) {
      if (curPoly && curPoly.length > 1) curPoly.push(curPoly[0]);
    } else if (op === OPS.rectangle) {
      const x = coords[i], y = coords[i + 1], w = coords[i + 2], h = coords[i + 3]; i += 4;
      const c = [apply(ctm, x, y), apply(ctm, x + w, y), apply(ctm, x + w, y + h), apply(ctm, x, y + h), apply(ctm, x, y)];
      path.push(c); curPoly = null; cur = c[4];
    }
  }
}

function emit(kind) {
  if (path.length) {
    out.push({
      kind,
      color: kind === "fill" ? fillColor.slice() : strokeColor.slice(),
      width: lineWidth,
      dashed,
      polys: path.map(p => p.map(([x, y]) => [Math.round(x * 100) / 100, Math.round(y * 100) / 100])),
    });
  }
  path = [];
}

const fns = opList.fnArray, args = opList.argsArray;
for (let k = 0; k < fns.length; k++) {
  const fn = fns[k], a = args[k];
  switch (fn) {
    case OPS.save: stack.push({ ctm: ctm.slice(), strokeColor: strokeColor.slice(), fillColor: fillColor.slice(), lineWidth, dashed }); break;
    case OPS.restore: { const s = stack.pop(); if (s) { ctm = s.ctm; strokeColor = s.strokeColor; fillColor = s.fillColor; lineWidth = s.lineWidth; dashed = s.dashed; } break; }
    case OPS.transform: ctm = mul(ctm, a); break;
    case OPS.setDash: dashed = Array.isArray(a[0]) && a[0].length > 0; break;
    case OPS.setStrokeRGBColor: strokeColor = [a[0], a[1], a[2]]; break;
    case OPS.setFillRGBColor: fillColor = [a[0], a[1], a[2]]; break;
    case OPS.setStrokeGray: strokeColor = [a[0], a[0], a[0]]; break;
    case OPS.setFillGray: fillColor = [a[0], a[0], a[0]]; break;
    case OPS.setStrokeCMYKColor: { const [c, m2, y2, kk] = a; strokeColor = [(1 - c) * (1 - kk), (1 - m2) * (1 - kk), (1 - y2) * (1 - kk)]; break; }
    case OPS.setFillCMYKColor: { const [c, m2, y2, kk] = a; fillColor = [(1 - c) * (1 - kk), (1 - m2) * (1 - kk), (1 - y2) * (1 - kk)]; break; }
    case OPS.setLineWidth: lineWidth = a[0]; break;
    case OPS.constructPath: {
      // pdfjs 4.x: args = [opsArray, coordsArray, minMax]
      buildPath(a[0], a[1]);
      break;
    }
    case OPS.stroke: emit("stroke"); break;
    case OPS.closeStroke: emit("stroke"); break;
    case OPS.fill: emit("fill"); break;
    case OPS.eoFill: emit("fill"); break;
    case OPS.fillStroke: emit("fillstroke"); break;
    case OPS.eoFillStroke: emit("fillstroke"); break;
    case OPS.endPath: path = []; break;
    default: break;
  }
}

// Also dump text items with positions (for tick labels / calibration sanity)
const tc = await page.getTextContent();
const texts = tc.items.map(it => ({ s: it.str, x: Math.round(it.transform[4] * 100) / 100, y: Math.round(it.transform[5] * 100) / 100, w: Math.round((it.width || 0) * 100) / 100 })).filter(t => t.s.trim());

writeFileSync(outFile, JSON.stringify({ page: pageNum, view: page.view, strokes: out, texts }, null, 0));
const colors = {};
for (const s of out) { const key = s.color.map(c => Math.round(c)).join(","); colors[key] = (colors[key] || 0) + s.polys.length; }
console.log("strokes:", out.length, "| polys per color:", JSON.stringify(colors), "| texts:", texts.length);
