import { useRef, useState, useEffect } from "react";
import { teamHex } from "../quiz/model";

// The room is a flat disc around the moderator (radius 1, +z straight ahead, +x to the right), drawn in
// perspective as if lying on the table in front of the presenter: farther points shrink towards the
// horizon. Seats are stored as true floor angles; only the drawing is projected.
const C = 2.5; // camera distance; smaller = stronger perspective
const WD = 400; // half width of the disc at the moderator's depth
const HD = 210; // depth scale
const HD_BACK = 95; // depth scale behind the moderator (270°), squashed so the map stays low
const TOP = 34;
const Y0 = TOP + (HD * C) / (1 + C); // screen y of the moderator
const MIN_DIST = 0.18;

const toRad = (deg) => (deg * Math.PI) / 180;
const scaleAt = (z) => C / (z + C);
const project = (x, z) => [x * WD * scaleAt(z), Y0 - z * (z < 0 ? HD_BACK : HD) * scaleAt(z)];
const seatPoint = (angle, dist) => project(dist * Math.sin(toRad(angle)), dist * Math.cos(toRad(angle)));

// Screen point -> floor point (inverse of project)
function unproject(sx, sy) {
  const zp = Math.min((Y0 - sy) / (sy > Y0 ? HD_BACK : HD), C * 0.9); // stay below the horizon
  const z = (zp * C) / (C - zp);
  return [sx / (WD * scaleAt(z)), z];
}

const arcPath = (half, dist) => {
  const points = [];
  for (let a = -half; a <= half + 0.001; a += 3) points.push(seatPoint(a, dist));
  return points.map(([x, y], i) => `${i ? "L" : "M"} ${x.toFixed(1)} ${y.toFixed(1)}`).join(" ");
};

