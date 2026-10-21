<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  import { t } from "../../lib/copy";

  interface Props {
    link: string;
  }

  let { link }: Props = $props();

  /** "GitHub · app/pull/12" for GitHub, otherwise the host and path. */
  function label(url: string): string {
    try {
      const u = new URL(url);
      const path = u.pathname.replace(/^\/|\/$/g, "");
      return u.hostname === "github.com" ? `${t("item.linkGithub")} · ${path}` : `${u.hostname} · ${path}`;
    } catch {
      return url;
    }
  }
</script>

<section aria-label={t("item.links")}>
  <h3 class="section-label">{t("item.links")}</h3>
  <button type="button" class="link" onclick={() => openUrl(link)}>{label(link)} ↗</button>
</section>

<style>
  h3 {
    margin-bottom: 6px;
  }

  .link {
    padding: 4px 0;
    border: 0;
    background: transparent;
    color: var(--accent);
    font-size: 14px;
    text-align: left;
    word-break: break-all;
  }
</style>
