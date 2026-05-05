export interface AquariumOptionFrame {
  key: string;
  label: string;
  disabled?: boolean;
}

export interface AquariumAgentFrame {
  id: string;
  name: string;
  glyph: string;
  shape: string;
  status: string;
  thought: string;
  color: string;
  glow: string;
  baseX: number;
  baseY: number;
  driftX: number;
  driftY: number;
  phase: number;
  activity: number;
  options: AquariumOptionFrame[];
}

export interface AquariumUiButtonFrame {
  key: string;
  label: string;
  disabled?: boolean;
  tone?: string;
}

export interface AquariumUiStatusFrame {
  label: string;
  tone?: string;
}

export interface AquariumUiFrame {
  eyebrow: string;
  title: string;
  reason: string;
  activeDeckLabel: string;
  activeSubdeck: string;
  statuses: AquariumUiStatusFrame[];
  deckButtons: AquariumUiButtonFrame[];
  subdeckButtons: AquariumUiButtonFrame[];
  actionButtons: AquariumUiButtonFrame[];
  panelTitle: string;
  panelLines: string[];
  alert?: string;
}

export interface AquariumFrame {
  agents: AquariumAgentFrame[];
  selectedAgentId: string;
  activeLabel?: string;
  ui?: AquariumUiFrame;
  variant: "band" | "fullscreen";
}

export interface AquariumRenderer {
  clearPointer(): void;
  dispose(): void;
  pickAgent(): string | null;
  pickOption(): string | null;
  pointerDownClient(clientX: number, clientY: number): void;
  pointerUp(): void;
  setFrame(frame: AquariumFrame): void;
  setPointerClient(clientX: number, clientY: number): void;
}

interface FluidTarget {
  texture: WebGLTexture;
  fbo: WebGLFramebuffer;
  width: number;
  height: number;
}

interface DoubleTarget {
  read: FluidTarget;
  write: FluidTarget;
  swap: () => void;
}

interface MotionState {
  x: number;
  y: number;
  vx: number;
  vy: number;
}

interface AgentPersonality {
  angle: number;
  radius: number;
  eccentricity: number;
  orbitSpeed: number;
  radialTempo: number;
  tangentialTempo: number;
  expressiveness: number;
  glowTempo: number;
  inkTempo: number;
  precision: number;
  hoverStillness: number;
}

interface AgentStateVector {
  activity: number;
  blocked: number;
  ready: number;
  review: number;
  urgency: number;
}

interface AgentChirpMatrix {
  angle: number;
  distortion: number;
  expression: number;
  glowPulse: number;
  hoverDamping: number;
  inkPulse: number;
  orbitRadius: number;
  radial: number;
  tangential: number;
}

interface ProjectedAgent extends AquariumAgentFrame, MotionState {
  chirps: AgentChirpMatrix;
  index: number;
  emissionPulse: number;
  hover: number;
  speed: number;
}

interface HotZone {
  x: number;
  y: number;
  radius: number;
  key: string;
  height?: number;
  width?: number;
}

type FluidParamKey =
  | "timeScale"
  | "curlStrength"
  | "swirlForce"
  | "splatForce"
  | "splatRadius"
  | "velocityDissipation"
  | "dyeDissipation"
  | "injectionGain"
  | "sourceOpacity"
  | "acesExposure"
  | "acesGlow"
  | "acesSaturation";

type FluidParams = Record<FluidParamKey, number>;

interface FluidParamDefinition {
  key: FluidParamKey;
  label: string;
  min: number;
  max: number;
  decimals: number;
  scale?: "linear" | "log" | "softLog" | "persistenceLog";
  softMin?: number;
}

interface FluidParamZone {
  key: "toggle" | "reset" | FluidParamKey;
  x: number;
  y: number;
  width: number;
  height: number;
}

const fullscreenPositions: Record<string, { x: number; y: number }> = {
  coordinator: { x: 60, y: 42 },
  imagination: { x: 60, y: 19 },
  research: { x: 90, y: 24 },
  reorientation: { x: 83, y: 43 },
  modeling: { x: 64, y: 55 },
  verification: { x: 92, y: 58 },
  implementation: { x: 76, y: 67 },
};

const compactPositions: Record<string, { x: number; y: number }> = {
  coordinator: { x: 50, y: 25 },
  imagination: { x: 14, y: 34 },
  research: { x: 86, y: 34 },
  reorientation: { x: 50, y: 43 },
  modeling: { x: 14, y: 52 },
  verification: { x: 86, y: 52 },
  implementation: { x: 50, y: 61 },
};

const agentPersonalities: Record<string, AgentPersonality> = {
  coordinator: {
    angle: -0.36,
    radius: 0,
    eccentricity: 0.04,
    orbitSpeed: 0.012,
    radialTempo: 0.18,
    tangentialTempo: 0.14,
    expressiveness: 0.34,
    glowTempo: 0.34,
    inkTempo: 0.22,
    precision: 0.82,
    hoverStillness: 0.9,
  },
  imagination: {
    angle: -2.22,
    radius: 0.72,
    eccentricity: 0.2,
    orbitSpeed: 0.026,
    radialTempo: 0.42,
    tangentialTempo: 0.38,
    expressiveness: 1.16,
    glowTempo: 0.9,
    inkTempo: 0.72,
    precision: 0.34,
    hoverStillness: 0.82,
  },
  research: {
    angle: -0.76,
    radius: 0.78,
    eccentricity: 0.1,
    orbitSpeed: 0.018,
    radialTempo: 0.28,
    tangentialTempo: 0.22,
    expressiveness: 0.62,
    glowTempo: 0.46,
    inkTempo: 0.33,
    precision: 0.92,
    hoverStillness: 0.9,
  },
  reorientation: {
    angle: 0.1,
    radius: 0.48,
    eccentricity: 0.16,
    orbitSpeed: 0.014,
    radialTempo: 0.24,
    tangentialTempo: 0.2,
    expressiveness: 0.74,
    glowTempo: 0.56,
    inkTempo: 0.42,
    precision: 0.74,
    hoverStillness: 0.86,
  },
  modeling: {
    angle: 2.34,
    radius: 0.62,
    eccentricity: 0.08,
    orbitSpeed: 0.015,
    radialTempo: 0.2,
    tangentialTempo: 0.18,
    expressiveness: 0.46,
    glowTempo: 0.38,
    inkTempo: 0.3,
    precision: 0.88,
    hoverStillness: 0.92,
  },
  verification: {
    angle: 1.02,
    radius: 0.7,
    eccentricity: 0.12,
    orbitSpeed: 0.017,
    radialTempo: 0.26,
    tangentialTempo: 0.2,
    expressiveness: 0.56,
    glowTempo: 0.42,
    inkTempo: 0.28,
    precision: 0.96,
    hoverStillness: 0.94,
  },
  implementation: {
    angle: 1.74,
    radius: 0.84,
    eccentricity: 0.06,
    orbitSpeed: 0.022,
    radialTempo: 0.36,
    tangentialTempo: 0.3,
    expressiveness: 0.88,
    glowTempo: 0.62,
    inkTempo: 0.58,
    precision: 0.6,
    hoverStillness: 0.8,
  },
};

const fluidParamStorageKey = "epiphany:aquarium-fluid-params:v3";

const defaultFluidParams: FluidParams = {
  timeScale: 0.032,
  curlStrength: 64,
  swirlForce: 90,
  splatForce: 7.5,
  splatRadius: 54,
  velocityDissipation: 0.996,
  dyeDissipation: 0.9994,
  injectionGain: 0.075,
  sourceOpacity: 1.15,
  acesExposure: 1.32,
  acesGlow: 0.82,
  acesSaturation: 1.16,
};

const fluidParamDefinitions: FluidParamDefinition[] = [
  { key: "timeScale", label: "Flow Speed", min: 0, max: 0.16, decimals: 5, scale: "softLog", softMin: 0.000001 },
  { key: "curlStrength", label: "Eddy Curl", min: 0, max: 180, decimals: 1, scale: "softLog", softMin: 0.1 },
  { key: "swirlForce", label: "Wake Swirl", min: 0, max: 220, decimals: 1, scale: "softLog", softMin: 0.1 },
  { key: "splatForce", label: "Wake Force", min: 0, max: 24, decimals: 2, scale: "softLog", softMin: 0.02 },
  { key: "splatRadius", label: "Wake Radius", min: 1, max: 180, decimals: 1, scale: "log" },
  { key: "velocityDissipation", label: "Flow Persistence", min: 0.05, max: 0.9998, decimals: 4, scale: "persistenceLog" },
  { key: "dyeDissipation", label: "Ink Persistence", min: 0.05, max: 0.99995, decimals: 4, scale: "persistenceLog" },
  { key: "injectionGain", label: "Ink Gain", min: 0, max: 0.5, decimals: 4, scale: "softLog", softMin: 0.0005 },
  { key: "sourceOpacity", label: "Emitter Strength", min: 0, max: 4, decimals: 3, scale: "softLog", softMin: 0.004 },
  { key: "acesExposure", label: "Exposure", min: 0.1, max: 4.5, decimals: 2, scale: "log" },
  { key: "acesGlow", label: "Glow", min: 0, max: 2.6, decimals: 2, scale: "softLog", softMin: 0.003 },
  { key: "acesSaturation", label: "Saturation", min: 0.35, max: 2.2, decimals: 2 },
];

const fluidForceScales = [
  { radius: 2.25, force: 0.42, curl: 0.52, inject: 0.28 },
  { radius: 0.94, force: 0.76, curl: 0.82, inject: 0.36 },
  { radius: 0.38, force: 0.58, curl: 1.28, inject: 0.42 },
];

const vertexShader = `#version 300 es
precision highp float;
out vec2 vUv;
const vec2 positions[3] = vec2[3](
  vec2(-1.0, -1.0),
  vec2(3.0, -1.0),
  vec2(-1.0, 3.0)
);
void main() {
  vec2 position = positions[gl_VertexID];
  vUv = position * 0.5 + 0.5;
  gl_Position = vec4(position, 0.0, 1.0);
}`;

const advectShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uVelocity;
uniform sampler2D uSource;
uniform vec2 uTexelSize;
uniform float uDt;
uniform float uDissipation;
void main() {
  vec2 velocity = texture(uVelocity, vUv).xy;
  vec2 coord = vUv - velocity * uTexelSize * uDt;
  outColor = texture(uSource, coord) * uDissipation;
}`;

const velocitySplatShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uVelocity;
uniform vec4 uAgents[7];
uniform float uActivity[7];
uniform int uCount;
uniform float uAspect;
uniform float uSplatForce;
uniform float uSplatRadius;
uniform float uSwirlForce;
uniform float uVelocityDamping;
void main() {
  vec2 velocity = texture(uVelocity, vUv).xy * uVelocityDamping;
  for (int i = 0; i < 7; i += 1) {
    if (i >= uCount) break;
    vec2 agent = uAgents[i].xy;
    vec2 delta = vUv - agent;
    delta.x *= uAspect;
    float influence = exp(-dot(delta, delta) * uSplatRadius);
    vec2 tangent = normalize(vec2(-delta.y, delta.x) + 0.0001) * (uSwirlForce * 0.64 + uActivity[i] * uSwirlForce);
    vec2 push = uAgents[i].zw * uSplatForce + tangent;
    velocity += push * influence * (0.18 + uActivity[i] * 0.24);
  }
  outColor = vec4(velocity, 0.0, 1.0);
}`;

const curlShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uVelocity;
uniform vec2 uTexelSize;
void main() {
  float left = texture(uVelocity, vUv - vec2(uTexelSize.x, 0.0)).y;
  float right = texture(uVelocity, vUv + vec2(uTexelSize.x, 0.0)).y;
  float bottom = texture(uVelocity, vUv - vec2(0.0, uTexelSize.y)).x;
  float top = texture(uVelocity, vUv + vec2(0.0, uTexelSize.y)).x;
  float curl = right - left - top + bottom;
  outColor = vec4(curl, 0.0, 0.0, 1.0);
}`;

const vorticityShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uVelocity;
uniform sampler2D uCurl;
uniform vec2 uTexelSize;
uniform float uCurlStrength;
uniform float uDt;
void main() {
  float left = abs(texture(uCurl, vUv - vec2(uTexelSize.x, 0.0)).x);
  float right = abs(texture(uCurl, vUv + vec2(uTexelSize.x, 0.0)).x);
  float bottom = abs(texture(uCurl, vUv - vec2(0.0, uTexelSize.y)).x);
  float top = abs(texture(uCurl, vUv + vec2(0.0, uTexelSize.y)).x);
  float center = texture(uCurl, vUv).x;
  vec2 force = 0.5 * vec2(right - left, top - bottom);
  force = normalize(force + 0.0001) * center * uCurlStrength;
  vec2 velocity = texture(uVelocity, vUv).xy + force * uDt;
  outColor = vec4(velocity, 0.0, 1.0);
}`;

const divergenceShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uVelocity;
uniform vec2 uTexelSize;
void main() {
  float left = texture(uVelocity, vUv - vec2(uTexelSize.x, 0.0)).x;
  float right = texture(uVelocity, vUv + vec2(uTexelSize.x, 0.0)).x;
  float bottom = texture(uVelocity, vUv - vec2(0.0, uTexelSize.y)).y;
  float top = texture(uVelocity, vUv + vec2(0.0, uTexelSize.y)).y;
  float divergence = 0.5 * (right - left + top - bottom);
  outColor = vec4(divergence, 0.0, 0.0, 1.0);
}`;

const pressureShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uPressure;
uniform sampler2D uDivergence;
uniform vec2 uTexelSize;
void main() {
  float left = texture(uPressure, vUv - vec2(uTexelSize.x, 0.0)).x;
  float right = texture(uPressure, vUv + vec2(uTexelSize.x, 0.0)).x;
  float bottom = texture(uPressure, vUv - vec2(0.0, uTexelSize.y)).x;
  float top = texture(uPressure, vUv + vec2(0.0, uTexelSize.y)).x;
  float divergence = texture(uDivergence, vUv).x;
  float pressure = (left + right + bottom + top - divergence) * 0.25;
  outColor = vec4(pressure, 0.0, 0.0, 1.0);
}`;

const gradientShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uPressure;
uniform sampler2D uVelocity;
uniform vec2 uTexelSize;
void main() {
  float left = texture(uPressure, vUv - vec2(uTexelSize.x, 0.0)).x;
  float right = texture(uPressure, vUv + vec2(uTexelSize.x, 0.0)).x;
  float bottom = texture(uPressure, vUv - vec2(0.0, uTexelSize.y)).x;
  float top = texture(uPressure, vUv + vec2(0.0, uTexelSize.y)).x;
  vec2 velocity = texture(uVelocity, vUv).xy - vec2(right - left, top - bottom) * 0.48;
  outColor = vec4(velocity, 0.0, 1.0);
}`;

const injectShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uDye;
uniform sampler2D uSource;
uniform float uGain;
uniform float uDissipation;
void main() {
  vec4 dye = texture(uDye, vUv);
  vec4 source = texture(uSource, vUv);
  float sourceWeight = source.a * uGain;
  vec3 color = max(dye.rgb, vec3(0.0)) * uDissipation + source.rgb * sourceWeight;
  float density = max(dye.a, 0.0) * uDissipation + sourceWeight;
  outColor = vec4(min(color, vec3(32.0)), min(density, 48.0));
}`;

