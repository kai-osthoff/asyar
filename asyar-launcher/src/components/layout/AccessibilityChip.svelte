<script lang="ts">
  import StatusDot from '../base/StatusDot.svelte';
  import { accessibilityStatusService as svc } from '../../services/permissions/accessibilityStatus.svelte';
  import { showSettingsWindow } from '../../lib/ipc/commands';
  import { t } from '../../services/i18n';

  // Sits in the bottom bar's fixed-height cluster rather than in the results
  // area: the launcher shell is a deliberately static 480px layout, and a
  // height-changing element there costs a WebKit relayout on every show.
  // It also stays out of the feedback slot, which FeedbackBar shares with
  // InformationPanel — a permanent notice there would evict the panel for good.
  async function openSettings() {
    // 'privacy' is the tab id from settingsTabs (routes/settings/+page.svelte);
    // showSettingsWindow forwards it via the asyar:navigate-settings-tab event.
    await showSettingsWindow('privacy');
  }
</script>

{#if svc.shouldWarn}
  <button
    class="ax-chip"
    onclick={openSettings}
    title={svc.status === 'stale_grant'
      ? t('settings.privacy.accessibility_chip_stale')
      : t('settings.privacy.accessibility_chip_not_granted')}
  >
    <StatusDot color="warning" />
    <span>{t('settings.privacy.accessibility_cant_paste')}</span>
  </button>
{/if}

<style>
  .ax-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    background: none;
    border: none;
    padding: 0 var(--space-2);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    white-space: nowrap;
  }
  .ax-chip:hover {
    color: var(--text-primary);
  }
</style>
