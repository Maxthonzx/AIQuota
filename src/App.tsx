import { useCallback, useEffect, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import {
  applyTheme,
  readTheme,
  THEME_KEY,
  type ThemePreference,
} from "./theme";
import { toResetDate, formatResetTime } from "./resetTime";
import {
  CONNECTIONS_KEY,
  PROVIDERS,
  makeConnection,
  makeCustom,
  metaOf,
  readConnections,
  requestIsCurrent,
  serializeConnections,
  typeLabel,
  type Connection,
  type ProviderResult,
  type QuotaWindow,
} from "./providers";
import ProviderEditor from "./ProviderEditor";
import { Icon, Modal } from "./ui";
import "./App.css";

const REMIND_KEY = "aiquota_remind_enabled",
  EXHAUSTED_KEY = "aiquota_exhausted";
function preference(key: string, fallback: string) {
  try {
    return localStorage.getItem(key) ?? fallback;
  } catch {
    return fallback;
  }
}
function readInitial() {
  try {
    return { connections: readConnections(localStorage), error: "" };
  } catch {
    return {
      connections: null,
      error: "无法读取供应商配置。原配置已保留，请检查应用存储后重启。",
    };
  }
}
function readExhausted(): Record<string, true> {
  try {
    return JSON.parse(preference(EXHAUSTED_KEY, "{}"));
  } catch {
    return {};
  }
}
function countdown(value: QuotaWindow["resets_at"], now: number) {
  const date = toResetDate(value);
  if (!date) return null;
  const mins = Math.floor((date.getTime() - now) / 60000);
  if (mins < 0) return "已到重置时间";
  if (mins < 60) return `${mins} 分钟后重置`;
  const hours = Math.floor(mins / 60);
  return hours < 24
    ? `${hours} 小时 ${mins % 60} 分后重置`
    : `${Math.floor(hours / 24)} 天 ${hours % 24} 小时后重置`;
}
function WindowRow({ window: w, now }: { window: QuotaWindow; now: number }) {
  const percent = w.used_percent,
    date = toResetDate(w.resets_at);
  return (
    <div className="quota-window">
      <div className="quota-heading">
        <span>{w.label}</span>
        <strong>
          {percent === null ? (
            "—"
          ) : (
            <>
              {Math.round(percent)}
              <small>% 已用</small>
            </>
          )}
        </strong>
      </div>
      {percent !== null && (
        <div
          className="quota-track"
          role="progressbar"
          aria-label={`${w.label}已用`}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.min(100, Math.max(0, percent))}
        >
          <span
            className={
              percent >= 85 ? "critical" : percent >= 60 ? "warning" : ""
            }
            style={{ width: `${Math.min(100, Math.max(0, percent))}%` }}
          />
        </div>
      )}
      {w.status !== "ok" && (
        <span className="quota-limited">
          {w.status === "limited" ? "额度已用尽 / 限流中" : w.status}
        </span>
      )}
      {date && (
        <div className="reset-line">
          <Icon name="clock" />
          <div>
            <span>{countdown(w.resets_at, now)}</span>
            <time dateTime={date.toISOString()} title="按电脑当前时区显示">
              {formatResetTime(w.resets_at)}
            </time>
          </div>
        </div>
      )}
    </div>
  );
}
function ProviderMark({ connection }: { connection: Connection }) {
  const meta = metaOf(connection);
  return (
    <span
      className="provider-mark"
      style={{ "--provider-color": meta.color } as React.CSSProperties}
    >
      {connection.custom ? <Icon name="layers" /> : meta.glyph}
    </span>
  );
}
function Catalog({
  connections,
  onPick,
  onClose,
}: {
  connections: Connection[];
  onPick: (connection: Connection) => void;
  onClose: () => void;
}) {
  const [search, setSearch] = useState(""),
    [filter, setFilter] = useState("all");
  const visible = PROVIDERS.filter(
    (p) =>
      (filter === "all" || p.type === filter) &&
      `${p.name} ${p.description}`.toLowerCase().includes(search.toLowerCase()),
  );
  return (
    <Modal
      title="添加供应商"
      subtitle="选择已有订阅、API 服务，或连接自定义额度接口。"
      onClose={onClose}
      wide
    >
      <div className="catalog-body">
        <div className="search-field">
          <Icon name="search" />
          <input
            aria-label="搜索供应商"
            placeholder="搜索供应商…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            autoFocus
          />
        </div>
        <div className="tabs" role="tablist" aria-label="供应商类型">
          {[
            ["all", "全部"],
            ["subscription", "订阅"],
            ["api", "API"],
          ].map(([id, label]) => (
            <button
              role="tab"
              aria-selected={filter === id}
              key={id}
              onClick={() => setFilter(id)}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="catalog-grid">
          {visible.map((meta) => {
            const existing = connections.find((c) => c.provider === meta.id),
              connection = existing ?? makeConnection(meta);
            return (
              <button
                className="catalog-item"
                key={meta.id}
                onClick={() => onPick(connection)}
              >
                <ProviderMark connection={connection} />
                <span>
                  <strong>{meta.name}</strong>
                  <small>{meta.description}</small>
                </span>
                {existing ? (
                  <span className="badge">已添加</span>
                ) : (
                  <Icon name="plus" />
                )}
              </button>
            );
          })}
          {!visible.length && (
            <p className="catalog-no-results">
              没有匹配的预设，可以添加自定义供应商。
            </p>
          )}
        </div>
        <button className="custom-entry" onClick={() => onPick(makeCustom())}>
          <span className="custom-mark">
            <Icon name="plus" />
          </span>
          <span>
            <strong>自定义供应商</strong>
            <small>连接 JSON 额度接口，支持订阅和 API。</small>
          </span>
          <Icon name="arrow" />
        </button>
      </div>
      <footer className="catalog-footer">
        <Icon name="shield" />
        Key 保存在 Windows 凭据管理器或 macOS 钥匙串。
      </footer>
    </Modal>
  );
}

export default function App() {
  const [initial] = useState(readInitial);
  const [connections, setConnections] = useState<Connection[]>(
    initial.connections ?? [],
  );
  const connectionsRef = useRef(connections);
  const [ready, setReady] = useState(false),
    [error, setError] = useState(initial.error),
    [toast, setToast] = useState("");
  const [page, setPage] = useState<"overview" | "providers" | "settings">(
    "overview",
  );
  useEffect(() => { window.scrollTo({ top: 0 }); }, [page]);
  const [catalog, setCatalog] = useState(false),
    [editing, setEditing] = useState<Connection | null>(null);
  const [results, setResults] = useState<Record<string, ProviderResult[]>>({}),
    [refreshing, setRefreshing] = useState<Record<string, boolean>>({});
  const [lastFetch, setLastFetch] = useState<number | null>(null),
    [now, setNow] = useState(() => Date.now());
  const [filter, setFilter] = useState("all"),
    [search, setSearch] = useState("");
  const [collapsed, setCollapsed] = useState<string[]>(() => {
    try {
      const v = JSON.parse(preference("aiquota_card_layout_v1", "{}"));
      return Array.isArray(v.collapsed) ? v.collapsed : [];
    } catch {
      return [];
    }
  });
  const [theme, setTheme] = useState<ThemePreference>(readTheme);
  const [intervalSec, setIntervalSec] = useState(() => {
    const v = Number(preference("aiquota_refresh_seconds", "120"));
    return [60, 120, 300, 600].includes(v) ? v : 120;
  });
  const [remindEnabled, setRemindEnabled] = useState(
    () => preference(REMIND_KEY, "1") !== "0",
  );
  const remindRef = useRef(remindEnabled),
    exhausted = useRef(readExhausted());
  const sequence = useRef<Record<string, number>>({}),
    active = useRef<Record<string, string | undefined>>({});
  const delayed = useRef<ReturnType<typeof setTimeout>[]>([]);
  const draggedProvider = useRef<string | null>(null);
  const commit = (next: Connection[]) => {
    localStorage.setItem(CONNECTIONS_KEY, serializeConnections(next));
    connectionsRef.current = next;
    setConnections(next);
  };
  useEffect(() => {
    const cleanup = applyTheme(theme);
    try {
      localStorage.setItem(THEME_KEY, theme);
    } catch {
      /* session preference */
    }
    if (isTauri())
      getCurrentWindow()
        .setTheme(theme === "auto" ? null : theme)
        .catch(() => {});
    return cleanup;
  }, [theme]);
  useEffect(() => {
    remindRef.current = remindEnabled;
    try {
      localStorage.setItem(REMIND_KEY, remindEnabled ? "1" : "0");
    } catch {
      /* session preference */
    }
  }, [remindEnabled]);
  useEffect(() => {
    try {
      localStorage.setItem("aiquota_refresh_seconds", String(intervalSec));
    } catch {
      /* session preference */
    }
  }, [intervalSec]);
  useEffect(() => {
    try {
      localStorage.setItem(
        "aiquota_card_layout_v1",
        JSON.stringify({ collapsed }),
      );
    } catch {
      /* session layout */
    }
  }, [collapsed]);
  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => setToast(""), 6000);
      return () => clearTimeout(timer);
    }
  }, [toast]);
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);
  useEffect(() => () => delayed.current.forEach(clearTimeout), []);
  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const credentials = await invoke<
          Record<string, boolean> & { storage_error?: string }
        >("credential_status");
        if (!alive) return;
        if (credentials.storage_error) {
          setError(credentials.storage_error);
          return;
        }
        if (initial.error) return;
        if (initial.connections === null) {
          const detected = PROVIDERS.filter((p) => credentials[p.id]).map(
            makeConnection,
          );
          localStorage.setItem(CONNECTIONS_KEY, serializeConnections(detected));
          connectionsRef.current = detected;
          setConnections(detected);
        }
        setReady(true);
      } catch {
        if (alive) setError("无法加载本机连接配置，请检查系统安全存储后重启。");
      }
    })();
    return () => {
      alive = false;
    };
  }, [initial]);
  const checkRecovery = useCallback(
    (connection: Connection, list: ProviderResult[]) => {
      const saved = { ...exhausted.current },
        recovered: string[] = [];
      for (const result of list) {
        const base = `${connection.id}|${result.account ?? ""}`;
        if (result.status !== "ok") {
          if (result.error?.includes("429")) saved[`${base}|*`] = true;
          continue;
        }
        if (saved[`${base}|*`]) {
          recovered.push(connection.name);
          delete saved[`${base}|*`];
        }
        for (const w of result.windows) {
          const key = `${base}|${w.label}`;
          if (
            (w.used_percent !== null && w.used_percent >= 100) ||
            w.status === "limited"
          )
            saved[key] = true;
          else if (saved[key]) {
            recovered.push(`${connection.name} ${w.label}`);
            delete saved[key];
          }
        }
      }
      exhausted.current = saved;
      try {
        localStorage.setItem(EXHAUSTED_KEY, JSON.stringify(saved));
      } catch {
        /* session state */
      }
      if (remindRef.current && recovered.length)
        isPermissionGranted()
          .then((granted) => {
            if (granted)
              sendNotification({
                title: "额度已恢复",
                body: [...new Set(recovered)].join("、"),
              });
          })
          .catch(() => {});
    },
    [],
  );
  const refreshOne = useCallback(
    async (connection: Connection) => {
      const id = connection.id,
        fingerprint = JSON.stringify(connection);
      if (
        active.current[id] === fingerprint ||
        !connectionsRef.current.some((c) => c.id === id)
      )
        return;
      const request = (sequence.current[id] ?? 0) + 1;
      sequence.current[id] = request;
      active.current[id] = fingerprint;
      setRefreshing((r) => ({ ...r, [id]: true }));
      try {
        const list = connection.custom
          ? [
              await invoke<ProviderResult>("fetch_custom", {
                config: connection.custom,
              }),
            ]
          : await invoke<ProviderResult[]>("fetch_provider", {
              id: connection.provider,
            });
        if (
          !requestIsCurrent(
            connection,
            connectionsRef.current,
            request,
            sequence.current[id],
          )
        )
          return;
        setResults((r) => ({ ...r, [id]: list }));
        setLastFetch(Date.now());
        checkRecovery(connection, list);
      } catch {
        if (
          requestIsCurrent(
            connection,
            connectionsRef.current,
            request,
            sequence.current[id],
          )
        )
          setResults((r) => ({
            ...r,
            [id]: [
              {
                id,
                account: null,
                status: "error",
                error: "查询失败，请稍后刷新或检查连接配置。",
                windows: [],
                extras: [],
                action: null,
              },
            ],
          }));
      } finally {
        if (sequence.current[id] === request) {
          delete active.current[id];
          setRefreshing((r) => ({ ...r, [id]: false }));
        }
      }
    },
    [checkRecovery],
  );
  const refreshAll = useCallback(async () => {
    await Promise.all(connectionsRef.current.map(refreshOne));
  }, [refreshOne]);
  useEffect(() => {
    if (ready) void refreshAll();
  }, [ready, refreshAll]);
  useEffect(() => {
    if (!ready) return;
    const timer = setInterval(() => {
      void refreshAll();
    }, intervalSec * 1000);
    return () => clearInterval(timer);
  }, [ready, intervalSec, refreshAll]);
  const saveConnection = async (
    connection: Connection,
    key: string | null | undefined,
  ) => {
    localStorage.setItem(
      CONNECTIONS_KEY,
      serializeConnections(connectionsRef.current),
    );
    if (key !== undefined)
      await invoke("set_provider_key", { id: connection.id, key });
    const previous = connectionsRef.current,
      next = previous.some((c) => c.id === connection.id)
        ? previous.map((c) => (c.id === connection.id ? connection : c))
        : [...previous, connection];
    sequence.current[connection.id] =
      (sequence.current[connection.id] ?? 0) + 1;
    delete active.current[connection.id];
    commit(next);
    setEditing(null);
    setCatalog(false);
    setToast("供应商已保存，正在查询最新额度。");
    setResults((r) => {
      const copy = { ...r };
      delete copy[connection.id];
      return copy;
    });
    void refreshOne(connection);
  };
  const removeConnection = async (connection: Connection) => {
    localStorage.setItem(
      CONNECTIONS_KEY,
      serializeConnections(connectionsRef.current),
    );
    if (connection.custom)
      await invoke("set_provider_key", { id: connection.id, key: null });
    commit(connectionsRef.current.filter((c) => c.id !== connection.id));
    sequence.current[connection.id] =
      (sequence.current[connection.id] ?? 0) + 1;
    delete active.current[connection.id];
    setResults((r) => {
      const copy = { ...r };
      delete copy[connection.id];
      return copy;
    });
    setRefreshing((r) => {
      const copy = { ...r };
      delete copy[connection.id];
      return copy;
    });
    const history = { ...exhausted.current };
    for (const key of Object.keys(history))
      if (key.startsWith(`${connection.id}|`)) delete history[key];
    exhausted.current = history;
    try {
      localStorage.setItem(EXHAUSTED_KEY, JSON.stringify(history));
    } catch {
      /* session state */
    }
    setEditing(null);
    setToast("已移除供应商，停止自动查询。");
  };
  const move = useCallback((id: string, target: string) => {
    const next = [...connectionsRef.current],
      from = next.findIndex((c) => c.id === id),
      to = next.findIndex((c) => c.id === target);
    if (from < 0 || to < 0 || from === to) return;
    const [item] = next.splice(from, 1);
    next.splice(to, 0, item);
    try {
      localStorage.setItem(CONNECTIONS_KEY, serializeConnections(next));
      connectionsRef.current = next;
      setConnections(next);
    } catch {
      setToast("无法保存排序，请检查应用存储。");
    }
  }, []);
  useEffect(() => {
    const finish = (event: PointerEvent) => {
      const source = draggedProvider.current;
      draggedProvider.current = null;
      document.body.classList.remove("is-dragging");
      if (!source) return;
      const target = document.elementFromPoint(event.clientX, event.clientY)
        ?.closest<HTMLElement>("[data-provider-id]")?.dataset.providerId;
      if (target) move(source, target);
    };
    const cancel = () => {
      draggedProvider.current = null;
      document.body.classList.remove("is-dragging");
    };
    // Track the gesture at the window boundary: a captured button can be
    // replaced by a quota refresh while the user's pointer is still down.
    window.addEventListener("pointerup", finish);
    window.addEventListener("pointercancel", cancel);
    window.addEventListener("blur", cancel);
    return () => {
      window.removeEventListener("pointerup", finish);
      window.removeEventListener("pointercancel", cancel);
      window.removeEventListener("blur", cancel);
      cancel();
    };
  }, [move]);
  const login = async (kind: string, target: string) => {
    await invoke("open_login", { kind, target });
    setToast("已打开登录入口，完成后可保存配置或刷新额度。");
    for (const ms of [15000, 45000])
      delayed.current.push(
        setTimeout(() => {
          void refreshAll();
        }, ms),
      );
  };
  const toggleReminder = async () => {
    if (!remindEnabled) {
      try {
        if (
          !(await isPermissionGranted()) &&
          (await requestPermission()) !== "granted"
        ) {
          setToast("请在系统通知设置中允许 AIQuota 发送提醒。");
          return;
        }
      } catch {
        setToast("无法获取通知权限，请检查系统设置。");
        return;
      }
    }
    setRemindEnabled((v) => !v);
  };
  const testNotification = async () => {
    try {
      if (
        !(await isPermissionGranted()) &&
        (await requestPermission()) !== "granted"
      ) {
        setToast("请在系统通知设置中允许 AIQuota。");
        return;
      }
      sendNotification({
        title: "AIQuota",
        body: "通知已就绪，额度恢复时会提醒你。",
      });
      setToast("测试提醒已发送。");
    } catch {
      setToast("发送失败，请检查系统通知权限。");
    }
  };
  const allResults = connections.flatMap((c) => results[c.id] ?? []),
    connected = connections.filter((c) =>
      results[c.id]?.some((r) => r.status === "ok"),
    ).length;
  const attention = connections.filter((c) =>
    results[c.id]?.some(
      (r) =>
        r.status !== "ok" ||
        r.windows.some(
          (w) => w.status === "limited" || (w.used_percent ?? 0) >= 85,
        ),
    ),
  ).length;
  const nextReset = allResults
    .flatMap((r) => r.windows)
    .map((w) => toResetDate(w.resets_at))
    .filter((d): d is Date => !!d && d.getTime() > now)
    .sort((a, b) => a.getTime() - b.getTime())[0];
  const visible = connections.filter(
    (c) =>
      (filter === "all" || c.type === filter) &&
      `${c.name} ${metaOf(c).name}`
        .toLowerCase()
        .includes(search.toLowerCase()),
  );
  const busy = Object.values(refreshing).some(Boolean);
  const grip = (connection: Connection) => (
    <button
      className="icon-button drag-handle"
      aria-label={`拖动排序：${connection.name}`}
      title="拖动排序，也可按上下方向键"
      onKeyDown={(e) => {
        const i = connections.findIndex((c) => c.id === connection.id),
          target =
            connections[
              i + (e.key === "ArrowUp" ? -1 : e.key === "ArrowDown" ? 1 : 0)
            ];
        if (target && target.id !== connection.id) {
          e.preventDefault();
          move(connection.id, target.id);
        }
      }}
      onPointerDown={(e) => {
        if (e.button !== 0) return;
        e.preventDefault();
        draggedProvider.current = connection.id;
        document.body.classList.add("is-dragging");
      }}
    >
      <Icon name="grip" />
    </button>
  );
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <a
          className="brand"
          href="#"
          aria-label="AIQuota 总览"
          onClick={(e) => {
            e.preventDefault();
            setPage("overview");
          }}
        >
          <span className="brand-symbol">
            AQ
            <span />
          </span>
          <span>
            AIQuota<small>你的 AI 额度管家</small>
          </span>
        </a>
        <div className="nav-caption">工作空间</div>
        <nav aria-label="主导航">
          {[
            ["overview", "grid", "总览"],
            ["providers", "layers", "供应商"],
            ["settings", "settings", "设置"],
          ].map(([id, icon, label]) => (
            <button
              key={id}
              aria-current={page === id ? "page" : undefined}
              onClick={() => {
                setPage(id as typeof page);
                setSearch("");
              }}
            >
              <Icon name={icon} />
              <span>{label}</span>
              {id === "providers" && <small>{connections.length}</small>}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="local-badge">
            <Icon name="shield" />
            <div>
              本机安全存储<small>凭据由你掌控</small>
            </div>
          </div>
          <span className="version">v{__APP_VERSION__}</span>
        </div>
      </aside>
      <main className="workspace">
        <header className="page-header">
          <div>
            <div className="eyebrow">MY AI WORKSPACE</div>
            <h1>
              {page === "overview"
                ? "额度总览"
                : page === "providers"
                  ? "供应商"
                  : "偏好设置"}
            </h1>
            <p>
              {page === "overview"
                ? "每一份订阅与余额，心中有数。"
                : page === "providers"
                  ? "连接你在用的服务，让工作空间保持简洁。"
                  : "让 AIQuota 按你的习惯工作。"}
            </p>
          </div>
          {page !== "settings" && (
            <div className="header-actions">
              {page === "overview" && (
                <button
                  className="button secondary refresh-all"
                  aria-label="刷新全部额度"
                  onClick={() => {
                    void refreshAll();
                  }}
                  disabled={busy || !ready || !connections.length}
                >
                  <Icon name="refresh" className={busy ? "spin" : ""} />
                  <span>刷新</span>
                </button>
              )}
              <button
                className="button primary"
                onClick={() => setCatalog(true)}
                disabled={!ready}
              >
                <Icon name="plus" />
                添加供应商
              </button>
            </div>
          )}
        </header>
        {error && (
          <div className="page-alert" role="alert">
            <Icon name="alert" />
            {error}
          </div>
        )}
        {page === "overview" && (
          <section className="summary-strip" aria-label="额度摘要">
            <div>
              <span>已连接服务</span>
              <strong>
                {connected}
                <small>/ {connections.length}</small>
              </strong>
            </div>
            <div>
              <span>需要关注</span>
              <strong className={attention ? "warning-text" : ""}>
                {attention}
                <small>个服务</small>
              </strong>
            </div>
            <div className="summary-reset">
              <span>
                最近重置 <small>本机时区</small>
              </span>
              <strong>
                {nextReset
                  ? formatResetTime(nextReset.toISOString())
                  : "暂无重置计划"}
              </strong>
            </div>
          </section>
        )}
        {page !== "settings" && (
          <>
            <div className="collection-toolbar">
              <div className="tabs" role="tablist" aria-label="服务筛选">
                {[
                  ["all", "全部"],
                  ["subscription", "订阅"],
                  ["api", "API"],
                ].map(([id, label]) => (
                  <button
                    role="tab"
                    aria-selected={filter === id}
                    key={id}
                    onClick={() => setFilter(id)}
                  >
                    {label}
                    <span>
                      {
                        connections.filter((c) => id === "all" || c.type === id)
                          .length
                      }
                    </span>
                  </button>
                ))}
              </div>
              {page === "providers" ? (
                <div className="search-field compact-search">
                  <Icon name="search" />
                  <input
                    aria-label="搜索已添加的供应商"
                    placeholder="搜索供应商"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                  />
                </div>
              ) : (
                <span className="update-status">
                  <span
                    className={busy ? "status-dot pulsing" : "status-dot"}
                  />
                  {busy
                    ? "正在更新"
                    : lastFetch
                      ? `更新于 ${new Date(lastFetch).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`
                      : "等待连接"}
                </span>
              )}
            </div>
            {!ready && !error && (
              <div className="empty-state">
                <Icon name="refresh" className="spin" />
                <h2>正在读取本机连接…</h2>
              </div>
            )}
            {ready && !connections.length && (
              <div className="empty-state">
                <div className="empty-illustration">
                  <Icon name="layers" />
                  <span>
                    <Icon name="plus" />
                  </span>
                </div>
                <h2>从第一个供应商开始</h2>
                <p>
                  连接已有订阅或 API 账户，
                  <br />
                  在这里查看额度、余额与重置时间。
                </p>
                <button
                  className="button primary"
                  onClick={() => setCatalog(true)}
                >
                  <Icon name="plus" />
                  添加供应商
                </button>
                <small>
                  <Icon name="shield" />
                  Key 保存在你的电脑上
                </small>
              </div>
            )}
            {!!connections.length && !visible.length && (
              <div className="empty-state">
                <Icon name="search" />
                <h2>没有匹配的供应商</h2>
                <p>试试其他名称或切换服务类型。</p>
              </div>
            )}
            {page === "overview" && (
              <div className="quota-grid">
                {visible.flatMap((connection) =>
                  (results[connection.id]?.length
                    ? results[connection.id]
                    : [undefined]
                  ).map((result, index) => {
                    const cardId = `${connection.id}:${result?.account ?? ""}`,
                      isCollapsed =
                        collapsed.includes(cardId) && result?.status !== "ok";
                    return (
                      <article
                        className={`quota-card${isCollapsed ? " is-collapsed" : ""}`}
                        key={`${connection.id}:${result?.account ?? index}`}
                        data-provider-id={connection.id}
                      >
                        <header className="card-header">
                          <ProviderMark connection={connection} />
                          <div className="card-identity">
                            <h2>{connection.name}</h2>
                            <p>
                              {result?.account ??
                                metaOf(connection).description}
                            </p>
                          </div>
                          {grip(connection)}
                          <button
                            className="icon-button"
                            aria-label={`配置 ${connection.name}`}
                            onClick={() => setEditing(connection)}
                          >
                            <Icon name="settings" />
                          </button>
                        </header>
                        <div className="card-meta">
                          <span className="badge">
                            {typeLabel(connection.type)}
                          </span>
                          <span
                            className={`connection-status ${result?.status === "ok" ? "success" : result?.status === "error" ? "warning-text" : "muted"}`}
                          >
                            <span className="status-dot" />
                            {!result
                              ? "连接中"
                              : result.status === "ok"
                                ? "已连接"
                                : result.status === "error"
                                  ? "查询异常"
                                  : "待连接"}
                          </span>
                        </div>
                        {isCollapsed ? (
                          <p className="collapsed-note">
                            {result?.error ?? "完成配置后即可查看额度。"}
                          </p>
                        ) : (
                          <div className="card-content">
                            {!result && (
                              <div className="loading-quota">
                                <span />
                                <span />
                                <span />
                              </div>
                            )}
                            {result?.status === "ok" && (
                              <>
                                {result.windows.map((w, i) => (
                                  <WindowRow window={w} now={now} key={i} />
                                ))}
                                {result.extras.length > 0 && (
                                  <dl
                                    className={`quota-extras${!result.windows.length ? " balance-extras" : ""}`}
                                  >
                                    {result.extras.map(([key, value], i) => (
                                      <div key={i}>
                                        <dt>{key}</dt>
                                        <dd>{value}</dd>
                                      </div>
                                    ))}
                                  </dl>
                                )}
                              </>
                            )}
                            {result && result.status !== "ok" && (
                              <div className="connection-empty">
                                <span className="connection-empty-icon">
                                  <Icon
                                    name={
                                      result.status === "error"
                                        ? "alert"
                                        : "layers"
                                    }
                                  />
                                </span>
                                <strong>
                                  {result.status === "error"
                                    ? "暂时无法获取额度"
                                    : "完成连接，开始查看额度"}
                                </strong>
                                <p>{result.error ?? metaOf(connection).help}</p>
                                <button
                                  className="button secondary compact"
                                  onClick={() => setEditing(connection)}
                                >
                                  配置连接
                                  <Icon name="arrow" />
                                </button>
                              </div>
                            )}
                          </div>
                        )}
                        <footer className="card-footer">
                          <span>
                            {connection.custom
                              ? "自定义额度接口"
                              : metaOf(connection).connection}
                          </span>
                          {result && result.status !== "ok" && (
                            <button
                              className="text-link"
                              onClick={() =>
                                setCollapsed((v) =>
                                  v.includes(cardId)
                                    ? v.filter((k) => k !== cardId)
                                    : [...v, cardId],
                                )
                              }
                            >
                              {isCollapsed ? "展开" : "收起"}
                            </button>
                          )}
                          <button
                            className="icon-button"
                            aria-label={`刷新 ${connection.name}`}
                            onClick={() => {
                              void refreshOne(connection);
                            }}
                            disabled={!!refreshing[connection.id]}
                          >
                            <Icon
                              name="refresh"
                              className={
                                refreshing[connection.id] ? "spin" : ""
                              }
                            />
                          </button>
                        </footer>
                      </article>
                    );
                  }),
                )}
              </div>
            )}
            {page === "providers" && !!visible.length && (
              <>
                <div className="provider-list">
                  {visible.map((connection) => {
                    const index = connections.findIndex(
                      (c) => c.id === connection.id,
                    );
                    return (
                      <div
                        className="provider-list-row"
                        key={connection.id}
                        data-provider-id={connection.id}
                      >
                        {grip(connection)}
                        <ProviderMark connection={connection} />
                        <div className="provider-list-info">
                          <strong>{connection.name}</strong>
                          <p>{metaOf(connection).connection}</p>
                        </div>
                        <span className="badge">
                          {typeLabel(connection.type)}
                        </span>
                        <span className="provider-order-controls">
                          <button
                            className="icon-button"
                            disabled={index === 0}
                            aria-label={`上移 ${connection.name}`}
                            onClick={() =>
                              move(connection.id, connections[index - 1].id)
                            }
                          >
                            ↑
                          </button>
                          <button
                            className="icon-button"
                            disabled={index === connections.length - 1}
                            aria-label={`下移 ${connection.name}`}
                            onClick={() =>
                              move(connection.id, connections[index + 1].id)
                            }
                          >
                            ↓
                          </button>
                        </span>
                        <button
                          className="button secondary compact"
                          onClick={() => setEditing(connection)}
                        >
                          配置
                        </button>
                      </div>
                    );
                  })}
                </div>
                <p className="section-note">
                  <Icon name="grip" />
                  拖动或使用上下箭头排序，总览会同步更新。
                </p>
              </>
            )}
            {page === "overview" && !!connections.length && (
              <p className="section-note">
                <Icon name="clock" />
                按本机时区显示 · 每 {intervalSec / 60} 分钟自动刷新
                <span className="spacer" />
                拖动卡片调整顺序
              </p>
            )}
          </>
        )}
        {page === "settings" && (
          <div className="settings-content">
            <section className="settings-section">
              <h2>外观</h2>
              <p>选择喜欢的外观，也可以让它自然跟随系统。</p>
              <div className="theme-options">
                {[
                  ["auto", "monitor", "自动", "跟随系统"],
                  ["light", "sun", "浅色", "明亮清晰"],
                  ["dark", "moon", "深色", "安静专注"],
                ].map(([id, icon, label, desc]) => (
                  <button
                    className={theme === id ? "selected" : ""}
                    key={id}
                    aria-pressed={theme === id}
                    onClick={() => setTheme(id as ThemePreference)}
                  >
                    <Icon name={icon} />
                    <strong>{label}</strong>
                    <small>{desc}</small>
                    {theme === id && (
                      <span className="theme-check">
                        <Icon name="check" />
                      </span>
                    )}
                  </button>
                ))}
              </div>
            </section>
            <section className="settings-section">
              <h2>刷新与提醒</h2>
              <div className="setting-row">
                <div>
                  <strong>自动刷新间隔</strong>
                  <p>定期获取已添加供应商的最新额度。</p>
                </div>
                <select
                  aria-label="自动刷新间隔"
                  value={intervalSec}
                  onChange={(e) => setIntervalSec(Number(e.target.value))}
                >
                  {[60, 120, 300, 600].map((v) => (
                    <option value={v} key={v}>
                      {v / 60} 分钟
                    </option>
                  ))}
                </select>
              </div>
              <div className="setting-row">
                <div>
                  <strong>额度恢复提醒</strong>
                  <p>已耗尽的额度恢复可用时，通过系统通知提醒。</p>
                </div>
                <button
                  className="toggle"
                  role="switch"
                  aria-label="额度恢复提醒"
                  aria-checked={remindEnabled}
                  onClick={() => {
                    void toggleReminder();
                  }}
                >
                  <span />
                </button>
              </div>
              <button
                className="button secondary compact"
                onClick={() => {
                  void testNotification();
                }}
              >
                发送测试提醒
              </button>
            </section>
            <section className="settings-section privacy-section">
              <Icon name="shield" />
              <div>
                <h2>本地优先，私密连接</h2>
                <p>
                  Key 保存在系统安全存储，仅用于你配置的服务认证。本机 CLI 与
                  IDE
                  登录信息只读访问。自定义接口的名称、查询地址和字段映射保存在本机。
                </p>
                <small>AIQuota v{__APP_VERSION__} · MIT License</small>
              </div>
            </section>
          </div>
        )}
      </main>
      {toast && (
        <div className="toast" role="status">
          <Icon name="check" />
          {toast}
        </div>
      )}
      {catalog && !editing && (
        <Catalog
          connections={connections}
          onClose={() => setCatalog(false)}
          onPick={setEditing}
        />
      )}
      {editing && (
        <ProviderEditor
          connection={editing}
          existing={connections.some((c) => c.id === editing.id)}
          onClose={() => setEditing(null)}
          onSave={saveConnection}
          onRemove={() => removeConnection(editing)}
          onAccountsChange={() => {
            const current = connectionsRef.current.find(
              (c) => c.id === editing.id,
            );
            if (current) {
              delete active.current[current.id];
              void refreshOne(current);
            }
          }}
          onLogin={login}
        />
      )}
    </div>
  );
}