const displayShader = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uDye;
uniform vec2 uTexelSize;
uniform float uExposure;
uniform float uGlow;
uniform float uSaturation;
vec3 acesFilm(vec3 x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}
vec3 gradeSaturation(vec3 color, float saturation) {
  float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
  return mix(vec3(luma), color, saturation);
}
void main() {
  vec4 dye = texture(uDye, vUv);
  vec3 color = max(dye.rgb, vec3(0.0));
  float density = max(dye.a, 0.0);
  vec3 glowColor = color;
  float glowDensity = density;
  glowColor += texture(uDye, vUv + vec2(uTexelSize.x * 2.5, 0.0)).rgb;
  glowColor += texture(uDye, vUv - vec2(uTexelSize.x * 2.5, 0.0)).rgb;
  glowColor += texture(uDye, vUv + vec2(0.0, uTexelSize.y * 2.5)).rgb;
  glowColor += texture(uDye, vUv - vec2(0.0, uTexelSize.y * 2.5)).rgb;
  glowDensity += texture(uDye, vUv + vec2(uTexelSize.x * 4.5, uTexelSize.y * 4.5)).a;
  glowDensity += texture(uDye, vUv + vec2(-uTexelSize.x * 4.5, uTexelSize.y * 4.5)).a;
  glowDensity += texture(uDye, vUv + vec2(uTexelSize.x * 4.5, -uTexelSize.y * 4.5)).a;
  glowDensity += texture(uDye, vUv + vec2(-uTexelSize.x * 4.5, -uTexelSize.y * 4.5)).a;
  glowColor /= 5.0;
  glowDensity /= 5.0;
  float glow = smoothstep(0.04, 3.8, glowDensity) * uGlow;
  color = color * uExposure + glowColor * glow * 0.55;
  color = acesFilm(color);
  color = gradeSaturation(color, uSaturation);
  color = pow(max(color, vec3(0.0)), vec3(0.92));
  outColor = vec4(color, 1.0);
}`;

function loadFluidParams(): FluidParams {
  if (typeof window === "undefined") return { ...defaultFluidParams };
  try {
    const raw = window.localStorage.getItem(fluidParamStorageKey);
    if (!raw) return { ...defaultFluidParams };
    const parsed = JSON.parse(raw) as Partial<FluidParams>;
    return normalizeFluidParams(parsed);
  } catch {
    return { ...defaultFluidParams };
  }
}

function normalizeFluidParams(value: Partial<FluidParams>): FluidParams {
  const next = { ...defaultFluidParams };
  for (const definition of fluidParamDefinitions) {
    const candidate = value[definition.key];
    if (typeof candidate === "number" && Number.isFinite(candidate)) {
      next[definition.key] = clamp(candidate, definition.min, definition.max);
    }
  }
  return next;
}

function saveFluidParams(params: FluidParams) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(fluidParamStorageKey, JSON.stringify(params));
}

export function createAquariumRenderer(canvas: HTMLCanvasElement): AquariumRenderer;
export function createAquariumRenderer(canvas: HTMLCanvasElement, crispCanvas: HTMLCanvasElement | null): AquariumRenderer;
export function createAquariumRenderer(canvas: HTMLCanvasElement, crispCanvas: HTMLCanvasElement | null = null): AquariumRenderer {
  const gl = canvas.getContext("webgl2", { alpha: false, antialias: false, preserveDrawingBuffer: true });
  if (!gl || !gl.getExtension("EXT_color_buffer_float")) {
    return new CanvasAquariumRenderer(canvas, crispCanvas);
  }
  return new WebglAquariumRenderer(canvas, gl, crispCanvas);
}

class WebglAquariumRenderer implements AquariumRenderer {
  private activity = new Float32Array(7);
  private agentsUniform = new Float32Array(7 * 4);
  private curl: FluidTarget | null = null;
  private dye: DoubleTarget | null = null;
  private draggingFluidParam: FluidParamKey | null = null;
  private divergence: FluidTarget | null = null;
  private fluidPanelPinned = false;
  private fluidParams = loadFluidParams();
  private fluidParamZones: FluidParamZone[] = [];
  private frame: AquariumFrame = { agents: [], selectedAgentId: "coordinator", variant: "fullscreen" };
  private hotAgents: HotZone[] = [];
  private hotOptions: HotZone[] = [];
  private lastFluidParamChanged: FluidParamKey | null = null;
  private motion = new Map<string, MotionState>();
  private paramImpulse = 0;
  private pointer = { active: false, x: 0, y: 0 };
  private pressure: DoubleTarget | null = null;
  private programs: Record<string, WebGLProgram>;
  private raf = 0;
  private simHeight = 0;
  private simWidth = 0;
  private sourceCanvas = document.createElement("canvas");
  private sourceContext: CanvasRenderingContext2D;
  private sourceTexture: WebGLTexture | null = null;
  private time = 0;
  private velocity: DoubleTarget | null = null;

  private crispContext: CanvasRenderingContext2D | null = null;

  constructor(private canvas: HTMLCanvasElement, private gl: WebGL2RenderingContext, private crispCanvas: HTMLCanvasElement | null) {
    const sourceContext = this.sourceCanvas.getContext("2d");
    if (!sourceContext) {
      throw new Error("Aquarium source canvas could not be created.");
    }
    this.sourceContext = sourceContext;
    this.crispContext = crispCanvas?.getContext("2d") ?? null;
    this.programs = {
      advect: compileProgram(gl, vertexShader, advectShader),
      curl: compileProgram(gl, vertexShader, curlShader),
      display: compileProgram(gl, vertexShader, displayShader),
      divergence: compileProgram(gl, vertexShader, divergenceShader),
      gradient: compileProgram(gl, vertexShader, gradientShader),
      inject: compileProgram(gl, vertexShader, injectShader),
      pressure: compileProgram(gl, vertexShader, pressureShader),
      velocitySplat: compileProgram(gl, vertexShader, velocitySplatShader),
      vorticity: compileProgram(gl, vertexShader, vorticityShader),
    };
    this.raf = requestAnimationFrame(this.render);
  }

  clearPointer() {
    this.pointer = { active: false, x: 0, y: 0 };
  }

  dispose() {
    cancelAnimationFrame(this.raf);
  }

  pickAgent() {
    const hit = this.hotAgents.find((zone) => hitZone(zone, this.pointer.x, this.pointer.y));
    return hit?.key ?? null;
  }

  pickOption() {
    if (this.fluidParamZones.some((zone) => pointInRect(this.pointer.x, this.pointer.y, zone))) {
      return null;
    }
    const hit = this.hotOptions.find((zone) => hitZone(zone, this.pointer.x, this.pointer.y));
    return hit?.key ?? null;
  }

  setFrame(frame: AquariumFrame) {
    this.frame = frame;
  }

  setPointerClient(clientX: number, clientY: number) {
    const rect = this.canvas.getBoundingClientRect();
    const x = ((clientX - rect.left) / Math.max(rect.width, 1)) * this.simWidth;
    const y = ((clientY - rect.top) / Math.max(rect.height, 1)) * this.simHeight;
    this.pointer = { active: true, x, y };
    if (this.draggingFluidParam) {
      this.updateFluidParamFromPointer(this.draggingFluidParam, x);
    }
  }

  pointerDownClient(clientX: number, clientY: number) {
    this.setPointerClient(clientX, clientY);
    const zone = this.fluidParamZones.find((candidate) => pointInRect(this.pointer.x, this.pointer.y, candidate));
    if (!zone) return;
    if (zone.key === "toggle") {
      this.fluidPanelPinned = !this.fluidPanelPinned;
      return;
    }
    if (zone.key === "reset") {
      this.fluidParams = { ...defaultFluidParams };
      saveFluidParams(this.fluidParams);
      this.paramImpulse = 1;
      this.lastFluidParamChanged = "injectionGain";
      this.seedDye();
      return;
    }
    this.draggingFluidParam = zone.key;
    this.updateFluidParamFromPointer(zone.key, this.pointer.x);
  }

  pointerUp() {
    this.draggingFluidParam = null;
  }

  private render = (millis: number) => {
    this.resize();
    if (!this.velocity || !this.dye || !this.pressure || !this.divergence || !this.curl || !this.sourceTexture) {
      this.raf = requestAnimationFrame(this.render);
      return;
    }
    const time = millis / 1000;
    this.time = time;
    const projected = this.projectAgents(time);
    const activeAgent = this.nearestAgent(projected) ?? projected.find((agent) => agent.id === this.frame.selectedAgentId) ?? projected[0];
    this.drawSource(projected, activeAgent, time);
    this.uploadSource();
    this.stepFluid(projected);
    this.display();
    this.drawCrispOverlay(projected, activeAgent, time);
    this.raf = requestAnimationFrame(this.render);
  };

  private resize() {
    const rect = this.canvas.getBoundingClientRect();
    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    const displayWidth = Math.max(1, Math.floor(rect.width * dpr));
    const displayHeight = Math.max(1, Math.floor(rect.height * dpr));
    if (this.canvas.width !== displayWidth || this.canvas.height !== displayHeight) {
      this.canvas.width = displayWidth;
      this.canvas.height = displayHeight;
    }
    if (this.crispCanvas && (this.crispCanvas.width !== displayWidth || this.crispCanvas.height !== displayHeight)) {
      this.crispCanvas.width = displayWidth;
      this.crispCanvas.height = displayHeight;
    }
    const scale = this.frame.variant === "fullscreen" ? 0.72 : 0.76;
    const width = Math.max(256, Math.min(1280, Math.floor(displayWidth * scale)));
    const height = Math.max(192, Math.min(820, Math.floor(displayHeight * scale)));
    if (width === this.simWidth && height === this.simHeight) return;
    this.simWidth = width;
    this.simHeight = height;
    this.sourceCanvas.width = width;
    this.sourceCanvas.height = height;
    this.velocity = this.createDoubleTarget(width, height);
    this.dye = this.createDoubleTarget(width, height);
    this.pressure = this.createDoubleTarget(width, height);
    this.divergence = this.createTarget(width, height);
    this.curl = this.createTarget(width, height);
    this.sourceTexture = this.createSourceTexture(width, height);
    this.seedDye();
  }

  private createTarget(width: number, height: number): FluidTarget {
    const gl = this.gl;
    const texture = gl.createTexture();
    const fbo = gl.createFramebuffer();
    if (!texture || !fbo) throw new Error("WebGL target creation failed.");
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA16F, width, height, 0, gl.RGBA, gl.HALF_FLOAT, null);
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
    return { texture, fbo, width, height };
  }

  private createDoubleTarget(width: number, height: number): DoubleTarget {
    const target = {
      read: this.createTarget(width, height),
      write: this.createTarget(width, height),
      swap() {
        const next = target.read;
        target.read = target.write;
        target.write = next;
      },
    };
    return target;
  }

  private createSourceTexture(width: number, height: number) {
    const gl = this.gl;
    const texture = gl.createTexture();
    if (!texture) throw new Error("Source texture creation failed.");
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    return texture;
  }

  private seedDye() {
    if (!this.dye) return;
    this.sourceContext.save();
    this.sourceContext.globalAlpha = 1;
    this.sourceContext.fillStyle = "#07110e";
    this.sourceContext.fillRect(0, 0, this.simWidth, this.simHeight);
    this.sourceContext.restore();
    this.uploadSource();
    this.bindTexture(0, this.sourceTexture);
    this.drawTo(this.dye.read, this.programs.inject, (gl, program) => {
      gl.uniform1i(gl.getUniformLocation(program, "uDye"), 0);
      gl.uniform1i(gl.getUniformLocation(program, "uSource"), 0);
      gl.uniform1f(gl.getUniformLocation(program, "uGain"), 1);
      gl.uniform1f(gl.getUniformLocation(program, "uDissipation"), 1);
    });
  }

  private projectAgents(time: number): ProjectedAgent[] {
    this.hotAgents = [];
    const selfAgent = this.frame.agents.find((agent) => agent.id === "coordinator") ?? this.frame.agents[0];
    const selfBase = selfAgent ? this.basePoint(selfAgent) : { x: this.simWidth * 0.5, y: this.simHeight * 0.42 };
    let selfAnchor = this.motion.get("coordinator") ?? { x: selfBase.x, y: selfBase.y, vx: 0, vy: 0 };
    const orbitScale = Math.min(this.simWidth, this.simHeight) * (this.simWidth < 540 ? 0.3 : 0.32);
    return this.frame.agents.map((agent, index) => {
      const base = this.basePoint(agent);
      const state = this.motion.get(agent.id) ?? { x: base.x, y: base.y, vx: 0, vy: 0 };
      const personality = personalityFor(agent.id);
      const stateVector = projectAgentState(agent);
      const activity = Math.max(0.04, stateVector.activity);
      const hover = this.pointer.active ? hoverInfluence(state.x, state.y, this.pointer.x, this.pointer.y, 104) : 0;
      const chirps = projectChirpMatrix(agent, personality, stateVector, time, hover);
      const pull = this.pointer.active ? this.pointerPull(agent, state.x, state.y) : { x: 0, y: 0 };
      const pointerPull = { x: pull.x * lerp(1, 0.08, hover), y: pull.y * lerp(1, 0.08, hover) };
      const orbitRadius = orbitScale * chirps.orbitRadius;
      const angle = chirps.angle;
      const normalX = Math.cos(angle);
      const normalY = Math.sin(angle);
      const tangentX = -normalY;
      const tangentY = normalX;
      const swim = (this.frame.variant === "fullscreen" ? 18 + activity * 66 : 7 + activity * 20) * chirps.expression;
      const target =
        agent.id === "coordinator"
          ? {
              x: base.x + chirps.radial * swim * 0.26 + pointerPull.x,
              y: base.y + chirps.tangential * swim * 0.18 + pointerPull.y,
            }
          : {
              x:
                selfAnchor.x +
                normalX * orbitRadius * (1 + personality.eccentricity) +
                (normalX * chirps.radial + tangentX * chirps.tangential) * swim +
                pointerPull.x,
              y:
                selfAnchor.y +
                normalY * orbitRadius * (0.78 - personality.eccentricity * 0.18) +
                (normalY * chirps.radial + tangentY * chirps.tangential) * swim * 0.72 +
                pointerPull.y,
            };
      const follow = (0.0026 + activity * 0.006 + personality.expressiveness * 0.0018 + stateVector.urgency * 0.003) * chirps.hoverDamping;
      const damping = lerp(0.94, 0.78, hover);
      state.vx = state.vx * damping + (target.x - state.x) * follow;
      state.vy = state.vy * damping + (target.y - state.y) * follow;
      state.x = clamp(state.x + state.vx, 42, this.simWidth - 42);
      state.y = clamp(state.y + state.vy, 50, this.simHeight - 50);
      this.motion.set(agent.id, state);
      if (agent.id === "coordinator") {
        selfAnchor = state;
      }
      this.hotAgents.push({ x: state.x, y: state.y, radius: 54, key: agent.id });
      const emissionPulse = chirps.inkPulse * lerp(1, 0.5, hover);
      return { ...agent, ...state, chirps, emissionPulse, hover, index, speed: Math.hypot(state.vx, state.vy) };
    });
  }

  private basePoint(agent: AquariumAgentFrame) {
    const compact = this.simWidth < 540 ? compactPositions[agent.id] : undefined;
    const fullscreen = this.frame.variant === "fullscreen" ? fullscreenPositions[agent.id] : undefined;
    const x = compact?.x ?? fullscreen?.x ?? agent.baseX;
    const y = compact?.y ?? fullscreen?.y ?? agent.baseY;
    return {
      x: (x / 100) * this.simWidth,
      y: (y / 100) * this.simHeight,
    };
  }

  private pointerPull(agent: AquariumAgentFrame, x: number, y: number) {
    const dx = this.pointer.x - x;
    const dy = this.pointer.y - y;
    const dist = Math.hypot(dx, dy);
    if (dist < 1 || dist > 130) return { x: 0, y: 0 };
    const force = (1 - dist / 130) * (12 + agent.activity * 34);
    return { x: (dx / dist) * force, y: (dy / dist) * force };
  }

  private nearestAgent(projected: ProjectedAgent[]) {
    if (!this.pointer.active) return null;
    let best: ProjectedAgent | null = null;
    let bestDist = 94;
    for (const agent of projected) {
      const dist = distance(agent.x, agent.y, this.pointer.x, this.pointer.y);
      if (dist < bestDist) {
        best = agent;
        bestDist = dist;
      }
    }
    return best;
  }

  private drawSource(projected: ProjectedAgent[], activeAgent: ProjectedAgent | undefined, time: number) {
    const ctx = this.sourceContext;
    ctx.clearRect(0, 0, this.simWidth, this.simHeight);
    this.hotOptions = [];
    ctx.save();
    ctx.globalCompositeOperation = "source-over";
    for (const agent of projected) {
      const hot = agent.id === activeAgent?.id || agent.id === this.frame.selectedAgentId;
      this.drawAgentSource(ctx, agent, hot);
    }
    if (activeAgent) {
      this.drawThoughtSource(ctx, activeAgent);
      this.drawOptionsSource(ctx, activeAgent);
    }
    this.drawParameterImpulseSource(ctx, time);
    ctx.restore();
  }

  private drawAgentSource(ctx: CanvasRenderingContext2D, agent: ProjectedAgent, hot: boolean) {
    const size = 22 + agent.activity * 10 + agent.chirps.expression * 4 + (hot ? 4 : 0);
    const ink = this.fluidParams.sourceOpacity * agent.emissionPulse * agent.chirps.inkPulse;
    const distortion = agent.chirps.distortion + (hot ? 0.02 : 0);
    ctx.save();
    ctx.translate(agent.x, agent.y);
    ctx.rotate(Math.atan2(agent.vy, agent.vx || 0.001) * 0.12);
    ctx.globalCompositeOperation = "lighter";
    ctx.fillStyle = agent.color;
    for (let index = 0; index < 4; index += 1) {
      const lag = 4 + index * 6;
      const layerSize = size * (1 + index * 0.18);
      const pulse =
        0.68 +
        agent.chirps.glowPulse * 0.14 +
        chirplet(this.time, agent.phase + index * 0.61, 0.28 + agent.chirps.expression * 0.24, 0.025, 6.5) * 0.24;
      ctx.save();
      ctx.translate(-agent.vx * lag - index * 1.5, -agent.vy * lag + index);
      ctx.globalAlpha = clamp((0.024 + agent.activity * 0.028 + (hot ? 0.015 : 0) - index * 0.003) * ink * pulse, 0, 0.42);
      drawDistortedAgentPath(ctx, agent.shape, layerSize, this.time * 0.18 + index * 0.9, agent.phase + index * 0.37, distortion + index * 0.018);
      ctx.fill();
      ctx.restore();
    }
    ctx.restore();
  }

  private drawThoughtSource(ctx: CanvasRenderingContext2D, agent: ProjectedAgent) {
    const boxWidth = Math.min(260, Math.max(160, this.simWidth * 0.23));
    const x = clamp(agent.x + (agent.x > this.simWidth * 0.68 ? -boxWidth - 36 : 36), 14, this.simWidth - boxWidth - 14);
    const y = clamp(agent.y - 78, 12, this.simHeight - 116);
    ctx.save();
    ctx.globalAlpha = 0.018 * this.fluidParams.sourceOpacity;
    ctx.strokeStyle = agent.glow;
    ctx.shadowColor = agent.glow;
    ctx.shadowBlur = 5;
    roundedRect(ctx, x, y, boxWidth, 82, 9);
    ctx.stroke();
    ctx.shadowBlur = 0;
    ctx.restore();
  }

  private drawOptionsSource(ctx: CanvasRenderingContext2D, agent: ProjectedAgent) {
    const options = agent.options ?? [];
    if (!options.length) return;
    const radius = this.simWidth < 540 ? 64 : 86;
    const arc = Math.min(Math.PI * 1.2, Math.max(Math.PI * 0.7, options.length * 0.34));
    const start = -Math.PI / 2 - arc / 2;
    ctx.save();
    ctx.font = "900 9px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    for (let index = 0; index < options.length; index += 1) {
      const option = options[index];
      const angle = start + (arc * (index + 0.5)) / options.length;
      const x = clamp(agent.x + Math.cos(angle) * radius, 42, this.simWidth - 42);
      const y = clamp(agent.y + Math.sin(angle) * radius, 48, this.simHeight - 48);
      const hot = this.pointer.active && distance(this.pointer.x, this.pointer.y, x, y) < 32;
      if (!option.disabled) {
        this.hotOptions.push({ x, y, radius: 32, key: option.key });
      }
      ctx.globalAlpha = (option.disabled ? 0.008 : hot ? 0.04 : 0.018) * this.fluidParams.sourceOpacity;
      ctx.fillStyle = option.disabled ? agent.glow : agent.color;
      ctx.strokeStyle = option.disabled ? hexAlpha(agent.glow, 0.12) : hexAlpha(agent.glow, hot ? 0.62 : 0.32);
      roundedRect(ctx, x - 37, y - 15, 74, 30, 15);
      ctx.fill();
      ctx.stroke();
    }
    ctx.restore();
  }

  private drawDeckSource(ctx: CanvasRenderingContext2D) {
    if (!this.frame.activeLabel) return;
    ctx.save();
    ctx.globalAlpha = 0.005 * this.fluidParams.sourceOpacity;
    ctx.fillStyle = "#f7bd58";
    ctx.font = `900 ${Math.max(30, Math.min(this.simWidth, this.simHeight) * 0.1)}px Inter, system-ui, sans-serif`;
    ctx.textAlign = "right";
    ctx.textBaseline = "bottom";
    ctx.fillText(this.frame.activeLabel.toUpperCase(), this.simWidth - 14, this.simHeight - 10);
    ctx.restore();
  }

  private drawOperatorSource(ctx: CanvasRenderingContext2D, time: number) {
    if (!this.frame.ui) return;
    const layout = this.operatorLayout(time);
    const ink = this.fluidParams.sourceOpacity;
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.strokeStyle = "rgba(247, 189, 88, 0.72)";
    ctx.fillStyle = "rgba(247, 189, 88, 0.22)";
    ctx.lineWidth = 1;
    ctx.globalAlpha = 0.012 * ink;
    roundedRect(ctx, layout.topbar.x, layout.topbar.y, layout.topbar.width, layout.topbar.height, 10);
    ctx.stroke();
    ctx.globalAlpha = 0.01 * ink;
    roundedRect(ctx, layout.panel.x, layout.panel.y, layout.panel.width, layout.panel.height, 10);
    ctx.stroke();
    for (const button of [...layout.deckButtons, ...layout.subdeckButtons, ...layout.actionButtons]) {
      ctx.globalAlpha = button.disabled ? 0.004 * ink : 0.011 * ink;
      roundedRect(ctx, button.x, button.y, button.width, button.height, button.height / 2);
      ctx.stroke();
    }
    ctx.restore();
  }

  private drawParameterImpulseSource(ctx: CanvasRenderingContext2D, time: number) {
    if (this.paramImpulse <= 0 || this.frame.agents.length === 0) return;
    const color = paramColorFor(this.lastFluidParamChanged);
    const pulse = this.paramImpulse;
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.strokeStyle = color;
    ctx.fillStyle = color;
    ctx.shadowColor = color;
    ctx.shadowBlur = 18 + pulse * 22;
    for (const [index, agent] of this.frame.agents.entries()) {
      const state = this.motion.get(agent.id);
      if (!state) continue;
      const wave = 0.65 + 0.35 * chirplet(time, index * 0.8, 2.6, 0.08, 2.8);
      ctx.globalAlpha = pulse * (0.03 + agent.activity * 0.016) * wave;
      ctx.beginPath();
      ctx.arc(state.x, state.y, 18 + pulse * 44 + index * 2, 0, Math.PI * 2);
      ctx.stroke();
      ctx.globalAlpha = pulse * (0.02 + agent.activity * 0.012);
      ctx.beginPath();
      ctx.arc(state.x - state.vx * 18, state.y - state.vy * 18, 8 + pulse * 20, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  private uploadSource() {
    const gl = this.gl;
    if (!this.sourceTexture) return;
    gl.bindTexture(gl.TEXTURE_2D, this.sourceTexture);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, this.sourceCanvas);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
  }

  private stepFluid(projected: ProjectedAgent[]) {
    if (!this.velocity || !this.dye || !this.pressure || !this.divergence || !this.curl || !this.sourceTexture) return;
    const dt = this.fluidParams.timeScale / fluidForceScales.length;
    const velocityDissipation = Math.pow(this.fluidParams.velocityDissipation, 1 / fluidForceScales.length);
    const dyeDissipation = Math.pow(this.fluidParams.dyeDissipation, 1 / fluidForceScales.length);
    for (const scale of fluidForceScales) {
      this.runAdvect(this.velocity.read.texture, this.velocity.read.texture, this.velocity.write, dt, velocityDissipation);
      this.velocity.swap();
      this.runVelocitySplat(projected, scale.radius, scale.force, scale.curl, velocityDissipation);
      this.runCurl();
      this.runVorticity(dt, scale.curl);
      this.runDivergence();
      this.runPressure(12);
      this.runGradientSubtract();
      this.runAdvect(this.velocity.read.texture, this.dye.read.texture, this.dye.write, dt, dyeDissipation);
      this.dye.swap();
      this.runInject(scale.inject);
    }
    this.paramImpulse = Math.max(0, this.paramImpulse * 0.9 - 0.01);
  }

  private runAdvect(velocity: WebGLTexture, source: WebGLTexture, target: FluidTarget, dt: number, dissipation: number) {
    this.drawTo(target, this.programs.advect, (gl, program) => {
      this.bindTexture(0, velocity);
      this.bindTexture(1, source);
      gl.uniform1i(gl.getUniformLocation(program, "uVelocity"), 0);
      gl.uniform1i(gl.getUniformLocation(program, "uSource"), 1);
      gl.uniform2f(gl.getUniformLocation(program, "uTexelSize"), 1 / this.simWidth, 1 / this.simHeight);
      gl.uniform1f(gl.getUniformLocation(program, "uDt"), dt);
      gl.uniform1f(gl.getUniformLocation(program, "uDissipation"), dissipation);
    });
  }

  private runVelocitySplat(projected: ProjectedAgent[], radiusScale = 1, forceScale = 1, swirlScale = 1, damping = this.fluidParams.velocityDissipation) {
    if (!this.velocity) return;
    this.agentsUniform.fill(0);
    this.activity.fill(0);
    for (let index = 0; index < Math.min(7, projected.length); index += 1) {
      const agent = projected[index];
      this.agentsUniform[index * 4] = agent.x / this.simWidth;
      this.agentsUniform[index * 4 + 1] = 1 - agent.y / this.simHeight;
      this.agentsUniform[index * 4 + 2] = agent.vx * 1.45;
      this.agentsUniform[index * 4 + 3] = -agent.vy * 1.45;
      this.activity[index] = clamp(agent.activity * 0.72 + agent.chirps.expression * 0.18 + agent.chirps.inkPulse * 0.1, 0, 1.6);
    }
    this.drawTo(this.velocity.write, this.programs.velocitySplat, (gl, program) => {
      this.bindTexture(0, this.velocity?.read.texture ?? null);
      gl.uniform1i(gl.getUniformLocation(program, "uVelocity"), 0);
      gl.uniform4fv(gl.getUniformLocation(program, "uAgents"), this.agentsUniform);
      gl.uniform1fv(gl.getUniformLocation(program, "uActivity"), this.activity);
      gl.uniform1i(gl.getUniformLocation(program, "uCount"), Math.min(7, projected.length));
      gl.uniform1f(gl.getUniformLocation(program, "uAspect"), this.simWidth / Math.max(this.simHeight, 1));
      gl.uniform1f(gl.getUniformLocation(program, "uSplatForce"), this.fluidParams.splatForce * forceScale * (1 + this.paramImpulse * 2.5));
      gl.uniform1f(gl.getUniformLocation(program, "uSplatRadius"), splatFalloff(this.fluidParams.splatRadius * radiusScale));
      gl.uniform1f(gl.getUniformLocation(program, "uSwirlForce"), this.fluidParams.swirlForce * swirlScale * (1 + this.paramImpulse * 1.4));
      gl.uniform1f(gl.getUniformLocation(program, "uVelocityDamping"), damping);
    });
    this.velocity.swap();
  }

  private runCurl() {
    if (!this.velocity || !this.curl) return;
    this.drawTo(this.curl, this.programs.curl, (gl, program) => {
      this.bindTexture(0, this.velocity?.read.texture ?? null);
      gl.uniform1i(gl.getUniformLocation(program, "uVelocity"), 0);
      gl.uniform2f(gl.getUniformLocation(program, "uTexelSize"), 1 / this.simWidth, 1 / this.simHeight);
    });
  }

  private runVorticity(dt: number, strengthScale = 1) {
    if (!this.velocity || !this.curl) return;
    this.drawTo(this.velocity.write, this.programs.vorticity, (gl, program) => {
      this.bindTexture(0, this.velocity?.read.texture ?? null);
      this.bindTexture(1, this.curl?.texture ?? null);
      gl.uniform1i(gl.getUniformLocation(program, "uVelocity"), 0);
      gl.uniform1i(gl.getUniformLocation(program, "uCurl"), 1);
      gl.uniform2f(gl.getUniformLocation(program, "uTexelSize"), 1 / this.simWidth, 1 / this.simHeight);
      gl.uniform1f(gl.getUniformLocation(program, "uCurlStrength"), this.fluidParams.curlStrength * strengthScale * (1 + this.paramImpulse * 1.8));
      gl.uniform1f(gl.getUniformLocation(program, "uDt"), dt);
    });
    this.velocity.swap();
  }

  private runDivergence() {
    if (!this.velocity || !this.divergence) return;
    this.drawTo(this.divergence, this.programs.divergence, (gl, program) => {
      this.bindTexture(0, this.velocity?.read.texture ?? null);
      gl.uniform1i(gl.getUniformLocation(program, "uVelocity"), 0);
      gl.uniform2f(gl.getUniformLocation(program, "uTexelSize"), 1 / this.simWidth, 1 / this.simHeight);
    });
  }

  private runPressure(iterations = 24) {
    if (!this.pressure || !this.divergence) return;
    for (let index = 0; index < iterations; index += 1) {
      this.drawTo(this.pressure.write, this.programs.pressure, (gl, program) => {
        this.bindTexture(0, this.pressure?.read.texture ?? null);
        this.bindTexture(1, this.divergence?.texture ?? null);
        gl.uniform1i(gl.getUniformLocation(program, "uPressure"), 0);
        gl.uniform1i(gl.getUniformLocation(program, "uDivergence"), 1);
        gl.uniform2f(gl.getUniformLocation(program, "uTexelSize"), 1 / this.simWidth, 1 / this.simHeight);
      });
      this.pressure.swap();
    }
  }

  private runGradientSubtract() {
    if (!this.velocity || !this.pressure) return;
    this.drawTo(this.velocity.write, this.programs.gradient, (gl, program) => {
      this.bindTexture(0, this.pressure?.read.texture ?? null);
      this.bindTexture(1, this.velocity?.read.texture ?? null);
      gl.uniform1i(gl.getUniformLocation(program, "uPressure"), 0);
      gl.uniform1i(gl.getUniformLocation(program, "uVelocity"), 1);
      gl.uniform2f(gl.getUniformLocation(program, "uTexelSize"), 1 / this.simWidth, 1 / this.simHeight);
    });
    this.velocity.swap();
  }

  private runInject(gainScale = 1) {
    if (!this.dye || !this.sourceTexture) return;
    this.drawTo(this.dye.write, this.programs.inject, (gl, program) => {
      this.bindTexture(0, this.dye?.read.texture ?? null);
      this.bindTexture(1, this.sourceTexture);
      gl.uniform1i(gl.getUniformLocation(program, "uDye"), 0);
      gl.uniform1i(gl.getUniformLocation(program, "uSource"), 1);
      gl.uniform1f(gl.getUniformLocation(program, "uGain"), this.fluidParams.injectionGain * gainScale * (1 + this.paramImpulse * 1.7));
      gl.uniform1f(gl.getUniformLocation(program, "uDissipation"), this.fluidParams.dyeDissipation);
    });
    this.dye.swap();
  }

  private display() {
    if (!this.dye) return;
    const gl = this.gl;
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    gl.useProgram(this.programs.display);
    this.bindTexture(0, this.dye.read.texture);
    gl.uniform1i(gl.getUniformLocation(this.programs.display, "uDye"), 0);
    gl.uniform2f(gl.getUniformLocation(this.programs.display, "uTexelSize"), 1 / this.simWidth, 1 / this.simHeight);
    gl.uniform1f(gl.getUniformLocation(this.programs.display, "uExposure"), this.fluidParams.acesExposure);
    gl.uniform1f(gl.getUniformLocation(this.programs.display, "uGlow"), this.fluidParams.acesGlow);
    gl.uniform1f(gl.getUniformLocation(this.programs.display, "uSaturation"), this.fluidParams.acesSaturation);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }

  private drawCrispOverlay(projected: ProjectedAgent[], activeAgent: ProjectedAgent | undefined, time: number) {
    if (!this.crispCanvas || !this.crispContext || !this.simWidth || !this.simHeight) return;
    const ctx = this.crispContext;
    const scaleX = this.crispCanvas.width / this.simWidth;
    const scaleY = this.crispCanvas.height / this.simHeight;
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, this.crispCanvas.width, this.crispCanvas.height);
    ctx.scale(scaleX, scaleY);
    if (this.frame.variant !== "fullscreen") {
      for (const agent of projected) {
        const hot = agent.id === activeAgent?.id || agent.id === this.frame.selectedAgentId;
        this.drawCrispAgent(ctx, agent, hot);
      }
      if (activeAgent) {
        this.drawCrispThought(ctx, activeAgent);
        this.drawCrispOptions(ctx, activeAgent);
      }
      this.drawCrispDeck(ctx);
      this.drawOperatorUi(ctx, time);
    }
    this.drawFluidPanel(ctx, time);
    ctx.restore();
  }

  private operatorLayout(time: number) {
    const mobile = this.simWidth < 430;
    const wobble = chirplet(time, 0.9, 1.2, 0.08, 4);
    const topbar = {
      x: mobile ? 10 : 14,
      y: 12 + wobble * 1.8,
      width: this.simWidth - (mobile ? 20 : 28),
      height: mobile ? 154 : 68,
    };
    const deckWidth = mobile ? this.simWidth - 20 : 78;
    const deckX = mobile ? 10 : 14;
    const deckY = mobile ? topbar.y + topbar.height + 8 : topbar.y + topbar.height + 22;
    const deckButtons = (this.frame.ui?.deckButtons ?? []).map((button, index) => ({
      ...button,
      x: mobile ? deckX + index * ((deckWidth - 6) / 4) : deckX,
      y: mobile ? deckY : deckY + index * 46,
      width: mobile ? (deckWidth - 18) / 4 : deckWidth,
      height: mobile ? 36 : 38,
    }));
    const panel = {
      x: mobile ? 10 : deckX + deckWidth + 14,
      y: mobile ? deckY + 48 : deckY,
      width: mobile ? this.simWidth - 20 : Math.min(360, this.simWidth * 0.48),
      height: mobile ? Math.min(370, this.simHeight - deckY - 60) : Math.min(384, this.simHeight - deckY - 22),
    };
    let subdeckX = panel.x + 16;
    const subdeckButtons = (this.frame.ui?.subdeckButtons ?? []).map((button) => {
      const width = Math.min(98, Math.max(62, button.label.length * 6.2 + 22));
      const positioned = {
        ...button,
        x: subdeckX,
        y: panel.y + 48,
        width,
        height: 24,
      };
      subdeckX += width + 8;
      return positioned;
    });
    const actionButtons = (this.frame.ui?.actionButtons ?? []).map((button, index) => {
      const columns = mobile ? 1 : 2;
      const gap = 8;
      const width = (panel.width - 32 - gap * (columns - 1)) / columns;
      const x = panel.x + 16 + (index % columns) * (width + gap);
      const y = panel.y + 92 + Math.floor(index / columns) * 32;
      return { ...button, x, y, width, height: 26 };
    });
    return { actionButtons, deckButtons, panel, subdeckButtons, topbar };
  }

  private drawOperatorUi(ctx: CanvasRenderingContext2D, time: number) {
    const ui = this.frame.ui;
    if (!ui) return;
    const layout = this.operatorLayout(time);
    const bounce = chirplet(time, 2.7, 1.4, -0.06, 5);
    const mobile = this.simWidth < 430;
    ctx.save();
    ctx.textBaseline = "top";
    ctx.lineJoin = "round";

    ctx.globalAlpha = mobile ? 0.96 : 0.82;
    ctx.fillStyle = mobile ? "rgba(5, 12, 9, 0.9)" : "rgba(5, 12, 9, 0.72)";
    ctx.strokeStyle = "rgba(219, 238, 216, 0.34)";
    ctx.shadowColor = "rgba(146, 216, 118, 0.28)";
    ctx.shadowBlur = 16;
    roundedRect(ctx, layout.topbar.x, layout.topbar.y, layout.topbar.width, layout.topbar.height, 10);
    ctx.fill();
    ctx.stroke();
    ctx.shadowBlur = 0;
    ctx.fillStyle = "#b9d8b5";
    ctx.font = "900 8px Inter, system-ui, sans-serif";
    ctx.fillText(ui.eyebrow.toUpperCase(), layout.topbar.x + 14, layout.topbar.y + 10);
    ctx.fillStyle = "#fbfff8";
    ctx.font = `${mobile ? 20 : 24}px Inter, system-ui, sans-serif`;
    ctx.fillText(ui.title, layout.topbar.x + 14, layout.topbar.y + 24);
    ctx.fillStyle = "rgba(226, 245, 225, 0.78)";
    ctx.font = "800 9px Inter, system-ui, sans-serif";
    wrapCanvasText(ctx, ui.reason, layout.topbar.x + 14, layout.topbar.y + (mobile ? 51 : 51), mobile ? layout.topbar.width - 28 : layout.topbar.width * 0.58, 12, mobile ? 2 : 1);

    const pillY = mobile ? layout.topbar.y + 92 : layout.topbar.y + 14;
    ui.statuses.slice(0, mobile ? 2 : 3).forEach((status, index) => {
      const width = mobile ? layout.topbar.width - 28 : Math.min(112, Math.max(62, status.label.length * 5.8 + 20));
      const x = mobile
        ? layout.topbar.x + 14
        : layout.topbar.x + layout.topbar.width - width - 12 - index * (width + 8);
      const y = mobile ? pillY + index * 26 : pillY;
      this.drawCrispButton(ctx, { key: "", label: status.label, disabled: false, tone: status.tone, x, y, width, height: 24 }, false, time + index);
    });

    for (const [index, button] of layout.deckButtons.entries()) {
      const hot = this.pointer.active && hitZone({ ...button, radius: 0, key: button.key }, this.pointer.x, this.pointer.y);
      this.hotOptions.push({ x: button.x, y: button.y, width: button.width, height: button.height, radius: 0, key: button.key });
      this.drawCrispButton(ctx, button, hot, time + index * 0.7);
    }

    ctx.globalAlpha = mobile ? 0.94 : 0.82;
    ctx.fillStyle = mobile ? "rgba(5, 12, 9, 0.88)" : "rgba(5, 12, 9, 0.76)";
    ctx.strokeStyle = "rgba(219, 238, 216, 0.32)";
    ctx.shadowColor = "rgba(88, 221, 196, 0.18)";
    ctx.shadowBlur = 18;
    roundedRect(ctx, layout.panel.x, layout.panel.y + bounce * 2.4, layout.panel.width, layout.panel.height, 10);
    ctx.fill();
    ctx.stroke();
    ctx.shadowBlur = 0;

    ctx.fillStyle = "#b9d8b5";
    ctx.font = "900 8px Inter, system-ui, sans-serif";
    ctx.fillText(ui.activeDeckLabel.toUpperCase(), layout.panel.x + 16, layout.panel.y + 15 + bounce * 2.4);
    ctx.fillStyle = "#fbfff8";
    ctx.font = "900 16px Inter, system-ui, sans-serif";
    ctx.fillText(ui.activeSubdeck, layout.panel.x + 16, layout.panel.y + 28 + bounce * 2.4);

    for (const [index, button] of layout.subdeckButtons.entries()) {
      const hot = this.pointer.active && hitZone({ ...button, radius: 0, key: button.key }, this.pointer.x, this.pointer.y);
      this.hotOptions.push({ x: button.x, y: button.y, width: button.width, height: button.height, radius: 0, key: button.key });
      this.drawCrispButton(ctx, button, hot, time + index * 0.45);
    }

    if (layout.actionButtons.length > 0) {
      for (const [index, button] of layout.actionButtons.entries()) {
        const hot = this.pointer.active && hitZone({ ...button, radius: 0, key: button.key }, this.pointer.x, this.pointer.y);
        this.hotOptions.push({ x: button.x, y: button.y, width: button.width, height: button.height, radius: 0, key: button.key });
        this.drawCrispButton(ctx, button, hot, time + index * 0.32);
      }
    } else {
      ctx.fillStyle = "rgba(226, 245, 225, 0.8)";
      ctx.font = "800 10px Inter, system-ui, sans-serif";
      ui.panelLines.slice(0, 10).forEach((line, index) => {
        wrapCanvasText(ctx, line, layout.panel.x + 16, layout.panel.y + 88 + index * 25, layout.panel.width - 32, 13, 2);
      });
    }

    if (layout.actionButtons.length > 0 && ui.panelLines.length > 0) {
      ctx.fillStyle = "rgba(226, 245, 225, 0.72)";
      ctx.font = "800 9px Inter, system-ui, sans-serif";
      const startY = layout.panel.y + 92 + Math.ceil(layout.actionButtons.length / (this.simWidth < 430 ? 1 : 2)) * 32 + 8;
      ui.panelLines.slice(0, 3).forEach((line, index) => {
        wrapCanvasText(ctx, line, layout.panel.x + 16, startY + index * 18, layout.panel.width - 32, 12, 1);
      });
    }

    if (ui.alert && !mobile) {
      ctx.globalAlpha = 0.84;
      ctx.fillStyle = "rgba(255, 244, 203, 0.86)";
      ctx.strokeStyle = "rgba(247, 189, 88, 0.6)";
      const alertWidth = Math.min(300, layout.topbar.width * 0.42);
      const alertX = layout.topbar.x + layout.topbar.width - alertWidth - 12;
      const alertY = layout.topbar.y + layout.topbar.height + 10;
      roundedRect(ctx, alertX, alertY, alertWidth, 30, 7);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = "#70410f";
      ctx.font = "900 9px Inter, system-ui, sans-serif";
      wrapCanvasText(ctx, ui.alert, alertX + 10, alertY + 9, alertWidth - 20, 11, 1);
    }

    ctx.restore();
  }

  private drawCrispButton(ctx: CanvasRenderingContext2D, button: AquariumUiButtonFrame & { x: number; y: number; width: number; height: number }, hot: boolean, time: number) {
    const pulse = chirplet(time, 0.5, 1.8, 0.09, 3);
    const disabled = Boolean(button.disabled);
    const toneColor = toneColorFor(button.tone);
    ctx.save();
    ctx.translate(0, hot && !disabled ? pulse * 1.8 : 0);
    ctx.globalAlpha = disabled ? 0.36 : hot ? 0.94 : 0.78;
    ctx.fillStyle = hot && !disabled ? hexAlpha(toneColor, 0.32) : "rgba(8, 14, 12, 0.78)";
    ctx.strokeStyle = disabled ? "rgba(226, 245, 225, 0.16)" : hexAlpha(toneColor, hot ? 0.9 : 0.5);
    roundedRect(ctx, button.x, button.y, button.width, button.height, Math.min(16, button.height / 2));
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = disabled ? "rgba(236, 246, 235, 0.42)" : "#fbfff8";
    ctx.font = `900 ${button.width < 70 ? 7 : 9}px Inter, system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(button.label.toUpperCase().slice(0, button.width < 70 ? 9 : 18), button.x + button.width / 2, button.y + button.height / 2 + 0.5);
    ctx.restore();
  }

  private drawCrispAgent(ctx: CanvasRenderingContext2D, agent: ProjectedAgent, hot: boolean) {
    const glowPulse = agent.chirps.glowPulse;
    const size = 23 + agent.activity * 10 + agent.chirps.expression * 4 + (hot ? 5 : 0);
    ctx.save();
    ctx.translate(agent.x, agent.y);
    ctx.rotate(Math.atan2(agent.vy, agent.vx || 0.001) * 0.14);
    ctx.globalAlpha = clamp((hot ? 0.88 : 0.68) + glowPulse * 0.08, 0.4, 0.98);
    ctx.shadowColor = agent.glow;
    ctx.shadowBlur = (hot ? 16 : 8) * glowPulse;
    ctx.fillStyle = agent.color;
    drawDistortedAgentPath(ctx, agent.shape, size, this.time * 0.18, agent.phase, agent.chirps.distortion * 0.62 + (hot ? 0.014 : 0));
    ctx.fill();
    ctx.shadowBlur = 0;
    ctx.strokeStyle = hot ? hexAlpha("#ffffff", clamp(0.62 + glowPulse * 0.14, 0, 0.92)) : hexAlpha("#ffffff", clamp(0.38 + glowPulse * 0.1, 0, 0.68));
    ctx.lineWidth = (hot ? 1.7 : 1) + glowPulse * 0.22;
    ctx.stroke();
    ctx.fillStyle = "#fffaf0";
    ctx.font = `900 ${Math.max(10, size * 0.43)}px Inter, system-ui, sans-serif`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(agent.glyph, 0, 1);
    ctx.restore();

    ctx.save();
    ctx.globalAlpha = hot ? 0.82 : 0.62;
    ctx.fillStyle = "rgba(5, 12, 9, 0.72)";
    ctx.strokeStyle = hexAlpha(agent.color, hot ? 0.76 : 0.42);
    roundedRect(ctx, agent.x - 42, agent.y + size * 0.74, 84, 34, 7);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "rgba(247, 255, 247, 0.94)";
    ctx.font = "900 10px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(agent.name, agent.x, agent.y + size * 0.74 + 13);
    ctx.fillStyle = "rgba(226, 245, 225, 0.74)";
    ctx.font = "900 8px Inter, system-ui, sans-serif";
    ctx.fillText(agent.status.slice(0, 16).toUpperCase(), agent.x, agent.y + size * 0.74 + 26);
    ctx.restore();
  }

  private drawCrispThought(ctx: CanvasRenderingContext2D, agent: ProjectedAgent) {
    const boxWidth = Math.min(260, Math.max(160, this.simWidth * 0.23));
    const x = clamp(agent.x + (agent.x > this.simWidth * 0.68 ? -boxWidth - 36 : 36), 14, this.simWidth - boxWidth - 14);
    const y = clamp(agent.y - 78, 12, this.simHeight - 116);
    ctx.save();
    ctx.globalAlpha = 0.78;
    ctx.fillStyle = "rgba(248, 252, 242, 0.82)";
    ctx.strokeStyle = hexAlpha(agent.color, 0.72);
    ctx.shadowColor = agent.glow;
    ctx.shadowBlur = 12;
    roundedRect(ctx, x, y, boxWidth, 82, 9);
    ctx.fill();
    ctx.stroke();
    ctx.shadowBlur = 0;
    ctx.fillStyle = agent.color;
    ctx.font = "900 10px Inter, system-ui, sans-serif";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText(agent.name.toUpperCase(), x + 10, y + 10);
    ctx.fillStyle = "#172018";
    ctx.font = "800 12px Inter, system-ui, sans-serif";
    wrapCanvasText(ctx, agent.thought, x + 10, y + 27, boxWidth - 20, 15, 3);
    ctx.restore();
  }

  private drawCrispOptions(ctx: CanvasRenderingContext2D, agent: ProjectedAgent) {
    const options = agent.options ?? [];
    if (!options.length) return;
    const radius = this.simWidth < 540 ? 64 : 86;
    const arc = Math.min(Math.PI * 1.2, Math.max(Math.PI * 0.7, options.length * 0.34));
    const start = -Math.PI / 2 - arc / 2;
    ctx.save();
    ctx.font = "900 9px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    for (let index = 0; index < options.length; index += 1) {
      const option = options[index];
      const angle = start + (arc * (index + 0.5)) / options.length;
      const x = clamp(agent.x + Math.cos(angle) * radius, 42, this.simWidth - 42);
      const y = clamp(agent.y + Math.sin(angle) * radius, 48, this.simHeight - 48);
      const hot = this.pointer.active && distance(this.pointer.x, this.pointer.y, x, y) < 32;
      ctx.globalAlpha = option.disabled ? 0.34 : hot ? 0.9 : 0.72;
      ctx.fillStyle = option.disabled ? "rgba(8, 14, 12, 0.68)" : hot ? agent.color : "rgba(8, 14, 12, 0.78)";
      ctx.strokeStyle = option.disabled ? "rgba(226, 245, 225, 0.22)" : hexAlpha(agent.glow, hot ? 0.9 : 0.58);
      roundedRect(ctx, x - 37, y - 15, 74, 30, 15);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = option.disabled ? "rgba(236, 246, 235, 0.42)" : "#ffffff";
      ctx.fillText(option.label.toUpperCase(), x, y + 1);
    }
    ctx.restore();
  }

  private drawCrispDeck(ctx: CanvasRenderingContext2D) {
    if (!this.frame.activeLabel) return;
    ctx.save();
    ctx.globalAlpha = 0.16;
    ctx.fillStyle = "#f7bd58";
    ctx.font = `900 ${Math.max(30, Math.min(this.simWidth, this.simHeight) * 0.1)}px Inter, system-ui, sans-serif`;
    ctx.textAlign = "right";
    ctx.textBaseline = "bottom";
    ctx.fillText(this.frame.activeLabel.toUpperCase(), this.simWidth - 14, this.simHeight - 10);
    ctx.restore();
  }

  private drawFluidPanel(ctx: CanvasRenderingContext2D, time: number) {
    this.fluidParamZones = [];
    const iconSize = 32;
    const inspectorGuard = this.frame.variant === "fullscreen" && this.simWidth >= 720
      ? Math.min(230, this.simHeight * 0.25)
      : 0;
    const iconX = this.simWidth - iconSize - 16;
    const iconY = this.simHeight - iconSize - 16 - inspectorGuard;
    const iconZone: FluidParamZone = { key: "toggle", x: iconX, y: iconY, width: iconSize, height: iconSize };
    this.fluidParamZones.push(iconZone);
    const nearIcon = this.pointer.active && pointInInflatedRect(this.pointer.x, this.pointer.y, iconZone, 54);
    const open = this.fluidPanelPinned || nearIcon || this.draggingFluidParam !== null;
    const pulse = chirplet(time, 1.8, 2.4, 0.18, 2.8);

    ctx.save();
    ctx.globalAlpha = open ? 0.94 : 0.66;
    ctx.fillStyle = open ? "rgba(247, 189, 88, 0.24)" : "rgba(8, 14, 12, 0.76)";
    ctx.strokeStyle = "rgba(247, 189, 88, 0.76)";
    ctx.shadowColor = "rgba(247, 189, 88, 0.42)";
    ctx.shadowBlur = open ? 18 : 10;
    roundedRect(ctx, iconX + pulse * 1.2, iconY - pulse * 1.2, iconSize, iconSize, 11);
    ctx.fill();
    ctx.stroke();
    ctx.shadowBlur = 0;
    ctx.fillStyle = "#fff4cb";
    ctx.font = "900 16px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText("≈", iconX + iconSize / 2 + pulse * 1.2, iconY + iconSize / 2 - pulse * 1.2);

    if (!open) {
      ctx.restore();
      return;
    }

    const panelWidth = Math.min(318, this.simWidth - 24);
    const panelHeight = Math.min(426, this.simHeight - 48);
    const panelX = clamp(iconX - panelWidth + iconSize, 12, this.simWidth - panelWidth - 12);
    const panelY = clamp(iconY - panelHeight - 12 + chirplet(time, 0.2, 1.15, -0.05, 4) * 4, 12, this.simHeight - panelHeight - 12);
    ctx.globalAlpha = 0.9;
    ctx.fillStyle = "rgba(5, 12, 9, 0.82)";
    ctx.strokeStyle = "rgba(247, 189, 88, 0.48)";
    ctx.shadowColor = "rgba(88, 221, 196, 0.22)";
    ctx.shadowBlur = 20;
    roundedRect(ctx, panelX, panelY, panelWidth, panelHeight, 10);
    ctx.fill();
    ctx.stroke();
    ctx.shadowBlur = 0;
    ctx.fillStyle = "#f7bd58";
    ctx.font = "900 9px Inter, system-ui, sans-serif";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    ctx.fillText("FLUID PARAMETERS", panelX + 14, panelY + 12);
    ctx.fillStyle = "rgba(226, 245, 225, 0.76)";
    ctx.font = "800 9px Inter, system-ui, sans-serif";
    ctx.fillText(this.fluidPanelPinned ? "pinned; drag rails" : "hover; click icon to pin", panelX + 14, panelY + 27);

    const rowGap = panelHeight < 390 ? 23 : 27;
    const railX = panelX + 122;
    const railWidth = panelWidth - 142;
    fluidParamDefinitions.forEach((definition, index) => {
      const y = panelY + 52 + index * rowGap;
      const value = this.fluidParams[definition.key];
      const t = fluidParamToUnit(definition, value);
      const hot = this.pointer.active && this.pointer.x >= railX - 4 && this.pointer.x <= railX + railWidth + 4 && this.pointer.y >= y - 5 && this.pointer.y <= y + 13;
      this.fluidParamZones.push({ key: definition.key, x: railX - 5, y: y - 6, width: railWidth + 10, height: 18 });
      ctx.fillStyle = hot ? "#fbfff8" : "rgba(226, 245, 225, 0.82)";
      ctx.font = "900 8px Inter, system-ui, sans-serif";
      ctx.fillText(definition.label.toUpperCase(), panelX + 14, y - 4);
      ctx.fillStyle = "rgba(226, 245, 225, 0.42)";
      ctx.fillRect(railX, y + 3, railWidth, 3);
      ctx.fillStyle = hot ? "#58ddc4" : "#f7bd58";
      ctx.fillRect(railX, y + 3, railWidth * t, 3);
      ctx.beginPath();
      ctx.arc(railX + railWidth * t, y + 4.5 + chirplet(time, index, 2.1, 0.03, 3) * 0.8, hot ? 5 : 3.6, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = "rgba(251, 255, 248, 0.76)";
      ctx.font = "800 8px Inter, system-ui, sans-serif";
      ctx.textAlign = "right";
      ctx.fillText(value.toFixed(definition.decimals), panelX + panelWidth - 14, y - 4);
      ctx.textAlign = "left";
    });

    const resetZone: FluidParamZone = { key: "reset", x: panelX + panelWidth - 82, y: panelY + panelHeight - 34, width: 66, height: 22 };
    this.fluidParamZones.push(resetZone);
    const resetHot = this.pointer.active && pointInRect(this.pointer.x, this.pointer.y, resetZone);
    ctx.globalAlpha = resetHot ? 0.94 : 0.72;
    ctx.fillStyle = resetHot ? "rgba(241, 95, 69, 0.26)" : "rgba(8, 14, 12, 0.74)";
    ctx.strokeStyle = "rgba(241, 95, 69, 0.56)";
    roundedRect(ctx, resetZone.x, resetZone.y, resetZone.width, resetZone.height, 11);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "#fbfff8";
    ctx.font = "900 8px Inter, system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText("RESET", resetZone.x + resetZone.width / 2, resetZone.y + resetZone.height / 2);
    ctx.restore();
  }

  private updateFluidParamFromPointer(key: FluidParamKey, pointerX: number) {
    const zone = this.fluidParamZones.find((candidate) => candidate.key === key);
    const definition = fluidParamDefinitions.find((candidate) => candidate.key === key);
    if (!zone || !definition) return;
    const t = clamp((pointerX - zone.x) / Math.max(zone.width, 1), 0, 1);
    this.fluidParams = {
      ...this.fluidParams,
      [key]: fluidParamFromUnit(definition, t),
    };
    this.paramImpulse = Math.max(this.paramImpulse, 1);
    this.lastFluidParamChanged = key;
    saveFluidParams(this.fluidParams);
  }

  private drawTo(target: FluidTarget, program: WebGLProgram, uniforms: (gl: WebGL2RenderingContext, program: WebGLProgram) => void) {
    const gl = this.gl;
    gl.bindFramebuffer(gl.FRAMEBUFFER, target.fbo);
    gl.viewport(0, 0, target.width, target.height);
    gl.useProgram(program);
    uniforms(gl, program);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }

  private bindTexture(unit: number, texture: WebGLTexture | null) {
    if (!texture) return;
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, texture);
  }
}

