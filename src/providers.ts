export interface QuotaWindow {
  label: string;
  used_percent: number | null;
  resets_at: string | number | null;
  status: string;
}
export interface ProviderResult {
  id: string;
  account: string | null;
  status: "ok" | "error" | "no_credentials";
  error: string | null;
  windows: QuotaWindow[];
  extras: [string, string][];
  action: { label: string; kind: string; target: string } | null;
}
export interface CustomConfig {
  id: string;
  endpoint: string;
  auth: "none" | "bearer" | "header";
  header: string;
  metric: "balance" | "percent" | "usage";
  value_path: string;
  total_path: string;
  reset_path: string;
  unit: string;
}
export interface Connection {
  id: string;
  provider: string;
  name: string;
  type: "subscription" | "api";
  custom?: CustomConfig;
}
export interface ProviderMeta {
  id: string;
  name: string;
  description: string;
  color: string;
  glyph: string;
  type: Connection["type"];
  connection: string;
  help: string;
  keyLabel?: string;
  keyHint?: string;
  manual?: boolean;
  login?: { kind: string; target: string; label: string };
}
export const PROVIDERS: ProviderMeta[] = [
  {
    id: "codex",
    name: "ChatGPT / Codex",
    description: "ChatGPT 订阅 · Codex 额度",
    color: "#16866d",
    glyph: "C",
    type: "subscription",
    connection: "本机 Codex 登录",
    help: "安装 Codex CLI 并登录 ChatGPT 账号。AIQuota 自动读取本机登录状态，展示订阅额度。",
    login: { kind: "terminal", target: "codex", label: "打开 Codex 登录" },
  },
  {
    id: "kimi",
    name: "Kimi Code",
    description: "Coding 订阅 · 多账号",
    color: "#7362d7",
    glyph: "K",
    type: "subscription",
    connection: "本机登录或套餐 Key",
    help: "自动读取 Kimi Code 登录配置，也可使用自己的 Coding 订阅 Key。",
    manual: true,
    keyLabel: "Kimi Code Key",
    keyHint: "使用 Kimi Code 的订阅 Key；多个账号可在下方单独管理。",
    login: { kind: "terminal", target: "kimi", label: "打开 Kimi Code" },
  },
  {
    id: "google",
    name: "Google AI Pro",
    description: "Antigravity · Gemini 额度",
    color: "#437bd3",
    glyph: "G",
    type: "subscription",
    connection: "本机 Antigravity 登录",
    help: "安装并登录 Antigravity，保持应用及语言服务运行。AIQuota 通过本机服务查询额度。",
    login: { kind: "app", target: "Antigravity", label: "打开 Antigravity" },
  },
  {
    id: "opencode",
    name: "OpenCode Go",
    description: "Go 订阅 · 周期额度",
    color: "#bf7134",
    glyph: "O",
    type: "subscription",
    connection: "本机登录或 Go Key",
    help: "自动读取 OpenCode 当前选中的 Go 账号，或手动配置 Go 订阅 Key。",
    manual: true,
    keyLabel: "OpenCode Go Key",
    keyHint: "使用 Go 订阅对应的 Key。",
    login: {
      kind: "terminal",
      target: "opencode auth login",
      label: "打开 OpenCode 登录",
    },
  },
  {
    id: "cursor",
    name: "Cursor",
    description: "IDE 订阅 · 共享额度",
    color: "#667183",
    glyph: "↗",
    type: "subscription",
    connection: "本机 Cursor 登录",
    help: "安装 Cursor 并登录账号。AIQuota 只读访问本机登录信息，不修改 IDE 配置。",
    login: { kind: "app", target: "Cursor", label: "打开 Cursor" },
  },
  {
    id: "glm",
    name: "GLM Coding Plan",
    description: "智谱订阅 · 套餐 Key",
    color: "#298b9a",
    glyph: "Z",
    type: "subscription",
    connection: "套餐 Key",
    help: "使用智谱 Coding Plan 套餐页提供的 Key 连接订阅。",
    keyLabel: "Coding Plan 套餐 Key",
    keyHint:
      "请复制 bigmodel.cn 个人套餐页的套餐 Key，普通按量 API Key 无法查询此订阅。",
  },
  {
    id: "deepseek",
    name: "DeepSeek",
    description: "API · 账户余额",
    color: "#5268c9",
    glyph: "D",
    type: "api",
    connection: "API Key",
    help: "连接 DeepSeek 开放平台，查看 API 账户的可用余额。",
    keyLabel: "API Key",
    keyHint: "在 DeepSeek 开放平台创建的 API Key，仅用于账户余额查询。",
  },
];
export const CONNECTIONS_KEY = "aiquota_connections_v1";
export const typeLabel = (type: Connection["type"]) =>
  type === "subscription" ? "订阅" : "API";