// Seen from the presenter (bottom centre, facing the room). arc is the field of view in degrees (180 or
// 270). teams: [{ id, name, seat, ... }] in display order (index picks the color).
// Editable when onSeat is given: drag a marker, or click the map to place the team in `placing`.
// highlight: Set of team ids to emphasise (others are dimmed); used read-only in the presenter.
// fill: the map gets a fixed-size box (e.g. h-full) and widens its drawing area to the box, so the
// whole width is clickable. markerScale enlarges markers and names for small read-only maps.
export default function VenueMap({ arc = 180, teams, onSeat, placing = null, highlight = null, youLabel, fill = false, markerScale = 1, className = "" }) {
  const svgRef = useRef(null);
  const [dragging, setDragging] = useState(null);
  const [box, setBox] = useState(null);
  const half = arc / 2;
  const zBack = Math.cos(toRad(half)); // farthest point behind the moderator (negative for 270°)
  const height = (zBack < 0 ? project(0, zBack)[1] : Y0) + 34;
  const baseWidth = 2 * (WD + 40);

  useEffect(() => {
    if (!fill) return;
    const observer = new ResizeObserver(([entry]) => setBox({ w: entry.contentRect.width, h: entry.contentRect.height }));
    observer.observe(svgRef.current);
    return () => observer.disconnect();
  }, [fill]);
  const viewWidth = fill && box?.h ? Math.max(baseWidth, (height * box.w) / box.h) : baseWidth;

  const seatAt = (e) => {
    const svg = svgRef.current;
    const p = svg.createSVGPoint();
    p.x = e.clientX;
    p.y = e.clientY;
    const { x: sx, y: sy } = p.matrixTransform(svg.getScreenCTM().inverse());
    const [x, z] = unproject(sx, sy);
    const angle = Math.max(-half, Math.min(half, (Math.atan2(x, z) * 180) / Math.PI));
    const dist = Math.max(MIN_DIST, Math.min(1, Math.hypot(x, z)));
    return { angle: Math.round(angle), dist: Math.round(dist * 100) / 100 };
  };

  const editable = Boolean(onSeat);
  const handlers = editable ? {
    onPointerMove: (e) => dragging && onSeat(dragging, seatAt(e)),
    onPointerUp: () => setDragging(null),
    onPointerLeave: () => setDragging(null),
    onClick: (e) => placing && !dragging && onSeat(placing, seatAt(e)),
  } : {};

  const rays = [];
  for (let a = -Math.floor(half / 45) * 45; a <= half; a += 45) rays.push(a);
  const edge = arcPath(half, 1);
  const [lx, ly] = seatPoint(-half, 1);
  const [rx, ry] = seatPoint(half, 1);

  // Far teams first, so nearer markers are drawn on top
  const seated = teams
    .map((team, i) => ({ team, i }))
    .filter(({ team }) => team.seat)
    .sort((a, b) => b.team.seat.dist * Math.cos(toRad(b.team.seat.angle)) - a.team.seat.dist * Math.cos(toRad(a.team.seat.angle)));

  return (
    <svg ref={svgRef} viewBox={`${-viewWidth / 2} 0 ${viewWidth} ${height}`} className={`w-full select-none touch-none ${placing ? "cursor-crosshair" : ""} ${className}`} {...handlers}>
      {/* field of view, lying on the table */}
      <path d={`M 0 ${Y0} L ${lx} ${ly} ${edge.replace(/^M/, "L")} L ${rx} ${ry} Z`} fill="rgba(255,255,255,0.04)" stroke="rgba(255,255,255,0.18)" />
      <path d={arcPath(half, 0.55)} fill="none" stroke="rgba(255,255,255,0.07)" />
      {rays.map((a) => {
        const [x, y] = seatPoint(a, 1);
        return <line key={a} x1={0} y1={Y0} x2={x} y2={y} stroke="rgba(255,255,255,0.09)" strokeDasharray={a === 0 ? "none" : "6 6"} />;
      })}

      {/* the presenter */}
      <circle cx={0} cy={Y0} r={10} fill="#eab308" />
      <path d={`M -7 ${Y0 - 13} L 0 ${Y0 - 24} L 7 ${Y0 - 13} Z`} fill="#eab308" />
      {youLabel && <text x={18} y={Y0 + 5} fontSize="14" fill="#eab308" fontWeight="bold">{youLabel}</text>}

      {seated.map(({ team, i }) => {
        const z = team.seat.dist * Math.cos(toRad(team.seat.angle));
        const depth = Math.min(1.3, scaleAt(z)) * markerScale;
        const [x, y] = seatPoint(team.seat.angle, team.seat.dist);
        const isHighlighted = highlight?.has(team.id);
        const dimmed = highlight && !isHighlighted;
        const r = (isHighlighted ? 17 : 13) * depth;
        const label = team.name.length > 16 ? `${team.name.slice(0, 15)}…` : team.name;
        return (
          <g key={team.id} transform={`translate(${x} ${y})`} opacity={dimmed ? 0.25 : 1}
            onPointerDown={editable ? (e) => { e.stopPropagation(); e.currentTarget.ownerSVGElement.setPointerCapture?.(e.pointerId); setDragging(team.id); } : undefined}
            onClick={editable ? (e) => e.stopPropagation() : undefined}
            className={editable ? "cursor-grab" : ""}>
            <ellipse cy={r * 0.35} rx={r} ry={r * 0.35} fill="rgba(0,0,0,0.5)" />
            <circle r={r} fill={teamHex(team, i)} stroke={isHighlighted || placing === team.id ? "#facc15" : "rgba(255,255,255,0.6)"} strokeWidth={isHighlighted ? 3 : 1.5} />
            <text y={r + 15 * depth} textAnchor="middle" fontSize={(isHighlighted ? 15 : 13) * depth} fontWeight="bold" fill="white" stroke="#050505" strokeWidth="3" paintOrder="stroke">{label}</text>
          </g>
        );
      })}
    </svg>
  );
}