class CanvasAquariumRenderer implements AquariumRenderer {
  private frame: AquariumFrame = { agents: [], selectedAgentId: "coordinator", variant: "fullscreen" };
  private hotAgents: HotZone[] = [];
  private pointer = { active: false, x: 0, y: 0 };
  private raf = 0;

  constructor(private canvas: HTMLCanvasElement, private crispCanvas: HTMLCanvasElement | null = null) {
    this.raf = requestAnimationFrame(this.render);
  }

  clearPointer() {
    this.pointer = { active: false, x: 0, y: 0 };
  }

  dispose() {
    cancelAnimationFrame(this.raf);
  }

  pickAgent() {
    return this.hotAgents.find((zone) => hitZone(zone, this.pointer.x, this.pointer.y))?.key ?? null;
  }

  pickOption() {
    return null;
  }

  setFrame(frame: AquariumFrame) {
    this.frame = frame;
  }

  setPointerClient(clientX: number, clientY: number) {
    const rect = this.canvas.getBoundingClientRect();
    this.pointer = {
      active: true,
      x: ((clientX - rect.left) / Math.max(rect.width, 1)) * this.canvas.width,
      y: ((clientY - rect.top) / Math.max(rect.height, 1)) * this.canvas.height,
    };
  }

  pointerDownClient(clientX: number, clientY: number) {
    this.setPointerClient(clientX, clientY);
  }

