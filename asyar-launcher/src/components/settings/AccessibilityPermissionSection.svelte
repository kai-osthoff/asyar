<script lang="ts">
  import { onMount } from 'svelte';
  import SettingsCard from './SettingsCard.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import StatusDot from '../base/StatusDot.svelte';
  import Button from '../base/Button.svelte';
  import { accessibilityStatusService as svc } from '../../services/permissions/accessibilityStatus.svelte';
  import { t } from '../../services/i18n';

  const RESET_COMMAND = 'sudo tccutil reset Accessibility org.asyar.app';

  let repairMessage = $state<string | null>(null);
  let copied = $state(false);

  let dot = $derived(
    svc.status === 'granted'
      ? { color: 'success' as const, label: t('settings.privacy.accessibility_granted') }
      : svc.status === 'stale_grant'
        ? { color: 'warning' as const, label: t('settings.privacy.accessibility_not_in_effect') }
        : svc.status === 'unknown'
          ? { color: 'info' as const, label: t('settings.privacy.accessibility_unavailable') }
          : { color: 'warning' as const, label: t('settings.privacy.accessibility_not_granted') },
  );

  let description = $derived(
    svc.status === 'granted'
      ? t('settings.privacy.accessibility_desc_granted')
      : svc.status === 'stale_grant'
        ? t('settings.privacy.accessibility_desc_stale')
        : svc.status === 'unknown'
          ? t('settings.privacy.accessibility_desc_unavailable')
          : t('settings.privacy.accessibility_desc_not_granted'),
  );

  async function copyCommand() {
    await navigator.clipboard.writeText(RESET_COMMAND);
    copied = true;
  }

  async function runRepair() {
    repairMessage = null;
    const result = await svc.repair();
    repairMessage = result.ok
      ? t('settings.privacy.accessibility_repair_done')
      : (result.error ?? null);
  }

  onMount(() => {
    void svc.init();
    // The grant is made in another application, so the value can only have
    // changed while this window was in the background.
    const onFocus = () => void svc.refresh();
    window.addEventListener('focus', onFocus);
    return () => window.removeEventListener('focus', onFocus);
  });
</script>

<div class="section-header">{t('settings.privacy.accessibility')}</div>
<SettingsCard>
  <SettingsRow label={t('settings.privacy.accessibility_permission')} {description}>
    {#snippet children()}
      <div class="status-row">
        <StatusDot color={dot.color} />
        <span class="text-body">{dot.label}</span>
      </div>
    {/snippet}
  </SettingsRow>

  {#if svc.status !== 'granted'}
    <div class="actions">
      <Button onclick={() => svc.requestPermission()}
        >{t('settings.privacy.accessibility_grant')}</Button
      >
      <button class="link" onclick={() => svc.openPreferences()}>
        {t('settings.privacy.accessibility_open_settings')}
      </button>
    </div>
  {/if}

  {#if svc.status === 'stale_grant'}
    <div class="repair">
      <ol class="steps">
        <li>{t('settings.privacy.accessibility_repair_step_remove')}</li>
        <li>{t('settings.privacy.accessibility_repair_step_restart')}</li>
        <li>{t('settings.privacy.accessibility_repair_step_grant')}</li>
      </ol>
      <div class="command-row">
        <code>{RESET_COMMAND}</code>
        <button class="link" onclick={copyCommand}
          >{copied
            ? t('settings.privacy.accessibility_copied')
            : t('settings.privacy.accessibility_copy')}</button
        >
      </div>
      <div class="auto-repair">
        <Button onclick={runRepair}>
          {t('settings.privacy.accessibility_repair_auto')}
        </Button>
        {#if repairMessage}
          <p class="repair-message">{repairMessage}</p>
        {/if}
      </div>
    </div>
  {/if}
</SettingsCard>

<style>
  .status-row {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) 0 0;
  }
  .repair {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-top: var(--space-4);
  }
  .steps {
    margin: 0;
    padding-left: var(--space-5);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }
  .command-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .command-row code {
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    padding: var(--space-1) var(--space-2);
  }
  .auto-repair {
    border-top: 1px solid var(--separator);
    padding-top: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    align-items: flex-start;
  }
  .repair-message {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .link {
    background: none;
    border: none;
    padding: 0;
    color: var(--asyar-brand);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
</style>
