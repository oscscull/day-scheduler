<script lang="ts">
  import { t } from "../../lib/copy";
  import { longDate } from "../../lib/time";

  interface Props {
    until: string | null;
    onResume: () => void;
  }

  let { until, onResume }: Props = $props();
</script>

<div class="banner" role="status">
  <span class="text">
    <span class="title">
      {until ? t("pause.bannerUntil", { when: longDate(new Date(until)) }) : t("pause.bannerIndefinite")}
    </span>
    <span class="sub">{t("pause.bannerHint")}</span>
  </span>
  <button type="button" class="button-primary" onclick={onResume}>{t("pause.resume")}</button>
</div>

<style>
  .banner {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    padding: 14px 16px;
    border-radius: var(--radius);
    background: var(--paused-bg);
  }

  .text {
    flex: 1 1 280px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .title {
    font-weight: 500;
  }

  .sub {
    font-size: 13px;
    color: #4a4843;
  }
</style>