  pointerUp() {
    // The 2D fallback has no draggable fluid parameters.
  }

  private render = (millis: number) => {
    const rect = this.canvas.getBoundingClientRect();
    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    this.canvas.width = Math.max(1, Math.floor(rect.width * dpr));
    this.canvas.height = Math.max(1, Math.floor(rect.height * dpr));
    const ctx = this.canvas.getContext("2d");
    if (!ctx) return;
    ctx.fillStyle = "#07110e";
    ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    this.hotAgents = [];
    const time = millis / 1000;
    this.frame.agents.forEach((agent) => {
      const position = fullscreenPositions[agent.id] ?? { x: agent.baseX, y: agent.baseY };
      const x = (position.x / 100) * this.canvas.width + Math.sin(time * 0.2 + agent.phase) * 22 * agent.activity;
      const y = (position.y / 100) * this.canvas.height + Math.cos(time * 0.18 + agent.phase) * 16 * agent.activity;
      this.hotAgents.push({ x, y, radius: 60, key: agent.id });
      ctx.globalAlpha = 0.07 + agent.activity * 0.045;
      ctx.fillStyle = agent.color;
      ctx.beginPath();
      ctx.arc(x, y, 26 + agent.activity * 12, 0, Math.PI * 2);
      ctx.fill();
      ctx.globalAlpha = 0.24;
      ctx.fillStyle = "#ffffff";
      ctx.font = "900 12px Inter, system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(agent.name, x, y + 46);
    });
    this.raf = requestAnimationFrame(this.render);
  };
}