export const metaOf = (connection: Connection): ProviderMeta =>
  PROVIDERS.find((p) => p.id === connection.provider) ?? {
    id: connection.id,
    name: connection.name,
    description: "自定义额度接口",
    color: "#8b6cbb",
    glyph: connection.name.charAt(0).toUpperCase(),
    type: connection.type,
    connection: "自定义 API",
    help: "通过自己的额度接口查询余额或使用量。",
  };
export function makeConnection(meta: ProviderMeta): Connection {
  return { id: meta.id, provider: meta.id, name: meta.name, type: meta.type };
}
export function makeCustom(): Connection {
  const id = `custom-${crypto.randomUUID()}`;
  return {
    id,
    provider: "custom",
    name: "",
    type: "api",
    custom: {
      id,
      endpoint: "",
      auth: "bearer",
      header: "x-api-key",
      metric: "balance",
      value_path: "data.balance",
      total_path: "",
      reset_path: "",
      unit: "USD",
    },
  };
}
/** Allowlist serialized fields so credentials cannot accidentally reach localStorage. */
export function serializeConnections(connections: Connection[]): string {
  return JSON.stringify(
    connections.map((c) => ({
      id: c.id,
      provider: c.provider,
      name: c.name,
      type: c.type,
      ...(c.custom
        ? {
            custom: {
              id: c.id,
              endpoint: c.custom.endpoint,
              auth: c.custom.auth,
              header: c.custom.header,
              metric: c.custom.metric,
              value_path: c.custom.value_path,
              total_path: c.custom.total_path,
              reset_path: c.custom.reset_path,
              unit: c.custom.unit,
            },
          }
        : {}),
    })),
  );
}
export function readConnections(
  storage: Pick<Storage, "getItem">,
): Connection[] | null {
  const value = storage.getItem(CONNECTIONS_KEY);
  if (value === null) return null;
  const parsed: unknown = JSON.parse(value);
  if (!Array.isArray(parsed)) throw Error("供应商配置格式无效");
  const ids = new Set<string>();
  for (const c of parsed) {
    if (
      !c ||
      typeof c !== "object" ||
      typeof c.id !== "string" ||
      typeof c.name !== "string" ||
      !["api", "subscription"].includes(c.type) ||
      ids.has(c.id)
    )
      throw Error("供应商配置格式无效");
    if (c.provider === "custom") {
      if (
        !/^custom-[a-zA-Z0-9-]{1,73}$/.test(c.id) ||
        !c.custom ||
        c.custom.id !== c.id ||
        [
          "endpoint",
          "auth",
          "header",
          "metric",
          "value_path",
          "total_path",
          "reset_path",
          "unit",
        ].some((k) => typeof c.custom[k] !== "string")
      )
        throw Error("自定义供应商配置格式无效");
    } else if (!PROVIDERS.some((p) => p.id === c.id && p.id === c.provider))
      throw Error("未知的供应商配置");
    ids.add(c.id);
  }
  return JSON.parse(serializeConnections(parsed));
}
/** Compare the complete configuration, not just the ID: edits invalidate in-flight queries. */
export function requestIsCurrent(
  snapshot: Connection,
  current: Connection[],
  sequence: number,
  latest: number,
): boolean {
  return (
    sequence === latest &&
    current.some(
      (c) =>
        c.id === snapshot.id && JSON.stringify(c) === JSON.stringify(snapshot),
    )
  );
}
