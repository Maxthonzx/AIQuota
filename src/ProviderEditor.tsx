import { useEffect, useState, type FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { metaOf, type Connection, type CustomConfig } from "./providers";
import { Icon, Modal } from "./ui";

export default function ProviderEditor({
  connection,
  existing,
  onClose,
  onSave,
  onRemove,
  onAccountsChange,
  onLogin,
}: {
  connection: Connection;
  existing: boolean;
  onClose: () => void;
  onSave: (
    connection: Connection,
    key: string | null | undefined,
  ) => Promise<void>;
  onRemove: () => Promise<void>;
  onAccountsChange: () => void;
  onLogin: (kind: string, target: string) => Promise<void>;
}) {
  const [draft, setDraft] = useState(connection);
  const [key, setKey] = useState("");
  const [visible, setVisible] = useState(false);
  const [hasKey, setHasKey] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [mode, setMode] = useState<"local" | "key">("local");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [removing, setRemoving] = useState(false);
  const [accounts, setAccounts] = useState<string[]>([]);
  const [accountName, setAccountName] = useState("");
  const [accountKey, setAccountKey] = useState("");
  const meta = metaOf(draft);
  const custom = draft.custom;
  const needsKey = custom
    ? custom.auth !== "none"
    : !!meta.keyLabel && (!meta.manual || mode === "key");
  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        if (connection.custom || meta.keyLabel) {
          const saved = await invoke<boolean>("has_provider_key", {
            id: connection.id,
          });
          if (alive) {
            setHasKey(saved);
            if (saved) setMode("key");
          }
        }
        if (connection.provider === "kimi") {
          const list = await invoke<string[]>("list_kimi_accounts");
          if (alive) setAccounts(list);
        }
      } catch {
        if (alive) setError("无法读取安全存储，请解锁系统凭据存储后重试。");
      } finally {
        if (alive) setLoaded(true);
      }
    })();
    return () => {
      alive = false;
    };
  }, [connection.id, connection.provider, connection.custom, meta.keyLabel]);
  const updateCustom = <K extends keyof CustomConfig>(
    field: K,
    value: CustomConfig[K],
  ) => setDraft((d) => ({ ...d, custom: { ...d.custom!, [field]: value } }));
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setError("");
    if (!draft.name.trim()) {
      setError("请填写供应商名称。");
      return;
    }
    if (needsKey && !hasKey && !key.trim()) {
      setError("请填写 API Key 后保存。");
      return;
    }
    if (custom) {
      try {
        const url = new URL(custom.endpoint);
        const loopback = ["localhost", "127.0.0.1", "[::1]"].includes(
          url.hostname,
        );
        if (
          url.protocol !== "https:" &&
          !(url.protocol === "http:" && loopback)
        )
          throw Error();
        if (url.username || url.password || url.hash) throw Error();
      } catch {
        setError(
          "请输入完整的 HTTPS 额度查询地址，本机 localhost 可使用 HTTP。地址中不要填写 Key。",
        );
        return;
      }
      if (
        !custom.value_path.trim() ||
        (custom.metric === "usage" && !custom.total_path.trim())
      ) {
        setError("请填写额度字段路径。");
        return;
      }
      for (const path of [
        custom.value_path,
        custom.total_path,
        custom.reset_path,
      ]) {
        if (path && !/^[a-zA-Z0-9_-]+(?:\.[a-zA-Z0-9_-]+)*$/.test(path)) {
          setError(
            "字段路径请使用 data.balance 或 data.items.0.used 这样的点号格式。",
          );
          return;
        }
      }
    }
    setBusy(true);
    try {
      const next = { ...draft, name: draft.name.trim() };
      const secret = needsKey
        ? key.trim() || undefined
        : meta.manual || custom
          ? null
          : undefined;
      await onSave(next, secret);
      setKey("");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const mutateAccount = async (label?: string) => {
    setBusy(true);
    setError("");
    try {
      if (label) await invoke("remove_kimi_account", { label });
      else
        await invoke("add_kimi_account", {
          label: accountName,
          key: accountKey,
        });
      setAccounts(await invoke<string[]>("list_kimi_accounts"));
      setAccountName("");
      setAccountKey("");
      onAccountsChange();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const login = async () => {
    if (!meta.login) return;
    setBusy(true);
    setError("");
    try {
      await onLogin(meta.login.kind, meta.login.target);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const remove = async () => {
    setBusy(true);
    setError("");
    try {
      await onRemove();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal
      title={existing ? "配置供应商" : "连接供应商"}
      subtitle="将订阅额度与 API 余额放在同一个地方。"
      onClose={onClose}
      busy={busy}
    >
      <form onSubmit={submit}>
        <fieldset className="form-body" disabled={busy}>
          <div className="connection-banner">
            <span
              className="provider-mark"
              style={{ "--provider-color": meta.color } as React.CSSProperties}
            >
              {custom ? <Icon name="layers" /> : meta.glyph}
            </span>
            <div>
              <strong>{custom ? "自定义供应商" : meta.name}</strong>
              <p>{meta.connection}</p>
            </div>
            {hasKey && (
              <span className="badge success">
                <Icon name="shield" />
                Key 已保存
              </span>
            )}
          </div>
          <label className="field">
            显示名称
            <input
              autoFocus
              value={draft.name}
              maxLength={60}
              placeholder="例如：我的 DeepSeek"
              onChange={(e) =>
                setDraft((d) => ({ ...d, name: e.target.value }))
              }
              required
            />
          </label>
          {custom && (
            <>
              <label className="field">
                服务类型
                <select
                  value={draft.type}
                  onChange={(e) =>
                    setDraft((d) => ({
                      ...d,
                      type: e.target.value as Connection["type"],
                    }))
                  }
                >
                  <option value="api">API 按量服务</option>
                  <option value="subscription">订阅 / Coding Plan</option>
                </select>
              </label>
              <label className="field">
                额度查询地址
                <input
                  type="url"
                  value={custom.endpoint}
                  onChange={(e) => updateCustom("endpoint", e.target.value)}
                  placeholder="https://api.example.com/account/balance"
                  maxLength={2048}
                  required
                />
                <small>
                  填写返回 JSON 的 GET 额度接口，不是模型对话地址。Key
                  请填在下方认证栏。
                </small>
              </label>
              <label className="field">
                认证方式
                <select
                  value={custom.auth}
                  onChange={(e) =>
                    updateCustom("auth", e.target.value as CustomConfig["auth"])
                  }
                >
                  <option value="bearer">Bearer Token</option>
                  <option value="header">API Key Header</option>
                  <option value="none">无需认证</option>
                </select>
              </label>
              {custom.auth === "header" && (
                <label className="field">
                  认证 Header
                  <select
                    value={custom.header}
                    onChange={(e) => updateCustom("header", e.target.value)}
                  >
                    {[
                      "x-api-key",
                      "api-key",
                      "x-goog-api-key",
                      "Authorization",
                    ].map((h) => (
                      <option key={h}>{h}</option>
                    ))}
                  </select>
                  <small>
                    Header 认证会原样发送 Key；Bearer 认证自动添加 Bearer 前缀。
                  </small>
                </label>
              )}
            </>
          )}
          {!custom && meta.manual && (
            <div className="field">
              <span>连接方式</span>
              <div className="segmented">
                <button
                  type="button"
                  aria-pressed={mode === "local"}
                  onClick={() => {
                    setMode("local");
                    setKey("");
                  }}
                >
                  读取本机登录
                </button>
                <button
                  type="button"
                  aria-pressed={mode === "key"}
                  onClick={() => setMode("key")}
                >
                  手动填写 Key
                </button>
              </div>
            </div>
          )}
          {!custom && !needsKey && (
            <div className="info-box">
              <Icon name="monitor" />
              <div>
                <strong>{meta.connection}</strong>
                <p>{meta.help}</p>
                {meta.login && (
                  <button
                    type="button"
                    className="button secondary compact"
                    onClick={login}
                  >
                    {meta.login.label}
                    <Icon name="arrow" />
                  </button>
                )}
                <small>登录完成后保存，AIQuota 会重新检测连接。</small>
              </div>
            </div>
          )}
          {needsKey && (
            <div className="field">
              <label htmlFor="provider-key">
                {custom ? "API Key / Token" : meta.keyLabel}
              </label>
              <div className="secret-input">
                <input
                  id="provider-key"
                  type={visible ? "text" : "password"}
                  value={key}
                  autoComplete="new-password"
                  spellCheck={false}
                  maxLength={2048}
                  placeholder={
                    hasKey ? "已安全保存，留空保持不变" : "粘贴你的 Key"
                  }
                  onChange={(e) => setKey(e.target.value)}
                  aria-describedby="key-help"
                />
                <button
                  type="button"
                  className="icon-button"
                  aria-label={visible ? "隐藏 Key" : "显示 Key"}
                  aria-pressed={visible}
                  onClick={() => setVisible((v) => !v)}
                >
                  <Icon name={visible ? "hide" : "eye"} />
                </button>
              </div>
              <small id="key-help">
                {custom ? "仅发送到你配置的额度接口。" : meta.keyHint}
              </small>
              <div className="secure-note">
                <Icon name="shield" />
                保存在系统安全存储，不会回显已保存的 Key。
              </div>
            </div>
          )}
          {custom && (
            <section className="mapping-panel">
              <div>
                <h3>额度显示</h3>
                <p>按接口的 JSON 响应设置字段路径。</p>
              </div>
              <label className="field">
                显示方式
                <select
                  value={custom.metric}
                  onChange={(e) =>
                    updateCustom(
                      "metric",
                      e.target.value as CustomConfig["metric"],
                    )
                  }
                >
                  <option value="balance">可用余额</option>
                  <option value="percent">已用百分比（0–100）</option>
                  <option value="usage">已用量 / 总额度</option>
                </select>
              </label>
              <div className="form-columns">
                <label className="field">
                  {custom.metric === "balance"
                    ? "余额字段"
                    : custom.metric === "percent"
                      ? "已用百分比字段"
                      : "已用量字段"}
                  <input
                    value={custom.value_path}
                    placeholder="data.balance"
                    onChange={(e) => updateCustom("value_path", e.target.value)}
                    maxLength={200}
                    required
                  />
                </label>
                {custom.metric === "usage" ? (
                  <label className="field">
                    总额度字段
                    <input
                      value={custom.total_path}
                      placeholder="data.total"
                      onChange={(e) =>
                        updateCustom("total_path", e.target.value)
                      }
                      maxLength={200}
                      required
                    />
                  </label>
                ) : (
                  <label className="field">
                    单位（可选）
                    <input
                      value={custom.unit}
                      placeholder="USD / CNY / tokens"
                      onChange={(e) => updateCustom("unit", e.target.value)}
                      maxLength={16}
                    />
                  </label>
                )}
              </div>
              {custom.metric === "usage" && (
                <label className="field">
                  单位（可选）
                  <input
                    value={custom.unit}
                    onChange={(e) => updateCustom("unit", e.target.value)}
                    maxLength={16}
                    placeholder="tokens"
                  />
                </label>
              )}
              <label className="field">
                重置时间字段（可选）
                <input
                  value={custom.reset_path}
                  placeholder="data.reset_at"
                  onChange={(e) => updateCustom("reset_path", e.target.value)}
                  maxLength={200}
                />
                <small>
                  支持 ISO 时间、秒或毫秒时间戳，按电脑当前时区显示。
                </small>
              </label>
              <details>
                <summary>字段路径示例</summary>
                <pre>{'{ "data": { "balance": 12.50 } }'}</pre>
                <p>
                  余额字段填 <code>data.balance</code>。数组可用{" "}
                  <code>data.items.0.used</code>。
                </p>
              </details>
            </section>
          )}
          {connection.provider === "kimi" && existing && (
            <details className="accounts-panel">
              <summary>
                管理其他 Kimi 账号{" "}
                <span className="muted">{accounts.length} 个</span>
              </summary>
              {accounts.map((label) => (
                <div className="account-row" key={label}>
                  <span>{label}</span>
                  <button
                    type="button"
                    className="icon-button danger"
                    aria-label={`删除账号 ${label}`}
                    onClick={() => mutateAccount(label)}
                  >
                    <Icon name="trash" />
                  </button>
                </div>
              ))}
              <div className="field">
                <label htmlFor="account-name">账号备注</label>
                <input
                  id="account-name"
                  placeholder="例如：工作账号"
                  value={accountName}
                  onChange={(e) => setAccountName(e.target.value)}
                  maxLength={60}
                />
              </div>
              <div className="field">
                <label htmlFor="account-key">账号 Key</label>
                <input
                  id="account-key"
                  type="password"
                  placeholder="Kimi Code Key"
                  value={accountKey}
                  onChange={(e) => setAccountKey(e.target.value)}
                  autoComplete="new-password"
                  maxLength={2048}
                />
              </div>
              <button
                type="button"
                className="button secondary compact"
                disabled={!accountName.trim() || !accountKey.trim()}
                onClick={() => mutateAccount()}
              >
                添加账号
              </button>
            </details>
          )}
          {error && (
            <div className="form-error" role="alert">
              <Icon name="alert" />
              {error}
            </div>
          )}
          {removing && (
            <div className="remove-confirm">
              <p>
                {custom
                  ? "移除后会停止查询，并删除此自定义服务在 AIQuota 保存的 Key。"
                  : "移除后停止查询并隐藏所有相关卡片。已保存的 Key 和本机应用登录保留，可随时重新添加。"}
              </p>
              <button
                type="button"
                className="button danger-button compact"
                onClick={remove}
              >
                确认移除
              </button>
              <button
                type="button"
                className="button text-button compact"
                onClick={() => setRemoving(false)}
              >
                取消
              </button>
            </div>
          )}
        </fieldset>
        <footer className="modal-footer">
          {existing && (
            <button
              type="button"
              className="button text-button danger"
              onClick={() => setRemoving(true)}
              disabled={busy}
            >
              <Icon name="trash" />
              移除
            </button>
          )}
          <span className="spacer" />
          <button
            type="button"
            className="button secondary"
            onClick={onClose}
            disabled={busy}
          >
            取消
          </button>
          <button
            type="submit"
            className="button primary"
            disabled={busy || !loaded}
          >
            {busy ? "处理中…" : existing ? "保存并刷新" : "添加并连接"}
          </button>
        </footer>
      </form>
    </Modal>
  );
}