function compileProgram(gl: WebGL2RenderingContext, vertex: string, fragment: string) {
  const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertex);
  const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragment);
  const program = gl.createProgram();
  if (!program) throw new Error("WebGL program creation failed.");
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);
  gl.deleteShader(vertexShader);
  gl.deleteShader(fragmentShader);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const info = gl.getProgramInfoLog(program);
    gl.deleteProgram(program);
    throw new Error(`WebGL program link failed: ${info}`);
  }
  return program;
}

function compileShader(gl: WebGL2RenderingContext, type: number, source: string) {
  const shader = gl.createShader(type);
  if (!shader) throw new Error("WebGL shader creation failed.");
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const info = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw new Error(`WebGL shader compile failed: ${info}`);
  }
  return shader;
}

function hitZone(zone: HotZone, x: number, y: number) {
  if (zone.width !== undefined && zone.height !== undefined) {
    return x >= zone.x && x <= zone.x + zone.width && y >= zone.y && y <= zone.y + zone.height;
  }
  return distance(zone.x, zone.y, x, y) <= zone.radius;
}

function pointInRect(x: number, y: number, rect: FluidParamZone) {
  return x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height;
}

function pointInInflatedRect(x: number, y: number, rect: FluidParamZone, inflate: number) {
  return x >= rect.x - inflate && x <= rect.x + rect.width + inflate && y >= rect.y - inflate && y <= rect.y + rect.height + inflate;
}

function personalityFor(id: string) {
  return agentPersonalities[id] ?? agentPersonalities.coordinator;
}

function projectAgentState(agent: AquariumAgentFrame): AgentStateVector {
  const status = agent.status.toLowerCase();
  const activity = clamp(agent.activity, 0, 1);
  const blocked = statusSignal(status, ["blocked", "missing", "failed", "error", "unknown", "needed", "regather"]);
  const ready = statusSignal(status, ["ready", "ok", "clear", "completed", "captured", "continue"]);
  const review = statusSignal(status, ["review", "accept", "findings", "patch", "required"]);
  const urgency = clamp(
    activity * 0.62 +
      blocked * 0.28 +
      review * 0.2 +
      statusSignal(status, ["critical", "high", "prepare", "launch", "running", "active"]) * 0.26,
    0,
    1,
  );
  return { activity, blocked, ready, review, urgency };
}

function statusSignal(status: string, needles: string[]) {
  return needles.some((needle) => status.includes(needle)) ? 1 : 0;
}

function projectChirpMatrix(
  agent: AquariumAgentFrame,
  personality: AgentPersonality,
  state: AgentStateVector,
  time: number,
  hover: number,
): AgentChirpMatrix {
  const hoverAmount = hover * personality.hoverStillness;
  const hoverFrequency = lerp(1, 0.18, hoverAmount);
  const hoverAmplitude = lerp(1, 0.12, hoverAmount);
  const heat = clamp(state.activity * 0.52 + state.urgency * 0.3 + state.review * 0.16 + state.blocked * 0.2, 0, 1);
  const expressiveGain = (0.58 + personality.expressiveness * 0.72 + heat * 0.58) * hoverAmplitude;
  const radial = chirplet(
    time,
    agent.phase + personality.angle * 0.41,
    personality.radialTempo * hoverFrequency * (1 + heat * 0.55),
    0.012 + personality.expressiveness * 0.02 + state.urgency * 0.018,
    8.5 - personality.expressiveness * 1.3,
  );
  const tangential = chirplet(
    time,
    agent.phase * 1.73 + personality.angle,
    personality.tangentialTempo * hoverFrequency * (1 + state.activity * 0.7),
    -0.016 - personality.precision * 0.012,
    7.2 + personality.precision * 1.8,
  );
  const flicker = chirplet(
    time,
    agent.phase * 2.37 + state.review * 0.9,
    personality.glowTempo * hoverFrequency * (1.05 + heat),
    0.03 + state.blocked * 0.04,
    4.8 - heat * 1.4,
  );
  const ink = chirplet(
    time,
    agent.phase * 2.91 + state.urgency,
    personality.inkTempo * hoverFrequency * (0.95 + state.activity * 1.1),
    -0.018 + personality.expressiveness * 0.025,
    6.4 - personality.expressiveness * 1.1,
  );
  const orbitDrift = time * personality.orbitSpeed * hoverFrequency * (0.75 + state.activity * 0.65 + state.urgency * 0.34);
  return {
    angle: personality.angle + orbitDrift + tangential * 0.18 * expressiveGain + state.blocked * 0.08 - state.review * 0.06,
    distortion: clamp(0.035 + personality.expressiveness * 0.055 + heat * 0.045 + Math.abs(flicker) * 0.026, 0.018, 0.18),
    expression: clamp(expressiveGain, 0.12, 1.9),
    glowPulse: clamp(0.62 + Math.abs(flicker) * 0.55 + heat * 0.46 + state.review * 0.18 + state.ready * 0.08, 0.34, 2.15),
    hoverDamping: lerp(1, 0.34, hoverAmount),
    inkPulse: clamp(0.38 + Math.abs(ink) * 0.72 + heat * 0.5 + state.blocked * 0.1, 0.06, 2.2),
    orbitRadius: clamp(personality.radius * lerp(1.04, 0.82, state.blocked) * lerp(1, 1.1, state.ready), 0, 1.1),
    radial: radial * expressiveGain,
    tangential: tangential * expressiveGain,
  };
}

function chirplet(time: number, phase: number, frequency: number, chirp: number, period: number) {
  const local = ((time + phase) % period + period) % period;
  const centered = local - period / 2;
  const envelope = 0.28 + 0.72 * Math.exp(-(centered * centered) / (period * period * 0.18));
  return Math.sin(phase + frequency * local + chirp * local * local) * envelope;
}

function toneColorFor(tone?: string) {
  if (tone === "danger") return "#f15f45";
  if (tone === "warn") return "#f7bd58";
  if (tone === "ok") return "#58ddc4";
  return "#92d876";
}

function paramColorFor(key: FluidParamKey | null) {
  if (key === "curlStrength" || key === "swirlForce") return "#9f6ee7";
  if (key === "splatForce" || key === "splatRadius") return "#58ddc4";
  if (key === "injectionGain" || key === "sourceOpacity") return "#f7bd58";
  if (key === "acesExposure" || key === "acesGlow" || key === "acesSaturation") return "#fbfff8";
  if (key === "dyeDissipation") return "#92d876";
  if (key === "velocityDissipation") return "#63c5da";
  return "#f15f45";
}

function splatFalloff(radius: number) {
  return clamp(74088 / Math.max(radius * radius, 1), 3, 1200);
}

function fluidParamToUnit(definition: FluidParamDefinition, value: number) {
  const clamped = clamp(value, definition.min, definition.max);
  if (definition.scale === "log") {
    const min = Math.max(definition.min, Number.EPSILON);
    const max = Math.max(definition.max, min * 1.0001);
    return clamp(Math.log(clamped / min) / Math.log(max / min), 0, 1);
  }
  if (definition.scale === "softLog") {
    if (clamped <= definition.min) return 0;
    const min = softLogMin(definition);
    const max = Math.max(definition.max, min * 1.0001);
    return clamp(Math.log(Math.max(clamped, min) / min) / Math.log(max / min), 0, 1);
  }
  if (definition.scale === "persistenceLog") {
    const maxLoss = Math.max(1 - definition.min, Number.EPSILON);
    const minLoss = Math.max(1 - definition.max, Number.EPSILON);
    const loss = clamp(1 - clamped, minLoss, maxLoss);
    return clamp(Math.log(maxLoss / loss) / Math.log(maxLoss / minLoss), 0, 1);
  }
  return clamp((clamped - definition.min) / Math.max(definition.max - definition.min, Number.EPSILON), 0, 1);
}

function fluidParamFromUnit(definition: FluidParamDefinition, unit: number) {
  const t = clamp(unit, 0, 1);
  if (definition.scale === "log") {
    const min = Math.max(definition.min, Number.EPSILON);
    const max = Math.max(definition.max, min * 1.0001);
    return min * Math.pow(max / min, t);
  }
  if (definition.scale === "softLog") {
    if (t <= 0) return definition.min;
    const min = softLogMin(definition);
    const max = Math.max(definition.max, min * 1.0001);
    return min * Math.pow(max / min, t);
  }
  if (definition.scale === "persistenceLog") {
    const maxLoss = Math.max(1 - definition.min, Number.EPSILON);
    const minLoss = Math.max(1 - definition.max, Number.EPSILON);
    const loss = maxLoss * Math.pow(minLoss / maxLoss, t);
    return clamp(1 - loss, definition.min, definition.max);
  }
  return definition.min + t * (definition.max - definition.min);
}

function softLogMin(definition: FluidParamDefinition) {
  if (definition.softMin !== undefined) return Math.max(definition.softMin, Number.EPSILON);
  return Math.max(definition.max / 4096, Number.EPSILON);
}

function lerp(from: number, to: number, t: number) {
  return from + (to - from) * clamp(t, 0, 1);
}

function hoverInfluence(x: number, y: number, pointerX: number, pointerY: number, radius: number) {
  return 1 - clamp(distance(x, y, pointerX, pointerY) / Math.max(radius, 1), 0, 1);
}

function drawDistortedAgentPath(
  ctx: CanvasRenderingContext2D,
  shape: string,
  size: number,
  time: number,
  phase: number,
  amount: number,
) {
  const points = shapeOutlinePoints(shape, size, 72);
  ctx.beginPath();
  points.forEach((point, index) => {
    const angle = Math.atan2(point.y, point.x);
    const radialNoise = perlin3(point.x * 0.036 + phase * 9.7, point.y * 0.036 - phase * 4.3, time * 0.38 + phase);
    const fineNoise = perlin3(point.x * 0.092 - phase * 3.1, point.y * 0.092 + phase * 7.4, time * 0.72 + 17.0);
    const radial = 1 + (radialNoise * 0.72 + fineNoise * 0.28) * amount;
    const tangent = fineNoise * amount * size * 0.055;
    const x = point.x * radial + Math.cos(angle + Math.PI / 2) * tangent;
    const y = point.y * radial + Math.sin(angle + Math.PI / 2) * tangent;
    if (index === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.closePath();
}

function shapeOutlinePoints(shape: string, size: number, segments: number) {
  const r = size / 2;
  if (shape === "kite" || shape === "diamond") {
    return samplePolygon(
      [
        { x: 0, y: -r },
        { x: r * 0.9, y: 0 },
        { x: 0, y: r },
        { x: -r * 0.9, y: 0 },
      ],
      segments,
    );
  }
  if (shape === "hex") {
    const vertices = Array.from({ length: 6 }, (_, index) => {
      const angle = Math.PI / 6 + index * (Math.PI / 3);
      return { x: Math.cos(angle) * r, y: Math.sin(angle) * r };
    });
    return samplePolygon(vertices, segments);
  }
  if (shape === "capsule") return superellipsePoints(r * 1.18, r * 0.74, 3.6, segments);
  if (shape === "lens") return ellipsePoints(r * 1.08, r * 0.76, Math.PI / 4, segments);
  if (shape === "seed") return ellipsePoints(r * 0.82, r * 1.08, Math.PI / 4, segments);
  return ellipsePoints(r, r, 0, segments);
}

function samplePolygon(vertices: { x: number; y: number }[], segments: number) {
  const points: { x: number; y: number }[] = [];
  const stepsPerEdge = Math.max(3, Math.ceil(segments / vertices.length));
  for (let index = 0; index < vertices.length; index += 1) {
    const from = vertices[index];
    const to = vertices[(index + 1) % vertices.length];
    for (let step = 0; step < stepsPerEdge; step += 1) {
      const t = step / stepsPerEdge;
      points.push({ x: lerp(from.x, to.x, t), y: lerp(from.y, to.y, t) });
    }
  }
  return points;
}

function ellipsePoints(rx: number, ry: number, rotation: number, segments: number) {
  const points: { x: number; y: number }[] = [];
  const cosRotation = Math.cos(rotation);
  const sinRotation = Math.sin(rotation);
  for (let index = 0; index < segments; index += 1) {
    const angle = (index / segments) * Math.PI * 2;
    const x = Math.cos(angle) * rx;
    const y = Math.sin(angle) * ry;
    points.push({ x: x * cosRotation - y * sinRotation, y: x * sinRotation + y * cosRotation });
  }
  return points;
}

function superellipsePoints(rx: number, ry: number, exponent: number, segments: number) {
  const points: { x: number; y: number }[] = [];
  for (let index = 0; index < segments; index += 1) {
    const angle = (index / segments) * Math.PI * 2;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    points.push({
      x: Math.sign(cos) * Math.pow(Math.abs(cos), 2 / exponent) * rx,
      y: Math.sign(sin) * Math.pow(Math.abs(sin), 2 / exponent) * ry,
    });
  }
  return points;
}

function perlin3(x: number, y: number, z: number) {
  const x0 = Math.floor(x);
  const y0 = Math.floor(y);
  const z0 = Math.floor(z);
  const xf = x - x0;
  const yf = y - y0;
  const zf = z - z0;
  const u = fade(xf);
  const v = fade(yf);
  const w = fade(zf);
  const n000 = grad3(hash3(x0, y0, z0), xf, yf, zf);
  const n100 = grad3(hash3(x0 + 1, y0, z0), xf - 1, yf, zf);
  const n010 = grad3(hash3(x0, y0 + 1, z0), xf, yf - 1, zf);
  const n110 = grad3(hash3(x0 + 1, y0 + 1, z0), xf - 1, yf - 1, zf);
  const n001 = grad3(hash3(x0, y0, z0 + 1), xf, yf, zf - 1);
  const n101 = grad3(hash3(x0 + 1, y0, z0 + 1), xf - 1, yf, zf - 1);
  const n011 = grad3(hash3(x0, y0 + 1, z0 + 1), xf, yf - 1, zf - 1);
  const n111 = grad3(hash3(x0 + 1, y0 + 1, z0 + 1), xf - 1, yf - 1, zf - 1);
  const x00 = lerp(n000, n100, u);
  const x10 = lerp(n010, n110, u);
  const x01 = lerp(n001, n101, u);
  const x11 = lerp(n011, n111, u);
  const y0Mix = lerp(x00, x10, v);
  const y1Mix = lerp(x01, x11, v);
  return lerp(y0Mix, y1Mix, w);
}

function fade(t: number) {
  return t * t * t * (t * (t * 6 - 15) + 10);
}

function hash3(x: number, y: number, z: number) {
  let h = Math.imul(x, 374761393) ^ Math.imul(y, 668265263) ^ Math.imul(z, 2147483647);
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return h ^ (h >>> 16);
}

function grad3(hash: number, x: number, y: number, z: number) {
  const h = hash & 15;
  const u = h < 8 ? x : y;
  const v = h < 4 ? y : h === 12 || h === 14 ? x : z;
  return ((h & 1) === 0 ? u : -u) + ((h & 2) === 0 ? v : -v);
}

function drawAgentPath(ctx: CanvasRenderingContext2D, shape: string, size: number) {
  const r = size / 2;
  ctx.beginPath();
  if (shape === "kite" || shape === "diamond") {
    ctx.moveTo(0, -r);
    ctx.lineTo(r * 0.9, 0);
    ctx.lineTo(0, r);
    ctx.lineTo(-r * 0.9, 0);
    ctx.closePath();
  } else if (shape === "hex") {
    for (let index = 0; index < 6; index += 1) {
      const angle = Math.PI / 6 + index * (Math.PI / 3);
      const x = Math.cos(angle) * r;
      const y = Math.sin(angle) * r;
      if (index === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.closePath();
  } else if (shape === "capsule") {
    roundedRect(ctx, -r * 1.18, -r * 0.74, r * 2.36, r * 1.48, r * 0.54);
  } else if (shape === "lens") {
    ctx.ellipse(0, 0, r * 1.08, r * 0.76, Math.PI / 4, 0, Math.PI * 2);
  } else if (shape === "seed") {
    ctx.ellipse(0, 0, r * 0.82, r * 1.08, Math.PI / 4, 0, Math.PI * 2);
  } else {
    ctx.arc(0, 0, r, 0, Math.PI * 2);
  }
}

function roundedRect(ctx: CanvasRenderingContext2D, x: number, y: number, width: number, height: number, radius: number) {
  ctx.beginPath();
  ctx.moveTo(x + radius, y);
  ctx.arcTo(x + width, y, x + width, y + height, radius);
  ctx.arcTo(x + width, y + height, x, y + height, radius);
  ctx.arcTo(x, y + height, x, y, radius);
  ctx.arcTo(x, y, x + width, y, radius);
  ctx.closePath();
}

function wrapCanvasText(ctx: CanvasRenderingContext2D, value: string, x: number, y: number, maxWidth: number, lineHeight: number, maxLines: number) {
  const words = value.split(/\s+/);
  let line = "";
  let lineIndex = 0;
  for (const word of words) {
    const test = line ? `${line} ${word}` : word;
    if (ctx.measureText(test).width > maxWidth && line) {
      ctx.fillText(lineIndex + 1 === maxLines ? `${line}...` : line, x, y + lineIndex * lineHeight);
      line = word;
      lineIndex += 1;
      if (lineIndex >= maxLines) return;
    } else {
      line = test;
    }
  }
  if (line && lineIndex < maxLines) {
    ctx.fillText(line, x, y + lineIndex * lineHeight);
  }
}

function distance(ax: number, ay: number, bx: number, by: number) {
  return Math.hypot(ax - bx, ay - by);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function hexAlpha(hex: string, alpha: number) {
  const normalized = hex.replace("#", "");
  const value = Number.parseInt(normalized.length === 3
    ? normalized.split("").map((char) => `${char}${char}`).join("")
    : normalized, 16);
  return `rgba(${(value >> 16) & 255}, ${(value >> 8) & 255}, ${value & 255}, ${clamp(alpha, 0, 1)})`;
}
